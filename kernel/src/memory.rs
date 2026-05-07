//! First-pass physical memory, paging, and heap setup.
//!
//! Physical memory is managed in 4KiB frames because that is the default page
//! size used by x86_64 page tables. Chronosapian starts with identity-mapped heap
//! pages so the virtual address equals the physical address, which keeps early
//! memory behavior easy to inspect while the kernel is still small.

use alloc::alloc::{alloc, Layout};
use core::alloc::{GlobalAlloc, Layout as CoreLayout};
use core::cell::UnsafeCell;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

use crate::boot::{BootContext, MemoryRegion, MemoryRegionKind};

pub const HEAP_START: u64 = 0x200000;
pub const HEAP_SIZE: u64 = 1024 * 1024;
pub const USER_CODE_START: u64 = 0x400000;
pub const USER_STACK_START: u64 = 0x401000;
pub const USER_STACK_SIZE: u64 = PAGE_SIZE;
pub const USER_ELF_BASE: u64 = 0x0000_0080_0000_0000;
pub const USER_ELF_LIMIT: u64 = 0x0000_0100_0000_0000;
pub const USER_ELF_STACK_TOP: u64 = USER_ELF_BASE + 0x8000_0000;
pub const USER_ELF_STACK_SIZE: u64 = PAGE_SIZE * 4;
pub const USER_ELF_PML4_INDEX: usize = 1;

pub const PAGE_SIZE: u64 = 4096;

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

static TOTAL_MEMORY_BYTES: AtomicU64 = AtomicU64::new(0);
static MEMORY: GlobalMemory = GlobalMemory(UnsafeCell::new(None));

struct GlobalMemory(UnsafeCell<Option<MemoryState>>);

unsafe impl Sync for GlobalMemory {}

struct MemoryState {
    physical_memory_offset: VirtAddr,
    frame_allocator: BootInfoFrameAllocator,
}

#[derive(Clone, Copy)]
pub struct AddressSpace {
    pub pml4_frame: PhysFrame<Size4KiB>,
}

#[derive(Clone, Copy)]
pub struct MemoryStats {
    pub total_memory_bytes: u64,
    pub heap_start: u64,
    pub heap_size_bytes: u64,
    pub heap_used_bytes: u64,
}

pub fn init(boot_context: &'static BootContext) {
    TOTAL_MEMORY_BYTES.store(total_memory_bytes(boot_context.memory_regions), Ordering::Relaxed);

    if !heap_range_is_usable(boot_context.memory_regions) {
        crate::serial_println!(
            "[CHRONO] mem: heap range {:#x}..{:#x} is not usable",
            HEAP_START,
            HEAP_START + HEAP_SIZE
        );
        panic!("heap range is not usable");
    }
    if !user_demo_range_is_usable(boot_context.memory_regions) {
        crate::serial_println!(
            "[CHRONO] mem: ring3 demo range {:#x}..{:#x} is not usable",
            USER_CODE_START,
            USER_STACK_START + USER_STACK_SIZE
        );
        panic!("ring3 demo range is not usable");
    }
    if ranges_overlap(
        USER_CODE_START,
        USER_STACK_START + USER_STACK_SIZE,
        boot_context.kernel_addr,
        boot_context.kernel_addr + boot_context.kernel_len,
    ) {
        crate::serial_println!("[CHRONO] mem: ring3 demo range overlaps kernel image");
        panic!("ring3 demo range overlaps kernel");
    }

    let Some(physical_memory_offset) = boot_context.physical_memory_offset else {
        crate::serial_println!("[CHRONO] mem: physical memory offset missing");
        panic!("physical memory mapping missing");
    };
    let physical_memory_offset = VirtAddr::new(physical_memory_offset);
    let mut mapper = unsafe { init_offset_page_table(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(boot_context.memory_regions) };

    // Map user pages first so any newly-created parent page-table entries keep
    // the USER bit needed for ring 3 instruction fetches and stack accesses.
    map_user_demo_pages(&mut mapper, &mut frame_allocator);
    identity_map_kernel(boot_context, &mut mapper, &mut frame_allocator);
    map_heap(&mut mapper, &mut frame_allocator);
    ALLOCATOR.init(HEAP_START as usize, HEAP_SIZE as usize);
    leak_boot_allocation();

    unsafe {
        *MEMORY.0.get() = Some(MemoryState {
            physical_memory_offset,
            frame_allocator,
        });
    }

    crate::serial_println!(
        "[CHRONO] mem: heap initialized at {:#x} size 1MB",
        HEAP_START
    );
}

pub fn active_cr3() -> (PhysFrame<Size4KiB>, Cr3Flags) {
    Cr3::read()
}

pub unsafe fn switch_cr3(frame: PhysFrame<Size4KiB>, flags: Cr3Flags) {
    Cr3::write(frame, flags);
}

pub fn create_user_address_space() -> Option<AddressSpace> {
    with_memory_state(|state| {
        let frame = state.frame_allocator.allocate_frame()?;
        let active_table = unsafe { active_level_4_table(state.physical_memory_offset) };
        let new_table = unsafe {
            &mut *page_table_from_frame(state.physical_memory_offset, frame)
        };

        unsafe {
            core::ptr::copy_nonoverlapping(
                active_table as *const PageTable,
                new_table as *mut PageTable,
                1,
            );
        }
        clear_fixed_demo_pages(state, new_table)?;
        new_table[USER_ELF_PML4_INDEX].set_unused();

        Some(AddressSpace { pml4_frame: frame })
    })
    .flatten()
}

pub fn map_user_frame(
    address_space: AddressSpace,
    virtual_address: u64,
    flags: PageTableFlags,
) -> Option<PhysFrame<Size4KiB>> {
    with_memory_state(|state| {
        let page = Page::containing_address(VirtAddr::new(virtual_address));
        let frame = state.frame_allocator.allocate_frame()?;
        zero_frame_with_offset(state.physical_memory_offset, frame);

        let table = unsafe {
            &mut *page_table_from_frame(state.physical_memory_offset, address_space.pml4_frame)
        };
        let mut mapper = unsafe { OffsetPageTable::new(table, state.physical_memory_offset) };
        let flags = flags | PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;

        match unsafe { mapper.map_to(page, frame, flags, &mut state.frame_allocator) } {
            Ok(flush) => {
                flush.flush();
                Some(frame)
            }
            Err(MapToError::PageAlreadyMapped(_)) => None,
            Err(_) => None,
        }
    })
    .flatten()
}

pub fn zero_frame(frame: PhysFrame<Size4KiB>) {
    let _ = with_memory_state(|state| zero_frame_with_offset(state.physical_memory_offset, frame));
}

pub fn copy_to_frame(frame: PhysFrame<Size4KiB>, offset: usize, bytes: &[u8]) -> bool {
    if offset > PAGE_SIZE as usize || bytes.len() > PAGE_SIZE as usize - offset {
        return false;
    }

    with_memory_state(|state| {
        let destination = unsafe {
            frame_ptr(state.physical_memory_offset, frame)
                .add(offset)
        };
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len());
        }
        true
    })
    .unwrap_or(false)
}

pub fn stats() -> MemoryStats {
    MemoryStats {
        total_memory_bytes: TOTAL_MEMORY_BYTES.load(Ordering::Relaxed),
        heap_start: HEAP_START,
        heap_size_bytes: HEAP_SIZE,
        heap_used_bytes: ALLOCATOR.used() as u64,
    }
}

fn total_memory_bytes(memory_regions: &[MemoryRegion]) -> u64 {
    memory_regions
        .iter()
        .map(|region| region.end - region.start)
        .sum()
}

fn heap_range_is_usable(memory_regions: &[MemoryRegion]) -> bool {
    let heap_end = HEAP_START + HEAP_SIZE;

    memory_regions.iter().any(|region| {
        region.kind == MemoryRegionKind::Usable
            && region.start <= HEAP_START
            && region.end >= heap_end
    })
}

fn user_demo_range_is_usable(memory_regions: &[MemoryRegion]) -> bool {
    let user_demo_end = USER_STACK_START + USER_STACK_SIZE;

    memory_regions.iter().any(|region| {
        region.kind == MemoryRegionKind::Usable
            && region.start <= USER_CODE_START
            && region.end >= user_demo_end
    })
}

fn ranges_overlap(left_start: u64, left_end: u64, right_start: u64, right_end: u64) -> bool {
    left_start < right_end && right_start < left_end
}

fn map_user_demo_pages(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut BootInfoFrameAllocator,
) {
    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE;

    for address in [USER_CODE_START, USER_STACK_START] {
        let frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(PhysAddr::new(address));
        identity_map_frame(mapper, frame_allocator, frame, flags);
    }

    crate::serial_println!("[CHRONO] mem: ring3 demo pages mapped");
}

fn map_heap(mapper: &mut OffsetPageTable, frame_allocator: &mut BootInfoFrameAllocator) {
    let heap_start_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(HEAP_START));
    let heap_end_page: Page<Size4KiB> =
        Page::containing_address(VirtAddr::new(HEAP_START + HEAP_SIZE - 1));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        let frame: PhysFrame<Size4KiB> =
            PhysFrame::containing_address(PhysAddr::new(page.start_address().as_u64()));

        identity_map_frame(mapper, frame_allocator, frame, flags);
    }
}

fn identity_map_kernel(
    boot_context: &BootContext,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut BootInfoFrameAllocator,
) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let kernel_start = boot_context.kernel_addr;
    let kernel_end = boot_context.kernel_addr + boot_context.kernel_len;

    if kernel_end <= kernel_start {
        return;
    }

    let start_frame: PhysFrame<Size4KiB> =
        PhysFrame::containing_address(PhysAddr::new(kernel_start));
    let end_frame: PhysFrame<Size4KiB> =
        PhysFrame::containing_address(PhysAddr::new(kernel_end - 1));

    for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
        identity_map_frame(mapper, frame_allocator, frame, flags);
    }
}

fn identity_map_frame(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut BootInfoFrameAllocator,
    frame: PhysFrame<Size4KiB>,
    flags: PageTableFlags,
) {
    match unsafe { mapper.identity_map(frame, flags, frame_allocator) } {
        Ok(flush) => flush.flush(),
        Err(x86_64::structures::paging::mapper::MapToError::PageAlreadyMapped(_)) => {
            let page: Page<Size4KiB> =
                Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
            match unsafe { mapper.update_flags(page, flags) } {
                Ok(flush) => flush.flush(),
                Err(error) => panic!("identity map flag update failed: {:?}", error),
            }
        }
        Err(error) => panic!("identity map failed: {:?}", error),
    }
}

unsafe fn init_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);

    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let physical = level_4_table_frame.start_address();
    let virtual_address = physical_memory_offset + physical.as_u64();
    let page_table_pointer: *mut PageTable = virtual_address.as_mut_ptr();

    &mut *page_table_pointer
}

unsafe fn page_table_from_frame(
    physical_memory_offset: VirtAddr,
    frame: PhysFrame<Size4KiB>,
) -> *mut PageTable {
    (physical_memory_offset + frame.start_address().as_u64()).as_mut_ptr()
}

fn zero_frame_with_offset(physical_memory_offset: VirtAddr, frame: PhysFrame<Size4KiB>) {
    unsafe {
        core::ptr::write_bytes(frame_ptr(physical_memory_offset, frame), 0, PAGE_SIZE as usize);
    }
}

unsafe fn frame_ptr(physical_memory_offset: VirtAddr, frame: PhysFrame<Size4KiB>) -> *mut u8 {
    (physical_memory_offset + frame.start_address().as_u64()).as_mut_ptr()
}

fn with_memory_state<T>(f: impl FnOnce(&mut MemoryState) -> T) -> Option<T> {
    let state = unsafe { &mut *MEMORY.0.get() };
    state.as_mut().map(f)
}

fn clear_fixed_demo_pages(state: &mut MemoryState, pml4: &mut PageTable) -> Option<()> {
    let p3 = clone_child_table(state, pml4, page_table_index(USER_CODE_START, 39))?;
    let p2 = clone_child_table(state, p3, page_table_index(USER_CODE_START, 30))?;
    let p1 = clone_child_table(state, p2, page_table_index(USER_CODE_START, 21))?;

    p1[page_table_index(USER_CODE_START, 12)].set_unused();
    p1[page_table_index(USER_STACK_START, 12)].set_unused();
    Some(())
}

fn clone_child_table(
    state: &mut MemoryState,
    parent: &mut PageTable,
    index: usize,
) -> Option<&'static mut PageTable> {
    let flags = parent[index].flags();
    let old_frame = parent[index].frame().ok()?;
    let new_frame = state.frame_allocator.allocate_frame()?;
    let old_table = unsafe { page_table_from_frame(state.physical_memory_offset, old_frame) };
    let new_table = unsafe { page_table_from_frame(state.physical_memory_offset, new_frame) };

    unsafe {
        core::ptr::copy_nonoverlapping(old_table as *const PageTable, new_table, 1);
    }
    parent[index].set_frame(new_frame, flags);

    Some(unsafe { &mut *new_table })
}

fn page_table_index(address: u64, shift: u64) -> usize {
    ((address >> shift) & 0x1ff) as usize
}

fn leak_boot_allocation() {
    let layout = Layout::from_size_align(PAGE_SIZE as usize, PAGE_SIZE as usize)
        .expect("valid boot allocation layout");
    let pointer = unsafe { alloc(layout) };

    if pointer.is_null() {
        panic!("boot heap allocation failed");
    }
}

pub struct BootInfoFrameAllocator {
    memory_regions: &'static [MemoryRegion],
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn new(memory_regions: &'static [MemoryRegion]) -> Self {
        Self {
            memory_regions,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame<Size4KiB>> + '_ {
        self.memory_regions
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
            .filter(|region| align_up_u64(region.start, PAGE_SIZE) < region.end)
            .flat_map(|region| {
                let start_frame: PhysFrame<Size4KiB> =
                    PhysFrame::containing_address(PhysAddr::new(align_up_u64(
                        region.start,
                        PAGE_SIZE,
                    )));
                let end_frame: PhysFrame<Size4KiB> =
                    PhysFrame::containing_address(PhysAddr::new(region.end - 1));

                PhysFrame::range_inclusive(start_frame, end_frame)
            })
            .filter(|frame| !frame_is_heap_frame(*frame))
            .filter(|frame| !frame_is_user_demo_frame(*frame))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

fn frame_is_heap_frame(frame: PhysFrame<Size4KiB>) -> bool {
    let start = frame.start_address().as_u64();

    start >= HEAP_START && start < HEAP_START + HEAP_SIZE
}

fn frame_is_user_demo_frame(frame: PhysFrame<Size4KiB>) -> bool {
    let start = frame.start_address().as_u64();

    start == USER_CODE_START || start == USER_STACK_START
}

pub struct BumpAllocator {
    heap_start: AtomicUsize,
    heap_end: AtomicUsize,
    next: AtomicUsize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: AtomicUsize::new(0),
            heap_end: AtomicUsize::new(0),
            next: AtomicUsize::new(0),
        }
    }

    pub fn init(&self, heap_start: usize, heap_size: usize) {
        self.heap_start.store(heap_start, Ordering::SeqCst);
        self.heap_end.store(heap_start + heap_size, Ordering::SeqCst);
        self.next.store(heap_start, Ordering::SeqCst);
    }

    pub fn used(&self) -> usize {
        let heap_start = self.heap_start.load(Ordering::SeqCst);
        let next = self.next.load(Ordering::SeqCst);

        next.saturating_sub(heap_start)
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: CoreLayout) -> *mut u8 {
        let heap_end = self.heap_end.load(Ordering::SeqCst);

        if heap_end == 0 {
            return null_mut();
        }

        let mut current = self.next.load(Ordering::SeqCst);

        loop {
            let allocation_start = align_up(current, layout.align());
            let Some(allocation_end) = allocation_start.checked_add(layout.size()) else {
                return null_mut();
            };

            if allocation_end > heap_end {
                return null_mut();
            }

            match self.next.compare_exchange(
                current,
                allocation_end,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return allocation_start as *mut u8,
                Err(next_current) => current = next_current,
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: CoreLayout) {}
}

fn align_up(address: usize, alignment: usize) -> usize {
    let mask = alignment - 1;

    (address + mask) & !mask
}

fn align_up_u64(address: u64, alignment: u64) -> u64 {
    let mask = alignment - 1;

    (address + mask) & !mask
}

#[alloc_error_handler]
fn alloc_error_handler(layout: CoreLayout) -> ! {
    crate::serial_println!("[CHRONO] mem: allocation failed: {:?}", layout);
    panic!("allocation failed");
}
