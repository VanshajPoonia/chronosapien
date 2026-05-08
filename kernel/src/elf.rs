//! Tiny ELF64 parser for static user executables.

use alloc::vec::Vec;

use crate::memory;

pub const PT_LOAD: u32 = 1;
pub const PF_X: u32 = 1;
pub const PF_W: u32 = 2;

const EI_CLASS_64: u8 = 2;
const EI_DATA_LE: u8 = 1;
const EV_CURRENT: u32 = 1;
const ET_EXEC: u16 = 2;
const EM_X86_64: u16 = 62;
const ELF_HEADER_SIZE: u16 = 64;
const PROGRAM_HEADER_SIZE: u16 = 56;
const MAX_PROGRAM_HEADERS: u16 = 16;

#[derive(Clone, Copy)]
pub struct ProgramSegment {
    pub offset: u64,
    pub virtual_address: u64,
    pub file_size: u64,
    pub memory_size: u64,
    pub flags: u32,
}

pub struct ElfImage {
    pub entry: u64,
    pub segments: Vec<ProgramSegment>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElfError {
    TooSmall,
    BadMagic,
    Unsupported,
    Malformed,
}

pub fn parse(bytes: &[u8]) -> Result<ElfImage, ElfError> {
    if bytes.len() < ELF_HEADER_SIZE as usize {
        return Err(ElfError::TooSmall);
    }
    if &bytes[..4] != b"\x7FELF" {
        return Err(ElfError::BadMagic);
    }
    if bytes[4] != EI_CLASS_64 || bytes[5] != EI_DATA_LE || bytes[6] != EV_CURRENT as u8 {
        return Err(ElfError::Unsupported);
    }
    if read_u16(bytes, 16) != ET_EXEC
        || read_u16(bytes, 18) != EM_X86_64
        || read_u32(bytes, 20) != EV_CURRENT
        || read_u16(bytes, 52) != ELF_HEADER_SIZE
        || read_u16(bytes, 54) != PROGRAM_HEADER_SIZE
    {
        return Err(ElfError::Unsupported);
    }

    let entry = read_u64(bytes, 24);
    let phoff = read_u64(bytes, 32) as usize;
    let phentsize = read_u16(bytes, 54) as usize;
    let phnum = read_u16(bytes, 56);
    if phnum == 0 || phnum > MAX_PROGRAM_HEADERS {
        return Err(ElfError::Malformed);
    }

    let ph_size = phentsize
        .checked_mul(phnum as usize)
        .ok_or(ElfError::Malformed)?;
    let ph_end = phoff.checked_add(ph_size).ok_or(ElfError::Malformed)?;
    if ph_end > bytes.len() {
        return Err(ElfError::Malformed);
    }

    let mut segments = Vec::new();
    let mut entry_is_executable = false;

    for index in 0..phnum as usize {
        let start = phoff + index * phentsize;
        let header = &bytes[start..start + phentsize];
        let segment_type = read_u32(header, 0);
        if segment_type != PT_LOAD {
            continue;
        }

        let flags = read_u32(header, 4);
        let offset = read_u64(header, 8);
        let virtual_address = read_u64(header, 16);
        let file_size = read_u64(header, 32);
        let memory_size = read_u64(header, 40);
        let align = read_u64(header, 48);

        validate_load_segment(bytes.len(), offset, virtual_address, file_size, memory_size, align)?;

        let segment_end = virtual_address
            .checked_add(memory_size)
            .ok_or(ElfError::Malformed)?;
        if flags & PF_X != 0 && entry >= virtual_address && entry < segment_end {
            entry_is_executable = true;
        }

        segments.push(ProgramSegment {
            offset,
            virtual_address,
            file_size,
            memory_size,
            flags,
        });
    }

    if segments.is_empty() || !entry_is_executable {
        return Err(ElfError::Malformed);
    }

    Ok(ElfImage { entry, segments })
}

fn validate_load_segment(
    file_len: usize,
    offset: u64,
    virtual_address: u64,
    file_size: u64,
    memory_size: u64,
    align: u64,
) -> Result<(), ElfError> {
    if memory_size == 0 || memory_size < file_size {
        return Err(ElfError::Malformed);
    }

    let file_end = offset.checked_add(file_size).ok_or(ElfError::Malformed)?;
    if file_end as usize > file_len {
        return Err(ElfError::Malformed);
    }

    let memory_end = virtual_address
        .checked_add(memory_size)
        .ok_or(ElfError::Malformed)?;
    if virtual_address < memory::USER_ELF_BASE || memory_end > memory::USER_ELF_LIMIT {
        return Err(ElfError::Unsupported);
    }
    if align > 1 && offset % align != virtual_address % align {
        return Err(ElfError::Malformed);
    }

    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
}
