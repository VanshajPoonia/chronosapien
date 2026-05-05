//! ChronoOS boot handoff structures.
//!
//! The existing `bootloader` crate remains supported, but the kernel now has a
//! small local boot protocol that a custom loader can fill in too.

use core::cell::UnsafeCell;

use crate::framebuffer::{Framebuffer, FramebufferInfo, PixelFormat};

pub const CHRONO_BOOT_MAGIC: u64 = 0x5442_4f4e_4f52_4843;
pub const CHRONO_BOOT_VERSION: u32 = 1;

const MAX_BOOTLOADER_REGIONS: usize = 128;

#[derive(Clone, Copy)]
pub struct BootContext {
    pub framebuffer: Framebuffer,
    pub memory_regions: &'static [MemoryRegion],
    pub physical_memory_offset: Option<u64>,
    pub kernel_addr: u64,
    pub kernel_len: u64,
}

#[repr(C)]
pub struct ChronoBootInfo {
    pub magic: u64,
    pub version: u32,
    pub reserved: u32,
    pub framebuffer: ChronoFramebufferInfo,
    pub memory_regions: *const ChronoMemoryRegion,
    pub memory_region_count: u64,
    pub physical_memory_offset: u64,
    pub kernel_addr: u64,
    pub kernel_len: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ChronoFramebufferInfo {
    pub address: u64,
    pub byte_len: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub bytes_per_pixel: u32,
    pub pixel_format: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ChronoMemoryRegion {
    pub start: u64,
    pub end: u64,
    pub kind: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub kind: MemoryRegionKind,
}

impl MemoryRegion {
    const EMPTY: Self = Self {
        start: 0,
        end: 0,
        kind: MemoryRegionKind::Reserved,
    };
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionKind {
    Usable = 1,
    Reserved = 2,
}

struct BootloaderRegions(UnsafeCell<[MemoryRegion; MAX_BOOTLOADER_REGIONS]>);

unsafe impl Sync for BootloaderRegions {}

static BOOTLOADER_REGIONS: BootloaderRegions =
    BootloaderRegions(UnsafeCell::new([MemoryRegion::EMPTY; MAX_BOOTLOADER_REGIONS]));

pub fn context_from_bootloader(boot_info: &'static mut bootloader_api::BootInfo) -> BootContext {
    let region_count = copy_bootloader_regions(boot_info);
    let memory_regions = unsafe { &(*BOOTLOADER_REGIONS.0.get())[..region_count] };
    let framebuffer = boot_info
        .framebuffer
        .as_mut()
        .map(|framebuffer| {
            let info = framebuffer.info();
            Framebuffer {
                address: framebuffer.buffer_mut().as_mut_ptr(),
                info: FramebufferInfo {
                    byte_len: info.byte_len,
                    width: info.width,
                    height: info.height,
                    pixel_format: pixel_format_from_bootloader(info.pixel_format),
                    bytes_per_pixel: info.bytes_per_pixel,
                    stride: info.stride,
                },
            }
        })
        .unwrap_or_else(|| {
            crate::serial_println!("[CHRONO] fb: missing framebuffer");
            panic!("bootloader did not provide a framebuffer");
        });

    BootContext {
        framebuffer,
        memory_regions,
        physical_memory_offset: boot_info.physical_memory_offset.into_option(),
        kernel_addr: boot_info.kernel_addr,
        kernel_len: boot_info.kernel_len,
    }
}

pub unsafe fn context_from_custom(info: *const ChronoBootInfo) -> BootContext {
    let info = info.as_ref().expect("custom boot info pointer is null");

    if info.magic != CHRONO_BOOT_MAGIC || info.version != CHRONO_BOOT_VERSION {
        crate::serial_println!(
            "[CHRONO] custom bootloader: bad handoff magic={:#x} version={}",
            info.magic,
            info.version
        );
        panic!("invalid custom boot handoff");
    }

    let memory_regions = core::slice::from_raw_parts(
        info.memory_regions as *const MemoryRegion,
        info.memory_region_count as usize,
    );

    BootContext {
        framebuffer: Framebuffer {
            address: info.framebuffer.address as *mut u8,
            info: FramebufferInfo {
                byte_len: info.framebuffer.byte_len as usize,
                width: info.framebuffer.width as usize,
                height: info.framebuffer.height as usize,
                stride: info.framebuffer.stride as usize,
                bytes_per_pixel: info.framebuffer.bytes_per_pixel as usize,
                pixel_format: pixel_format_from_u32(info.framebuffer.pixel_format),
            },
        },
        memory_regions,
        physical_memory_offset: Some(info.physical_memory_offset),
        kernel_addr: info.kernel_addr,
        kernel_len: info.kernel_len,
    }
}

fn copy_bootloader_regions(boot_info: &'static bootloader_api::BootInfo) -> usize {
    let mut count = 0;

    for region in boot_info.memory_regions.iter() {
        if count >= MAX_BOOTLOADER_REGIONS {
            break;
        }

        unsafe {
            (*BOOTLOADER_REGIONS.0.get())[count] = MemoryRegion {
                start: region.start,
                end: region.end,
                kind: match region.kind {
                    bootloader_api::info::MemoryRegionKind::Usable => MemoryRegionKind::Usable,
                    _ => MemoryRegionKind::Reserved,
                },
            };
        }
        count += 1;
    }

    count
}

fn pixel_format_from_bootloader(format: bootloader_api::info::PixelFormat) -> PixelFormat {
    match format {
        bootloader_api::info::PixelFormat::Rgb => PixelFormat::Rgb,
        bootloader_api::info::PixelFormat::Bgr => PixelFormat::Bgr,
        bootloader_api::info::PixelFormat::U8 => PixelFormat::U8,
        _ => PixelFormat::Unknown,
    }
}

fn pixel_format_from_u32(format: u32) -> PixelFormat {
    match format {
        1 => PixelFormat::Rgb,
        2 => PixelFormat::Bgr,
        3 => PixelFormat::U8,
        _ => PixelFormat::Unknown,
    }
}
