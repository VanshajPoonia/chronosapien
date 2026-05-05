#![no_std]

//! Protected-mode helper definitions for the ChronoOS Stage 2 loader.
//!
//! The current Stage 2 assembly owns the BIOS calls and mode switch. This file
//! documents the Rust-side layout used by the image builder and gives the next
//! implementation step a small, typed surface for manifest validation and boot
//! info construction.

pub const MANIFEST_MAGIC: &[u8; 8] = b"CHRONO2M";
pub const CHRONO_BOOT_MAGIC: u64 = 0x5442_4f4e_4f52_4843;
pub const CHRONO_BOOT_VERSION: u32 = 1;
pub const MAX_SEGMENTS: usize = 8;

#[repr(C)]
pub struct Stage2Manifest {
    pub magic: [u8; 8],
    pub kernel_entry: u64,
    pub kernel_addr: u64,
    pub kernel_len: u64,
    pub segment_count: u16,
    pub reserved: [u8; 6],
    pub segments: [Stage2Segment; MAX_SEGMENTS],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Stage2Segment {
    pub lba: u64,
    pub sector_count: u16,
    pub reserved: [u8; 6],
    pub destination: u64,
    pub byte_len: u64,
}

#[repr(C)]
pub struct ChronoBootInfo {
    pub magic: u64,
    pub version: u32,
    pub reserved: u32,
    pub framebuffer: ChronoFramebufferInfo,
    pub memory_regions: u64,
    pub memory_region_count: u64,
    pub physical_memory_offset: u64,
    pub kernel_addr: u64,
    pub kernel_len: u64,
}

#[repr(C)]
pub struct ChronoFramebufferInfo {
    pub address: u64,
    pub byte_len: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub bytes_per_pixel: u32,
    pub pixel_format: u32,
}

pub fn manifest_is_valid(manifest: &Stage2Manifest) -> bool {
    &manifest.magic == MANIFEST_MAGIC && manifest.segment_count as usize <= MAX_SEGMENTS
}
