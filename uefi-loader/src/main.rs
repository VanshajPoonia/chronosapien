#![no_std]
#![no_main]

extern crate alloc;

use core::arch::asm;
use core::fmt::Write;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr;
use uefi::boot::{self, AllocateType, MemoryMap, MemoryType};
use uefi::fs::FileSystem;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::table::cfg::ConfigTableEntry;

const KERNEL_PATH: &str = "\\CHRONO\\KERNEL.ELF";
const CHRONO_BOOT_MAGIC: u64 = 0x5442_4f4e_4f52_4843;
const CHRONO_BOOT_VERSION: u32 = 2;
const MAX_MEMORY_REGIONS: usize = 256;
const PAGE_SIZE: u64 = 4096;
const TWO_MIB: u64 = 2 * 1024 * 1024;
const IDENTITY_BASE_BYTES: u64 = 64 * 1024 * 1024 * 1024;
const PTE_PRESENT: u64 = 1 << 0;
const PTE_WRITABLE: u64 = 1 << 1;
const PTE_HUGE: u64 = 1 << 7;
const COM1_PORT: u16 = 0x3F8;

#[repr(C)]
#[derive(Clone, Copy)]
struct ChronoBootInfo {
    magic: u64,
    version: u32,
    reserved: u32,
    framebuffer: ChronoFramebufferInfo,
    memory_regions: *const ChronoMemoryRegion,
    memory_region_count: u64,
    physical_memory_offset: u64,
    kernel_addr: u64,
    kernel_len: u64,
    rsdp_addr: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ChronoFramebufferInfo {
    address: u64,
    byte_len: u64,
    width: u32,
    height: u32,
    stride: u32,
    bytes_per_pixel: u32,
    pixel_format: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ChronoMemoryRegion {
    start: u64,
    end: u64,
    kind: u32,
}

#[derive(Clone, Copy)]
struct LoadedKernel {
    entry: u64,
    start: u64,
    end: u64,
}

#[derive(Clone, Copy)]
struct Framebuffer {
    address: u64,
    byte_len: u64,
    width: u32,
    height: u32,
    stride: u32,
    bytes_per_pixel: u32,
    pixel_format: u32,
}

#[entry]
fn main() -> Status {
    if let Err(status) = uefi_main() {
        serial_println("[CHRONO] uefi: boot failed");
        return status;
    }

    Status::SUCCESS
}

fn uefi_main() -> Result<(), Status> {
    uefi::helpers::init().map_err(|_| Status::LOAD_ERROR)?;
    serial_init();
    uefi::println!("[CHRONO] uefi: loader start");
    serial_println("[CHRONO] uefi: loader start");

    let kernel = {
        let image = boot::image_handle();
        let mut fs =
            FileSystem::new(boot::get_image_file_system(image).map_err(|_| Status::NOT_FOUND)?);
        fs.read(KERNEL_PATH).map_err(|_| Status::NOT_FOUND)?
    };

    let loaded = load_kernel(&kernel)?;
    let framebuffer = setup_gop()?;
    uefi::println!(
        "[CHRONO] uefi: framebuffer at 0x{:x}",
        framebuffer.address
    );
    serial_print_hex("[CHRONO] uefi: framebuffer at 0x", framebuffer.address);

    let rsdp_addr = find_rsdp();
    let memory_regions_storage = allocate_pages_for::<ChronoMemoryRegion>(MAX_MEMORY_REGIONS)?;
    let boot_info = allocate_pages_for::<ChronoBootInfo>(1)?;
    let stack_pointer = current_stack_pointer();

    let page_table = build_page_tables(&[
        (0, IDENTITY_BASE_BYTES),
        (loaded.start, loaded.end),
        (framebuffer.address, framebuffer.address + framebuffer.byte_len),
        (
            stack_pointer.saturating_sub(TWO_MIB),
            stack_pointer.saturating_add(TWO_MIB),
        ),
        (
            memory_regions_storage as u64,
            memory_regions_storage as u64
                + (MAX_MEMORY_REGIONS * size_of::<ChronoMemoryRegion>()) as u64,
        ),
        (boot_info as u64, boot_info as u64 + size_of::<ChronoBootInfo>() as u64),
    ])?;

    core::mem::forget(kernel);
    uefi::println!("[CHRONO] uefi: handoff ok");
    let memory_map = unsafe { boot::exit_boot_services(Some(MemoryType::LOADER_DATA)) };
    let memory_region_count =
        copy_memory_map(&memory_map, memory_regions_storage, MAX_MEMORY_REGIONS);

    unsafe {
        ptr::write(
            boot_info,
            ChronoBootInfo {
                magic: CHRONO_BOOT_MAGIC,
                version: CHRONO_BOOT_VERSION,
                reserved: 0,
                framebuffer: ChronoFramebufferInfo {
                    address: framebuffer.address,
                    byte_len: framebuffer.byte_len,
                    width: framebuffer.width,
                    height: framebuffer.height,
                    stride: framebuffer.stride,
                    bytes_per_pixel: framebuffer.bytes_per_pixel,
                    pixel_format: framebuffer.pixel_format,
                },
                memory_regions: memory_regions_storage,
                memory_region_count: memory_region_count as u64,
                physical_memory_offset: 0,
                kernel_addr: loaded.start,
                kernel_len: loaded.end.saturating_sub(loaded.start),
                rsdp_addr,
            },
        );
    }

    serial_println("[CHRONO] uefi: handoff ok");
    unsafe {
        jump_to_kernel(page_table, loaded.entry, boot_info as u64);
    }
}

fn load_kernel(bytes: &[u8]) -> Result<LoadedKernel, Status> {
    if bytes.get(0..4) != Some(b"\x7fELF") || bytes.get(4) != Some(&2) {
        return Err(Status::LOAD_ERROR);
    }

    let entry = read_u64(bytes, 24);
    let phoff = read_u64(bytes, 32) as usize;
    let phentsize = read_u16(bytes, 54) as usize;
    let phnum = read_u16(bytes, 56) as usize;
    let mut kernel_start = u64::MAX;
    let mut kernel_end = 0u64;

    for index in 0..phnum {
        let offset = phoff + index * phentsize;
        if read_u32(bytes, offset) != 1 {
            continue;
        }

        let file_offset = read_u64(bytes, offset + 8) as usize;
        let physical = read_u64(bytes, offset + 24);
        let file_size = read_u64(bytes, offset + 32) as usize;
        let memory_size = read_u64(bytes, offset + 40) as usize;
        let page_start = align_down(physical, PAGE_SIZE);
        let page_end = align_up(physical + memory_size as u64, PAGE_SIZE);

        allocate_at(page_start, page_end - page_start)?;
        unsafe {
            ptr::write_bytes(physical as *mut u8, 0, memory_size);
            ptr::copy_nonoverlapping(
                bytes[file_offset..file_offset + file_size].as_ptr(),
                physical as *mut u8,
                file_size,
            );
        }

        kernel_start = kernel_start.min(page_start);
        kernel_end = kernel_end.max(page_end);
    }

    if kernel_start == u64::MAX {
        return Err(Status::LOAD_ERROR);
    }

    Ok(LoadedKernel {
        entry,
        start: kernel_start,
        end: kernel_end,
    })
}

fn setup_gop() -> Result<Framebuffer, Status> {
    let handle = boot::get_handle_for_protocol::<GraphicsOutput>().map_err(|_| Status::NOT_FOUND)?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(handle)
        .map_err(|_| Status::NOT_FOUND)?;

    let preferred = gop
        .modes()
        .filter(|mode| mode.info().pixel_format() != PixelFormat::BltOnly)
        .find(|mode| mode.info().resolution() == (1024, 768));
    if let Some(mode) = preferred {
        gop.set_mode(&mode).map_err(|_| Status::DEVICE_ERROR)?;
    }

    let info = gop.current_mode_info();
    let (width, height) = info.resolution();
    let mut fb = gop.frame_buffer();
    let format = match info.pixel_format() {
        PixelFormat::Rgb => 1,
        PixelFormat::Bgr => 2,
        _ => return Err(Status::UNSUPPORTED),
    };

    Ok(Framebuffer {
        address: fb.as_mut_ptr() as u64,
        byte_len: fb.size() as u64,
        width: width as u32,
        height: height as u32,
        stride: info.stride() as u32,
        bytes_per_pixel: 4,
        pixel_format: format,
    })
}

fn find_rsdp() -> u64 {
    uefi::system::with_config_table(|entries| {
        for entry in entries {
            if entry.guid == ConfigTableEntry::ACPI2_GUID
                || entry.guid == ConfigTableEntry::ACPI_GUID
            {
                return entry.address as u64;
            }
        }

        0
    })
}

fn copy_memory_map(
    memory_map: &impl MemoryMap,
    destination: *mut ChronoMemoryRegion,
    capacity: usize,
) -> usize {
    let mut count = 0usize;
    for descriptor in memory_map.entries() {
        if count >= capacity {
            break;
        }

        let kind = match descriptor.ty {
            MemoryType::CONVENTIONAL => 1,
            _ => 2,
        };
        unsafe {
            destination.add(count).write(ChronoMemoryRegion {
                start: descriptor.phys_start,
                end: descriptor.phys_start + descriptor.page_count * PAGE_SIZE,
                kind,
            });
        }
        count += 1;
    }

    count
}

fn build_page_tables(ranges: &[(u64, u64)]) -> Result<u64, Status> {
    let pml4 = allocate_zeroed_page()?;
    let pdpt = allocate_zeroed_page()?;
    let mut page_directories = [0u64; 512];

    unsafe {
        write_entry(pml4, 0, pdpt | PTE_PRESENT | PTE_WRITABLE);
    }

    for &(start, end) in ranges {
        let mut address = align_down(start, TWO_MIB);
        let end = align_up(end, TWO_MIB);
        while address < end {
            let pml4_index = ((address >> 39) & 0x1ff) as usize;
            if pml4_index != 0 {
                return Err(Status::UNSUPPORTED);
            }

            let pdpt_index = ((address >> 30) & 0x1ff) as usize;
            let pd_index = ((address >> 21) & 0x1ff) as usize;
            if page_directories[pdpt_index] == 0 {
                let pd = allocate_zeroed_page()?;
                page_directories[pdpt_index] = pd;
                unsafe {
                    write_entry(pdpt, pdpt_index, pd | PTE_PRESENT | PTE_WRITABLE);
                }
            }

            unsafe {
                write_entry(
                    page_directories[pdpt_index],
                    pd_index,
                    address | PTE_PRESENT | PTE_WRITABLE | PTE_HUGE,
                );
            }
            address += TWO_MIB;
        }
    }

    Ok(pml4)
}

unsafe fn jump_to_kernel(pml4: u64, entry: u64, boot_info: u64) -> ! {
    asm!("mov cr3, {}", in(reg) pml4, options(nostack, preserves_flags));
    let kernel_entry: extern "sysv64" fn(u64) -> ! = core::mem::transmute(entry as usize);
    kernel_entry(boot_info)
}

fn current_stack_pointer() -> u64 {
    let stack_pointer: u64;
    unsafe {
        asm!("mov {}, rsp", out(reg) stack_pointer, options(nomem, nostack, preserves_flags));
    }
    stack_pointer
}

fn allocate_pages_for<T>(count: usize) -> Result<*mut T, Status> {
    let bytes = (count * size_of::<T>()) as u64;
    let pages = align_up(bytes, PAGE_SIZE) / PAGE_SIZE;
    let address = boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        pages as usize,
    )
    .map_err(|_| Status::OUT_OF_RESOURCES)?;
    Ok(address.as_ptr() as *mut T)
}

fn allocate_zeroed_page() -> Result<u64, Status> {
    let address = boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
        .map_err(|_| Status::OUT_OF_RESOURCES)?;
    unsafe {
        ptr::write_bytes(address.as_ptr(), 0, PAGE_SIZE as usize);
    }
    Ok(address.as_ptr() as u64)
}

fn allocate_at(address: u64, bytes: u64) -> Result<(), Status> {
    let pages = (align_up(bytes, PAGE_SIZE) / PAGE_SIZE) as usize;
    boot::allocate_pages(
        AllocateType::Address(address),
        MemoryType::LOADER_DATA,
        pages,
    )
    .map(|_| ())
    .map_err(|_| Status::OUT_OF_RESOURCES)
}

unsafe fn write_entry(table: u64, index: usize, value: u64) {
    ((table as *mut u64).add(index)).write(value);
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn align_down(value: u64, align: u64) -> u64 {
    value & !(align - 1)
}

fn align_up(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}

fn serial_init() {
    unsafe {
        outb(COM1_PORT + 1, 0x00);
        outb(COM1_PORT + 3, 0x80);
        outb(COM1_PORT, 0x03);
        outb(COM1_PORT + 1, 0x00);
        outb(COM1_PORT + 3, 0x03);
        outb(COM1_PORT + 2, 0xC7);
        outb(COM1_PORT + 4, 0x0B);
    }
}

fn serial_println(message: &str) {
    for byte in message.bytes() {
        serial_byte(byte);
    }
    serial_byte(b'\n');
}

fn serial_print_hex(prefix: &str, value: u64) {
    let mut buffer = ArrayString::<64>::new();
    let _ = write!(&mut buffer, "{}{:x}", prefix, value);
    serial_println(buffer.as_str());
}

fn serial_byte(byte: u8) {
    unsafe {
        while inb(COM1_PORT + 5) & 0x20 == 0 {}
        if byte == b'\n' {
            outb(COM1_PORT, b'\r');
        }
        outb(COM1_PORT, byte);
    }
}

unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println("[CHRONO] uefi: panic");
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

struct ArrayString<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> ArrayString<N> {
    const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }
}

impl<const N: usize> Write for ArrayString<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            if self.len >= N {
                return Err(core::fmt::Error);
            }
            self.bytes[self.len] = byte;
            self.len += 1;
        }
        Ok(())
    }
}
