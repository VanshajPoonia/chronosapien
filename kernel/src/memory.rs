//! First-pass physical memory, paging, and heap setup.
//!
//! Physical memory is managed in 4KiB frames because that is the default page
//! size used by x86_64 page tables. Chronosapian starts with identity-mapped heap
//! pages so the virtual address equals the physical address, which keeps early
//! memory behavior easy to inspect while the kernel is still small.

use alloc::alloc::{alloc, Layout};
use core::alloc::{GlobalAlloc, Layout as CoreLayout};
use core::cell::UnsafeCell;
use core::mem::{align_of, size_of};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

use crate::boot::{BootContext, MemoryRegion, MemoryRegionKind};

pub const SMP_BOOT_DATA_PHYS: u64 = 0x7000;
pub const SMP_TRAMPOLINE_PHYS: u64 = 0x8000;
pub const SMP_TRAMPOLINE_SIZE: u64 = PAGE_SIZE;
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
static ALLOCATOR: FreeListAllocator = FreeListAllocator::new();

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
    pub heap_free_bytes: u64,
    pub heap_largest_free_block_bytes: u64,
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

pub fn phys_to_mut(physical_address: u64) -> Option<*mut u8> {
    with_memory_state(|state| {
        (state.physical_memory_offset + physical_address).as_mut_ptr()
    })
}

pub fn identity_map_physical_range(start: u64, end: u64) -> bool {
    with_memory_state(|state| {
        let mut mapper = unsafe { init_offset_page_table(state.physical_memory_offset) };
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let start_frame: PhysFrame<Size4KiB> =
            PhysFrame::containing_address(PhysAddr::new(start));
        let end_frame: PhysFrame<Size4KiB> =
            PhysFrame::containing_address(PhysAddr::new(end.saturating_sub(1)));

        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            identity_map_frame(&mut mapper, &mut state.frame_allocator, frame, flags);
        }

        true
    })
    .unwrap_or(false)
}

pub fn stats() -> MemoryStats {
    let heap_free_bytes = ALLOCATOR.free_bytes() as u64;

    MemoryStats {
        total_memory_bytes: TOTAL_MEMORY_BYTES.load(Ordering::Relaxed),
        heap_start: HEAP_START,
        heap_size_bytes: HEAP_SIZE,
        heap_used_bytes: HEAP_SIZE.saturating_sub(heap_free_bytes),
        heap_free_bytes,
        heap_largest_free_block_bytes: ALLOCATOR.largest_free_block() as u64,
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
            .filter(|frame| !frame_is_smp_boot_frame(*frame))
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

fn frame_is_smp_boot_frame(frame: PhysFrame<Size4KiB>) -> bool {
    let start = frame.start_address().as_u64();

    start == SMP_BOOT_DATA_PHYS || start == SMP_TRAMPOLINE_PHYS
}

#[repr(C)]
struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

// Each allocation stores this tiny header immediately before the pointer handed
// to Rust. `dealloc` uses it to recover the original heap block, including any
// padding inserted to satisfy alignment.
#[repr(C)]
struct AllocHeader {
    block_start: usize,
    block_size: usize,
}

struct Allocation {
    payload_start: usize,
    header_start: usize,
    block_size: usize,
}

pub struct FreeListAllocator {
    locked: AtomicBool,
    heap_start: AtomicUsize,
    heap_end: AtomicUsize,
    // The head node is not a real free block. It is a permanent list anchor so
    // insert/remove code does not need special cases for the first heap block.
    free_list: UnsafeCell<FreeBlock>,
}

unsafe impl Sync for FreeListAllocator {}

impl FreeListAllocator {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            heap_start: AtomicUsize::new(0),
            heap_end: AtomicUsize::new(0),
            free_list: UnsafeCell::new(FreeBlock {
                size: 0,
                next: null_mut(),
            }),
        }
    }

    pub fn init(&self, heap_start: usize, heap_size: usize) {
        let heap_end = heap_start + heap_size;
        let heap_start = align_up(heap_start, align_of::<FreeBlock>());
        let heap_size = heap_end.saturating_sub(heap_start);

        self.lock();
        unsafe {
            // At boot the whole heap is one large free block. Later allocations
            // split this block and frees stitch adjacent blocks back together.
            let head = self.free_list.get();
            (*head).size = 0;
            (*head).next = null_mut();

            if heap_size >= size_of::<FreeBlock>() {
                let first = heap_start as *mut FreeBlock;
                first.write(FreeBlock {
                    size: heap_size,
                    next: null_mut(),
                });
                (*head).next = first;
            }
        }
        self.heap_start.store(heap_start, Ordering::SeqCst);
        self.heap_end.store(heap_end, Ordering::SeqCst);
        self.unlock();
    }

    pub fn free_bytes(&self) -> usize {
        self.lock();
        let total = unsafe {
            let mut total = 0usize;
            let mut current = (*self.free_list.get()).next;
            while !current.is_null() {
                total = total.saturating_add((*current).size);
                current = (*current).next;
            }
            total
        };
        self.unlock();
        total
    }

    pub fn largest_free_block(&self) -> usize {
        self.lock();
        let largest = unsafe {
            let mut largest = 0usize;
            let mut current = (*self.free_list.get()).next;
            while !current.is_null() {
                largest = largest.max((*current).size);
                current = (*current).next;
            }
            largest
        };
        self.unlock();
        largest
    }

    fn lock(&self) {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
    }

    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

unsafe impl GlobalAlloc for FreeListAllocator {
    unsafe fn alloc(&self, layout: CoreLayout) -> *mut u8 {
        if layout.size() == 0 || self.heap_end.load(Ordering::SeqCst) == 0 {
            return null_mut();
        }

        self.lock();
        let pointer = self.allocate_locked(layout);
        self.unlock();
        pointer
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: CoreLayout) {
        if ptr.is_null() || self.heap_end.load(Ordering::SeqCst) == 0 {
            return;
        }

        let header_start = (ptr as usize).saturating_sub(size_of::<AllocHeader>());
        let header = &*(header_start as *const AllocHeader);
        let block_start = header.block_start;
        let block_size = header.block_size;
        let heap_start = self.heap_start.load(Ordering::SeqCst);
        let heap_end = self.heap_end.load(Ordering::SeqCst);

        if block_start < heap_start
            || block_size < size_of::<FreeBlock>()
            || block_start.saturating_add(block_size) > heap_end
        {
            return;
        }

        self.lock();
        self.insert_free_block(block_start, block_size);
        self.unlock();
    }
}

impl FreeListAllocator {
    unsafe fn allocate_locked(&self, layout: CoreLayout) -> *mut u8 {
        let mut previous = self.free_list.get();
        let mut current = (*previous).next;

        while !current.is_null() {
            let block_start = current as usize;
            let block_size = (*current).size;

            if let Some(mut allocation) = allocation_from_block(block_start, block_size, layout) {
                let remaining = block_size - allocation.block_size;

                // First-fit allocation: take the first block that works. If a
                // useful tail remains, leave that tail on the free list.
                if remaining >= size_of::<FreeBlock>() {
                    let tail = (block_start + allocation.block_size) as *mut FreeBlock;
                    tail.write(FreeBlock {
                        size: remaining,
                        next: (*current).next,
                    });
                    (*previous).next = tail;
                } else {
                    allocation.block_size = block_size;
                    (*previous).next = (*current).next;
                }

                let header = allocation.header_start as *mut AllocHeader;
                header.write(AllocHeader {
                    block_start,
                    block_size: allocation.block_size,
                });

                return allocation.payload_start as *mut u8;
            }

            previous = current;
            current = (*current).next;
        }

        null_mut()
    }

    unsafe fn insert_free_block(&self, block_start: usize, block_size: usize) {
        let head = self.free_list.get();
        let mut previous = head;
        let mut current = (*previous).next;

        // Keep the list sorted by address. That makes coalescing simple: two
        // blocks are neighbors if one's end address equals the next start.
        while !current.is_null() && (current as usize) < block_start {
            previous = current;
            current = (*current).next;
        }

        let block = block_start as *mut FreeBlock;
        block.write(FreeBlock {
            size: block_size,
            next: current,
        });
        (*previous).next = block;
        self.coalesce_around(previous, block);
    }

    unsafe fn coalesce_around(&self, previous: *mut FreeBlock, block: *mut FreeBlock) {
        merge_with_next(block);

        let head = self.free_list.get();
        if previous != head && (previous as usize).saturating_add((*previous).size) == block as usize
        {
            (*previous).size = (*previous).size.saturating_add((*block).size);
            (*previous).next = (*block).next;
            merge_with_next(previous);
        }
    }
}

fn allocation_from_block(
    block_start: usize,
    block_size: usize,
    layout: CoreLayout,
) -> Option<Allocation> {
    let block_end = block_start.checked_add(block_size)?;
    let required_align = layout.align().max(align_of::<AllocHeader>());
    let payload_start = align_up(block_start.checked_add(size_of::<AllocHeader>())?, required_align);
    let header_start = payload_start.checked_sub(size_of::<AllocHeader>())?;
    let payload_end = payload_start.checked_add(layout.size())?;
    let block_end_after_alloc = align_up(payload_end, align_of::<FreeBlock>());

    if header_start < block_start || block_end_after_alloc > block_end {
        return None;
    }

    Some(Allocation {
        payload_start,
        header_start,
        block_size: block_end_after_alloc - block_start,
    })
}

unsafe fn merge_with_next(block: *mut FreeBlock) {
    while !(*block).next.is_null()
        && (block as usize).saturating_add((*block).size) == (*block).next as usize
    {
        let next = (*block).next;
        (*block).size = (*block).size.saturating_add((*next).size);
        (*block).next = (*next).next;
    }
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
