//! First-pass physical memory, paging, and heap setup.
//!
//! Physical memory is managed in 4KiB frames because that is the default page
//! size used by x86_64 page tables. Chronosapian starts with identity-mapped heap
//! pages so the virtual address equals the physical address, which keeps early
//! memory behavior easy to inspect while the kernel is still small.

use alloc::alloc::{alloc, Layout};
use core::alloc::{GlobalAlloc, Layout as CoreLayout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

use crate::boot::{BootContext, MemoryRegion, MemoryRegionKind};

pub const HEAP_START: u64 = 0x200000;
pub const HEAP_SIZE: u64 = 1024 * 1024;

const PAGE_SIZE: u64 = 4096;

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

static TOTAL_MEMORY_BYTES: AtomicU64 = AtomicU64::new(0);

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

    let Some(physical_memory_offset) = boot_context.physical_memory_offset else {
        crate::serial_println!("[CHRONO] mem: physical memory offset missing");
        panic!("physical memory mapping missing");
    };
    let physical_memory_offset = VirtAddr::new(physical_memory_offset);
    let mut mapper = unsafe { init_offset_page_table(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(boot_context.memory_regions) };

    identity_map_kernel(boot_context, &mut mapper, &mut frame_allocator);
    map_heap(&mut mapper, &mut frame_allocator);
    ALLOCATOR.init(HEAP_START as usize, HEAP_SIZE as usize);
    leak_boot_allocation();

    crate::serial_println!(
        "[CHRONO] mem: heap initialized at {:#x} size 1MB",
        HEAP_START
    );
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
    boot_info: &bootloader_api::BootInfo,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut BootInfoFrameAllocator,
) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let kernel_start = boot_info.kernel_addr;
    let kernel_end = boot_info.kernel_addr + boot_info.kernel_len;

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
        Err(x86_64::structures::paging::mapper::MapToError::PageAlreadyMapped(_)) => {}
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

fn leak_boot_allocation() {
    let layout = Layout::from_size_align(PAGE_SIZE as usize, PAGE_SIZE as usize)
        .expect("valid boot allocation layout");
    let pointer = unsafe { alloc(layout) };

    if pointer.is_null() {
        panic!("boot heap allocation failed");
    }
}

pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn new(memory_regions: &'static MemoryRegions) -> Self {
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
