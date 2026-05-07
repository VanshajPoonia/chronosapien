//! Tiny filesystem facade backed by ChronoFS when the ATA data disk is present.

use alloc::string::String;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

use crate::ata::{self, SECTOR_SIZE};

pub const MAX_FILENAME_LEN: usize = 32;

const MAGIC: [u8; 8] = *b"CHRONFS1";
const VERSION: u16 = 1;
const TOTAL_SECTORS: u32 = 32_768;
const MAX_FILES: usize = 64;
const MAX_FILE_BYTES: usize = 64 * 1024;
const FILE_ENTRY_SIZE: usize = 64;
const FILE_TABLE_START: u32 = 1;
const FILE_TABLE_SECTORS: u32 = 8;
const BITMAP_START: u32 = FILE_TABLE_START + FILE_TABLE_SECTORS;
const BITMAP_SECTORS: u32 = 8;
const DATA_START: u32 = BITMAP_START + BITMAP_SECTORS;
const BITMAP_BYTES: usize = BITMAP_SECTORS as usize * SECTOR_SIZE;
const ENTRIES_PER_SECTOR: usize = SECTOR_SIZE / FILE_ENTRY_SIZE;

#[derive(Clone)]
struct File {
    name: String,
    content: Vec<u8>,
}

struct FsState {
    files: Vec<File>,
    disk: Option<DiskState>,
}

impl FsState {
    const fn new() -> Self {
        Self {
            files: Vec::new(),
            disk: None,
        }
    }
}

struct GlobalFs(UnsafeCell<FsState>);

unsafe impl Sync for GlobalFs {}

static FS: GlobalFs = GlobalFs(UnsafeCell::new(FsState::new()));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FsError {
    EmptyName,
    InvalidName,
    NameTooLong,
    NotFound,
    FileTooLarge,
    NoSpace,
    Disk,
}

#[derive(Clone, Copy)]
struct DiskEntry {
    used: bool,
    name_len: u8,
    name: [u8; MAX_FILENAME_LEN],
    size: u32,
    start_sector: u32,
    sector_count: u32,
}

impl DiskEntry {
    const fn empty() -> Self {
        Self {
            used: false,
            name_len: 0,
            name: [0; MAX_FILENAME_LEN],
            size: 0,
            start_sector: 0,
            sector_count: 0,
        }
    }

    fn name_str(&self) -> Option<&str> {
        if !self.used || self.name_len as usize > MAX_FILENAME_LEN {
            return None;
        }

        core::str::from_utf8(&self.name[..self.name_len as usize]).ok()
    }

    fn matches(&self, name: &str) -> bool {
        self.name_str() == Some(name)
    }

    fn set(&mut self, name: &str, size: usize, start_sector: u32, sector_count: u32) {
        self.used = true;
        self.name_len = name.len() as u8;
        self.name = [0; MAX_FILENAME_LEN];
        self.name[..name.len()].copy_from_slice(name.as_bytes());
        self.size = size as u32;
        self.start_sector = start_sector;
        self.sector_count = sector_count;
    }
}

struct DiskState {
    entries: [DiskEntry; MAX_FILES],
    bitmap: Vec<u8>,
    file_count: u32,
}

impl DiskState {
    fn mount_or_format() -> Result<(Self, Vec<File>), FsError> {
        let mut sector = [0u8; SECTOR_SIZE];
        if ata::read_sector(0, &mut sector).is_err() {
            return Err(FsError::Disk);
        }

        if read_superblock(&sector).is_err() {
            crate::serial_println!("[CHRONO] fs: formatting ChronoFS data disk");
            format_disk()?;
        }

        let mut superblock = [0u8; SECTOR_SIZE];
        ata::read_sector(0, &mut superblock).map_err(|_| FsError::Disk)?;
        let superblock_file_count = read_superblock(&superblock)?;
        let bitmap = read_bitmap()?;
        let entries = read_file_table()?;
        let files = read_files(&entries)?;
        let file_count = files.len() as u32;

        if file_count != superblock_file_count {
            crate::serial_println!(
                "[CHRONO] fs: mounted ChronoFS files={} superblock={}",
                file_count,
                superblock_file_count
            );
        } else {
            crate::serial_println!("[CHRONO] fs: mounted ChronoFS files={}", file_count);
        }

        Ok((
            Self {
                entries,
                bitmap,
                file_count,
            },
            files,
        ))
    }

    fn write_file(
        &mut self,
        name: &str,
        content: &[u8],
        files: &mut Vec<File>,
    ) -> Result<(), FsError> {
        if content.len() > MAX_FILE_BYTES {
            return Err(FsError::FileTooLarge);
        }

        let sector_count = sectors_for(content.len());
        let existing_index = self.entries.iter().position(|entry| entry.matches(name));
        let entry_index = match existing_index {
            Some(index) => index,
            None => self
                .entries
                .iter()
                .position(|entry| !entry.used)
                .ok_or(FsError::NoSpace)?,
        };

        let old_entry = self.entries[entry_index];
        let same_location = old_entry.used && old_entry.sector_count >= sector_count;
        let start_sector = if same_location {
            old_entry.start_sector
        } else {
            self.find_free_run(sector_count).ok_or(FsError::NoSpace)?
        };

        write_content(start_sector, sector_count, content)?;

        if same_location && old_entry.sector_count > sector_count {
            self.mark_range(
                old_entry.start_sector + sector_count,
                old_entry.sector_count - sector_count,
                false,
            );
        }
        if !same_location && old_entry.used {
            self.mark_range(old_entry.start_sector, old_entry.sector_count, false);
        }
        if !same_location {
            self.mark_range(start_sector, sector_count, true);
        }

        self.entries[entry_index].set(name, content.len(), start_sector, sector_count);

        if existing_index.is_none() {
            self.file_count += 1;
        }

        self.sync_bitmap()?;
        self.sync_entry_sector(entry_index)?;
        self.sync_superblock()?;
        upsert_cache(files, name, content);

        crate::serial_println!("[CHRONO] fs: write {}", name);
        Ok(())
    }

    fn remove_file(&mut self, name: &str, files: &mut Vec<File>) -> Result<(), FsError> {
        let Some(entry_index) = self.entries.iter().position(|entry| entry.matches(name)) else {
            return Err(FsError::NotFound);
        };

        let old_entry = self.entries[entry_index];
        self.mark_range(old_entry.start_sector, old_entry.sector_count, false);
        self.entries[entry_index] = DiskEntry::empty();
        self.file_count = self.file_count.saturating_sub(1);

        self.sync_bitmap()?;
        self.sync_entry_sector(entry_index)?;
        self.sync_superblock()?;

        if let Some(index) = files.iter().position(|file| file.name == name) {
            files.remove(index);
        }

        crate::serial_println!("[CHRONO] fs: rm {}", name);
        Ok(())
    }

    fn find_free_run(&self, sector_count: u32) -> Option<u32> {
        if sector_count == 0 {
            return Some(DATA_START);
        }

        let mut run_start = DATA_START;
        let mut run_len = 0u32;

        for sector in DATA_START..TOTAL_SECTORS {
            if self.is_sector_used(sector) {
                run_start = sector + 1;
                run_len = 0;
                continue;
            }

            run_len += 1;
            if run_len == sector_count {
                return Some(run_start);
            }
        }

        None
    }

    fn is_sector_used(&self, sector: u32) -> bool {
        bitmap_get(&self.bitmap, sector)
    }

    fn mark_range(&mut self, start: u32, count: u32, used: bool) {
        for sector in start..start + count {
            bitmap_set(&mut self.bitmap, sector, used);
        }
    }

    fn sync_superblock(&self) -> Result<(), FsError> {
        let mut sector = [0u8; SECTOR_SIZE];
        write_superblock(&mut sector, self.file_count);
        ata::write_sector(0, &sector).map_err(|_| FsError::Disk)
    }

    fn sync_bitmap(&self) -> Result<(), FsError> {
        for index in 0..BITMAP_SECTORS as usize {
            let mut sector = [0u8; SECTOR_SIZE];
            let start = index * SECTOR_SIZE;
            sector.copy_from_slice(&self.bitmap[start..start + SECTOR_SIZE]);
            ata::write_sector(BITMAP_START + index as u32, &sector).map_err(|_| FsError::Disk)?;
        }

        Ok(())
    }

    fn sync_entry_sector(&self, entry_index: usize) -> Result<(), FsError> {
        let sector_index = entry_index / ENTRIES_PER_SECTOR;
        let mut sector = [0u8; SECTOR_SIZE];
        let first_entry = sector_index * ENTRIES_PER_SECTOR;

        for index in 0..ENTRIES_PER_SECTOR {
            let Some(entry) = self.entries.get(first_entry + index) else {
                break;
            };
            write_entry(
                entry,
                &mut sector[index * FILE_ENTRY_SIZE..(index + 1) * FILE_ENTRY_SIZE],
            );
        }

        ata::write_sector(FILE_TABLE_START + sector_index as u32, &sector).map_err(|_| FsError::Disk)
    }
}

/// Mounts ChronoFS from the ATA data disk, or keeps the heap fallback if it fails.
pub fn init() {
    let state = unsafe { &mut *FS.0.get() };

    match DiskState::mount_or_format() {
        Ok((disk, files)) => {
            state.files = files;
            state.disk = Some(disk);
        }
        Err(error) => {
            state.files.clear();
            state.disk = None;
            crate::serial_println!("[CHRONO] fs: disk unavailable ({:?}); using heap", error);
        }
    }
}

/// Visits files in the mounted table/cache order. Returns false when empty.
pub fn list(mut visit: impl FnMut(&str)) -> bool {
    let state: &'static FsState = unsafe { &*FS.0.get() };

    for file in &state.files {
        visit(&file.name);
    }

    !state.files.is_empty()
}

/// Returns a borrowed UTF-8 file body from the global filesystem cache.
pub fn read(name: &str) -> Result<&'static str, FsError> {
    let bytes = read_bytes(name)?;

    core::str::from_utf8(bytes).map_err(|_| FsError::Disk)
}

/// Returns a borrowed binary file body from the global filesystem cache.
pub fn read_bytes(name: &str) -> Result<&'static [u8], FsError> {
    validate_name(name)?;

    let state: &'static FsState = unsafe { &*FS.0.get() };

    state
        .files
        .iter()
        .find(|file| file.name == name)
        .map(|file| file.content.as_slice())
        .ok_or(FsError::NotFound)
}

pub fn write(name: &str, content: &str) -> Result<(), FsError> {
    write_bytes(name, content.as_bytes())
}

pub fn write_bytes(name: &str, content: &[u8]) -> Result<(), FsError> {
    validate_name(name)?;

    let state = unsafe { &mut *FS.0.get() };
    let files = &mut state.files;
    let disk = &mut state.disk;

    if let Some(disk) = disk.as_mut() {
        return disk.write_file(name, content, files);
    }

    upsert_cache(files, name, content);
    crate::serial_println!("[CHRONO] fs: write {}", name);
    Ok(())
}

pub fn remove(name: &str) -> Result<(), FsError> {
    validate_name(name)?;

    let state = unsafe { &mut *FS.0.get() };
    let files = &mut state.files;
    let disk = &mut state.disk;

    if let Some(disk) = disk.as_mut() {
        return disk.remove_file(name, files);
    }

    let Some(index) = files.iter().position(|file| file.name == name) else {
        return Err(FsError::NotFound);
    };

    files.remove(index);
    crate::serial_println!("[CHRONO] fs: rm {}", name);
    Ok(())
}

fn validate_name(name: &str) -> Result<(), FsError> {
    if name.is_empty() {
        return Err(FsError::EmptyName);
    }

    if name.len() > MAX_FILENAME_LEN {
        return Err(FsError::NameTooLong);
    }

    if name.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return Err(FsError::InvalidName);
    }

    Ok(())
}

fn format_disk() -> Result<(), FsError> {
    let mut bitmap = alloc::vec![0u8; BITMAP_BYTES];
    for sector in 0..DATA_START {
        bitmap_set(&mut bitmap, sector, true);
    }

    let empty_sector = [0u8; SECTOR_SIZE];
    for sector in FILE_TABLE_START..FILE_TABLE_START + FILE_TABLE_SECTORS {
        ata::write_sector(sector, &empty_sector).map_err(|_| FsError::Disk)?;
    }

    for index in 0..BITMAP_SECTORS as usize {
        let mut sector = [0u8; SECTOR_SIZE];
        let start = index * SECTOR_SIZE;
        sector.copy_from_slice(&bitmap[start..start + SECTOR_SIZE]);
        ata::write_sector(BITMAP_START + index as u32, &sector).map_err(|_| FsError::Disk)?;
    }

    let mut superblock = [0u8; SECTOR_SIZE];
    write_superblock(&mut superblock, 0);
    ata::write_sector(0, &superblock).map_err(|_| FsError::Disk)?;

    Ok(())
}

fn read_superblock(sector: &[u8; SECTOR_SIZE]) -> Result<u32, FsError> {
    if sector[..8] != MAGIC[..] {
        return Err(FsError::Disk);
    }
    if read_u16(sector, 8) != VERSION
        || read_u16(sector, 10) != SECTOR_SIZE as u16
        || read_u32(sector, 12) != TOTAL_SECTORS
        || read_u32(sector, 20) != FILE_TABLE_START
        || read_u32(sector, 24) != FILE_TABLE_SECTORS
        || read_u32(sector, 28) != BITMAP_START
        || read_u32(sector, 32) != BITMAP_SECTORS
        || read_u32(sector, 36) != DATA_START
        || read_u32(sector, 40) != MAX_FILES as u32
        || read_u32(sector, 44) != MAX_FILE_BYTES as u32
    {
        return Err(FsError::Disk);
    }

    let expected = read_u32(sector, 508);
    if checksum(&sector[..508]) != expected {
        return Err(FsError::Disk);
    }

    let file_count = read_u32(sector, 16);
    if file_count > MAX_FILES as u32 {
        return Err(FsError::Disk);
    }

    Ok(file_count)
}

fn write_superblock(sector: &mut [u8; SECTOR_SIZE], file_count: u32) {
    sector[..8].copy_from_slice(&MAGIC);
    write_u16(sector, 8, VERSION);
    write_u16(sector, 10, SECTOR_SIZE as u16);
    write_u32(sector, 12, TOTAL_SECTORS);
    write_u32(sector, 16, file_count);
    write_u32(sector, 20, FILE_TABLE_START);
    write_u32(sector, 24, FILE_TABLE_SECTORS);
    write_u32(sector, 28, BITMAP_START);
    write_u32(sector, 32, BITMAP_SECTORS);
    write_u32(sector, 36, DATA_START);
    write_u32(sector, 40, MAX_FILES as u32);
    write_u32(sector, 44, MAX_FILE_BYTES as u32);
    let sum = checksum(&sector[..508]);
    write_u32(sector, 508, sum);
}

fn read_bitmap() -> Result<Vec<u8>, FsError> {
    let mut bitmap = alloc::vec![0u8; BITMAP_BYTES];

    for index in 0..BITMAP_SECTORS as usize {
        let mut sector = [0u8; SECTOR_SIZE];
        ata::read_sector(BITMAP_START + index as u32, &mut sector).map_err(|_| FsError::Disk)?;
        let start = index * SECTOR_SIZE;
        bitmap[start..start + SECTOR_SIZE].copy_from_slice(&sector);
    }

    Ok(bitmap)
}

fn read_file_table() -> Result<[DiskEntry; MAX_FILES], FsError> {
    let mut entries = [DiskEntry::empty(); MAX_FILES];

    for sector_index in 0..FILE_TABLE_SECTORS as usize {
        let mut sector = [0u8; SECTOR_SIZE];
        ata::read_sector(FILE_TABLE_START + sector_index as u32, &mut sector)
            .map_err(|_| FsError::Disk)?;

        for entry_offset in 0..ENTRIES_PER_SECTOR {
            let entry_index = sector_index * ENTRIES_PER_SECTOR + entry_offset;
            if entry_index >= MAX_FILES {
                break;
            }
            entries[entry_index] = read_entry(
                &sector[entry_offset * FILE_ENTRY_SIZE..(entry_offset + 1) * FILE_ENTRY_SIZE],
            );
        }
    }

    Ok(entries)
}

fn read_files(entries: &[DiskEntry; MAX_FILES]) -> Result<Vec<File>, FsError> {
    let mut files = Vec::new();

    for entry in entries {
        if !entry.used {
            continue;
        }

        let Some(name) = entry.name_str() else {
            continue;
        };
        if entry.size as usize > MAX_FILE_BYTES || entry.start_sector < DATA_START {
            continue;
        }
        if entry.start_sector.saturating_add(entry.sector_count) > TOTAL_SECTORS {
            continue;
        }
        if sectors_for(entry.size as usize) > entry.sector_count {
            continue;
        }

        let content = read_content(entry.start_sector, entry.sector_count, entry.size as usize)?;
        files.push(File {
            name: String::from(name),
            content,
        });
    }

    Ok(files)
}

fn read_entry(bytes: &[u8]) -> DiskEntry {
    let mut name = [0u8; MAX_FILENAME_LEN];
    name.copy_from_slice(&bytes[16..48]);

    DiskEntry {
        used: bytes[0] == 1,
        name_len: bytes[1],
        size: read_u32(bytes, 4),
        start_sector: read_u32(bytes, 8),
        sector_count: read_u32(bytes, 12),
        name,
    }
}

fn write_entry(entry: &DiskEntry, bytes: &mut [u8]) {
    bytes.fill(0);
    bytes[0] = u8::from(entry.used);
    bytes[1] = entry.name_len;
    write_u32(bytes, 4, entry.size);
    write_u32(bytes, 8, entry.start_sector);
    write_u32(bytes, 12, entry.sector_count);
    bytes[16..48].copy_from_slice(&entry.name);
}

fn read_content(start_sector: u32, sector_count: u32, size: usize) -> Result<Vec<u8>, FsError> {
    let mut data = Vec::new();

    for sector in start_sector..start_sector + sector_count {
        let mut buffer = [0u8; SECTOR_SIZE];
        ata::read_sector(sector, &mut buffer).map_err(|_| FsError::Disk)?;
        data.extend_from_slice(&buffer);
    }

    data.truncate(size);
    Ok(data)
}

fn write_content(start_sector: u32, sector_count: u32, content: &[u8]) -> Result<(), FsError> {
    for index in 0..sector_count {
        let mut buffer = [0u8; SECTOR_SIZE];
        let start = index as usize * SECTOR_SIZE;
        let end = (start + SECTOR_SIZE).min(content.len());
        if start < content.len() {
            buffer[..end - start].copy_from_slice(&content[start..end]);
        }
        ata::write_sector(start_sector + index, &buffer).map_err(|_| FsError::Disk)?;
    }

    Ok(())
}

fn sectors_for(byte_len: usize) -> u32 {
    ((byte_len + SECTOR_SIZE - 1) / SECTOR_SIZE).max(1) as u32
}

fn upsert_cache(files: &mut Vec<File>, name: &str, content: &[u8]) {
    if let Some(file) = files.iter_mut().find(|file| file.name == name) {
        file.content.clear();
        file.content.extend_from_slice(content);
    } else {
        files.push(File {
            name: String::from(name),
            content: Vec::from(content),
        });
    }
}

fn bitmap_get(bitmap: &[u8], sector: u32) -> bool {
    let byte = bitmap[sector as usize / 8];
    let bit = sector as u8 & 7;
    byte & (1 << bit) != 0
}

fn bitmap_set(bitmap: &mut [u8], sector: u32, used: bool) {
    let byte_index = sector as usize / 8;
    let bit = sector as u8 & 7;
    if used {
        bitmap[byte_index] |= 1 << bit;
    } else {
        bitmap[byte_index] &= !(1 << bit);
    }
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0u32, |sum, byte| sum.wrapping_add(*byte as u32))
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

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
