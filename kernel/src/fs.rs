//! Tiny filesystem facade backed by ChronoFS when the ATA data disk is present.

use alloc::format;
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
const JOURNAL_NAME: &str = "__chronofs_journal";
const JOURNAL_MAGIC: [u8; 8] = *b"CHRJNL1\0";
const JOURNAL_VERSION: u16 = 1;
const JOURNAL_STATE_EMPTY: u8 = 0;
const JOURNAL_STATE_INTENT: u8 = 1;
const JOURNAL_STATE_COMMITTED: u8 = 2;
const JOURNAL_OP_NONE: u8 = 0;
const JOURNAL_OP_WRITE: u8 = 1;
const JOURNAL_OP_REMOVE: u8 = 2;
const JOURNAL_NAME_OFFSET: usize = 16;
const JOURNAL_OLD_ENTRY_OFFSET: usize = 48;
const JOURNAL_NEW_ENTRY_OFFSET: usize = JOURNAL_OLD_ENTRY_OFFSET + FILE_ENTRY_SIZE;
const JOURNAL_CHECKSUM_OFFSET: usize = 508;

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

pub struct FsckReport {
    pub disk_available: bool,
    pub repaired: bool,
    pub checked_entries: usize,
    pub live_entries: usize,
    pub warnings: usize,
    pub errors: usize,
    pub bitmap_mismatches: usize,
    pub duplicate_sectors: usize,
    pub invalid_entries: usize,
    pub repaired_items: usize,
    pub findings: Vec<String>,
}

impl FsckReport {
    fn new() -> Self {
        Self {
            disk_available: true,
            repaired: false,
            checked_entries: 0,
            live_entries: 0,
            warnings: 0,
            errors: 0,
            bitmap_mismatches: 0,
            duplicate_sectors: 0,
            invalid_entries: 0,
            repaired_items: 0,
            findings: Vec::new(),
        }
    }

    fn unavailable() -> Self {
        let mut report = Self::new();
        report.disk_available = false;
        report.add_warning(String::from(
            "persistent ChronoFS disk is unavailable; heap fallback cannot be checked",
        ));
        report
    }

    pub fn status_label(&self) -> &'static str {
        if !self.disk_available {
            "unavailable"
        } else if self.repaired {
            "repaired"
        } else if self.errors > 0 {
            "errors"
        } else if self.warnings > 0 {
            "warnings"
        } else {
            "clean"
        }
    }

    fn add_info(&mut self, finding: String) {
        self.findings.push(finding);
    }

    fn add_warning(&mut self, finding: String) {
        self.warnings += 1;
        self.findings.push(finding);
    }

    fn add_error(&mut self, finding: String) {
        self.errors += 1;
        self.findings.push(finding);
    }
}

pub struct JournalStatus {
    pub available: bool,
    pub clean: bool,
    pub state: &'static str,
    pub operation: &'static str,
    pub target: String,
    pub message: String,
}

impl JournalStatus {
    fn unavailable(message: &str) -> Self {
        Self {
            available: false,
            clean: false,
            state: "unavailable",
            operation: "none",
            target: String::new(),
            message: String::from(message),
        }
    }

    fn from_record(record: JournalRecord) -> Self {
        let target = String::from(record.target_name().unwrap_or(""));
        let state = journal_state_label(record.state);
        let operation = journal_operation_label(record.operation);
        let clean = record.state == JOURNAL_STATE_EMPTY;
        let message = if clean {
            String::from("journal is clean")
        } else {
            String::from("journal has an operation record")
        };

        Self {
            available: true,
            clean,
            state,
            operation,
            target,
            message,
        }
    }
}

pub struct FsStatus {
    pub disk_available: bool,
    pub persistent: bool,
    pub cache_file_count: usize,
    pub visible_file_count: usize,
    pub disk_file_count: usize,
    pub used_file_slots: usize,
    pub free_file_slots: usize,
    pub max_files: usize,
    pub max_file_bytes: usize,
    pub total_sectors: u32,
    pub data_start_sector: u32,
    pub journal_present: bool,
    pub journal: JournalStatus,
}

#[derive(Clone, Copy)]
pub enum FileStorage {
    PersistentDisk,
    HeapFallback,
}

impl FileStorage {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::PersistentDisk => "persistent ATA disk",
            Self::HeapFallback => "heap fallback",
        }
    }
}

pub struct FileInfo {
    pub name: String,
    pub size_bytes: usize,
    pub storage: FileStorage,
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

#[derive(Clone, Copy)]
struct JournalRecord {
    state: u8,
    operation: u8,
    entry_index: u8,
    name_len: u8,
    name: [u8; MAX_FILENAME_LEN],
    old_entry: DiskEntry,
    new_entry: DiskEntry,
}

impl JournalRecord {
    const fn empty() -> Self {
        Self {
            state: JOURNAL_STATE_EMPTY,
            operation: JOURNAL_OP_NONE,
            entry_index: 0,
            name_len: 0,
            name: [0; MAX_FILENAME_LEN],
            old_entry: DiskEntry::empty(),
            new_entry: DiskEntry::empty(),
        }
    }

    fn new(
        state: u8,
        operation: u8,
        entry_index: usize,
        name: &str,
        old_entry: DiskEntry,
        new_entry: DiskEntry,
    ) -> Self {
        let mut record = Self::empty();
        record.state = state;
        record.operation = operation;
        record.entry_index = entry_index as u8;
        record.name_len = name.len() as u8;
        record.name[..name.len()].copy_from_slice(name.as_bytes());
        record.old_entry = old_entry;
        record.new_entry = new_entry;
        record
    }

    fn from_sector(sector: &[u8; SECTOR_SIZE]) -> Result<Self, FsError> {
        if sector[..8] != JOURNAL_MAGIC[..] {
            return Err(FsError::Disk);
        }
        if read_u16(sector, 8) != JOURNAL_VERSION {
            return Err(FsError::Disk);
        }
        if checksum(&sector[..JOURNAL_CHECKSUM_OFFSET]) != read_u32(sector, JOURNAL_CHECKSUM_OFFSET)
        {
            return Err(FsError::Disk);
        }

        let name_len = sector[12];
        if name_len as usize > MAX_FILENAME_LEN {
            return Err(FsError::Disk);
        }

        let mut name = [0u8; MAX_FILENAME_LEN];
        name.copy_from_slice(&sector[JOURNAL_NAME_OFFSET..JOURNAL_NAME_OFFSET + MAX_FILENAME_LEN]);

        Ok(Self {
            state: sector[10],
            operation: sector[11],
            entry_index: sector[13],
            name_len,
            name,
            old_entry: read_entry(
                &sector[JOURNAL_OLD_ENTRY_OFFSET..JOURNAL_OLD_ENTRY_OFFSET + FILE_ENTRY_SIZE],
            ),
            new_entry: read_entry(
                &sector[JOURNAL_NEW_ENTRY_OFFSET..JOURNAL_NEW_ENTRY_OFFSET + FILE_ENTRY_SIZE],
            ),
        })
    }

    fn write_to_sector(&self, sector: &mut [u8; SECTOR_SIZE]) {
        sector.fill(0);
        sector[..8].copy_from_slice(&JOURNAL_MAGIC);
        write_u16(sector, 8, JOURNAL_VERSION);
        sector[10] = self.state;
        sector[11] = self.operation;
        sector[12] = self.name_len;
        sector[13] = self.entry_index;
        sector[JOURNAL_NAME_OFFSET..JOURNAL_NAME_OFFSET + MAX_FILENAME_LEN]
            .copy_from_slice(&self.name);
        write_entry(
            &self.old_entry,
            &mut sector[JOURNAL_OLD_ENTRY_OFFSET..JOURNAL_OLD_ENTRY_OFFSET + FILE_ENTRY_SIZE],
        );
        write_entry(
            &self.new_entry,
            &mut sector[JOURNAL_NEW_ENTRY_OFFSET..JOURNAL_NEW_ENTRY_OFFSET + FILE_ENTRY_SIZE],
        );
        let sum = checksum(&sector[..JOURNAL_CHECKSUM_OFFSET]);
        write_u32(sector, JOURNAL_CHECKSUM_OFFSET, sum);
    }

    fn target_name(&self) -> Option<&str> {
        if self.name_len == 0 || self.name_len as usize > MAX_FILENAME_LEN {
            return None;
        }

        core::str::from_utf8(&self.name[..self.name_len as usize]).ok()
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
        let mut disk = Self {
            entries,
            bitmap,
            file_count: 0,
        };
        disk.file_count = count_used_entries(&disk.entries) as u32;
        disk.recover_journal_on_mount();
        disk.ensure_journal();
        disk.file_count = count_used_entries(&disk.entries) as u32;
        let files = read_files(&disk.entries)?;
        let file_count = disk.file_count;

        if file_count != superblock_file_count {
            crate::serial_println!(
                "[CHRONO] fs: mounted ChronoFS files={} superblock={}",
                file_count,
                superblock_file_count
            );
        } else {
            crate::serial_println!("[CHRONO] fs: mounted ChronoFS files={}", file_count);
        }

        Ok((disk, files))
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
        let start_sector = self.find_free_run(sector_count).ok_or(FsError::NoSpace)?;
        let mut new_entry = DiskEntry::empty();
        new_entry.set(name, content.len(), start_sector, sector_count);

        write_content(start_sector, sector_count, content)?;
        let mut journal = JournalRecord::new(
            JOURNAL_STATE_INTENT,
            JOURNAL_OP_WRITE,
            entry_index,
            name,
            old_entry,
            new_entry,
        );
        self.write_journal_record(&journal)?;

        if old_entry.used {
            self.mark_range(old_entry.start_sector, old_entry.sector_count, false);
        }
        self.mark_range(start_sector, sector_count, true);

        self.entries[entry_index] = new_entry;

        if existing_index.is_none() {
            self.file_count += 1;
        }

        self.sync_bitmap()?;
        self.sync_entry_sector(entry_index)?;
        self.sync_superblock()?;
        journal.state = JOURNAL_STATE_COMMITTED;
        self.write_journal_record(&journal)?;
        self.complete_journal(&journal)?;
        upsert_cache(files, name, content);

        crate::serial_println!("[CHRONO] fs: write {}", name);
        Ok(())
    }

    fn remove_file(&mut self, name: &str, files: &mut Vec<File>) -> Result<(), FsError> {
        let Some(entry_index) = self.entries.iter().position(|entry| entry.matches(name)) else {
            return Err(FsError::NotFound);
        };

        let old_entry = self.entries[entry_index];
        let new_entry = DiskEntry::empty();
        let mut journal = JournalRecord::new(
            JOURNAL_STATE_INTENT,
            JOURNAL_OP_REMOVE,
            entry_index,
            name,
            old_entry,
            new_entry,
        );
        self.write_journal_record(&journal)?;

        self.mark_range(old_entry.start_sector, old_entry.sector_count, false);
        self.entries[entry_index] = new_entry;
        self.file_count = self.file_count.saturating_sub(1);

        self.sync_bitmap()?;
        self.sync_entry_sector(entry_index)?;
        self.sync_superblock()?;
        journal.state = JOURNAL_STATE_COMMITTED;
        self.write_journal_record(&journal)?;
        self.complete_journal(&journal)?;

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

    fn ensure_journal(&mut self) {
        if self.journal_entry().is_some() {
            return;
        }

        let Some(entry_index) = self.entries.iter().position(|entry| !entry.used) else {
            crate::serial_println!("[CHRONO] fs: journal unavailable; no free file slot");
            return;
        };
        let Some(start_sector) = self.find_free_run(1) else {
            crate::serial_println!("[CHRONO] fs: journal unavailable; no free data sector");
            return;
        };

        let mut entry = DiskEntry::empty();
        entry.set(JOURNAL_NAME, SECTOR_SIZE, start_sector, 1);
        self.entries[entry_index] = entry;
        self.mark_range(start_sector, 1, true);
        self.file_count += 1;

        if self.write_journal_record(&JournalRecord::empty()).is_err()
            || self.sync_bitmap().is_err()
            || self.sync_entry_sector(entry_index).is_err()
            || self.sync_superblock().is_err()
        {
            crate::serial_println!("[CHRONO] fs: journal creation failed; run fsck");
            return;
        }

        crate::serial_println!("[CHRONO] fs: journal created");
    }

    fn recover_journal_on_mount(&mut self) {
        let Some(_) = self.journal_entry() else {
            crate::serial_println!("[CHRONO] fs: journal missing; creating");
            return;
        };

        let record = match self.read_journal_record() {
            Ok(record) => record,
            Err(_) => {
                crate::serial_println!(
                    "[CHRONO] fs: journal invalid; recovery refused, run fsck"
                );
                return;
            }
        };

        match record.state {
            JOURNAL_STATE_EMPTY => {
                crate::serial_println!("[CHRONO] fs: journal clean");
            }
            JOURNAL_STATE_INTENT => {
                crate::serial_println!(
                    "[CHRONO] fs: journal rollback for {} {}",
                    journal_operation_label(record.operation),
                    record.target_name().unwrap_or("?")
                );
                self.apply_journal_recovery(record, record.old_entry);
            }
            JOURNAL_STATE_COMMITTED => {
                crate::serial_println!(
                    "[CHRONO] fs: journal roll-forward for {} {}",
                    journal_operation_label(record.operation),
                    record.target_name().unwrap_or("?")
                );
                self.apply_journal_recovery(record, record.new_entry);
            }
            _ => {
                crate::serial_println!(
                    "[CHRONO] fs: journal state unsupported; recovery refused, run fsck"
                );
            }
        }
    }

    fn apply_journal_recovery(&mut self, record: JournalRecord, target_entry: DiskEntry) {
        let entry_index = record.entry_index as usize;
        if !journal_record_is_safe(record, target_entry) || entry_index >= MAX_FILES {
            crate::serial_println!("[CHRONO] fs: journal recovery refused; unsafe record");
            return;
        }
        if self
            .journal_entry()
            .map(|(journal_index, _)| journal_index == entry_index)
            .unwrap_or(false)
        {
            crate::serial_println!("[CHRONO] fs: journal recovery refused; unsafe record");
            return;
        }

        let old_entry = self.entries[entry_index];
        self.entries[entry_index] = target_entry;
        let Some(bitmap) = rebuild_bitmap_from_entries(&self.entries) else {
            self.entries[entry_index] = old_entry;
            crate::serial_println!(
                "[CHRONO] fs: journal recovery refused; metadata still conflicts"
            );
            return;
        };

        self.bitmap = bitmap;
        self.file_count = count_used_entries(&self.entries) as u32;

        if self.sync_entry_sector(entry_index).is_err()
            || self.sync_bitmap().is_err()
            || self.sync_superblock().is_err()
            || self.complete_journal(&record).is_err()
        {
            crate::serial_println!("[CHRONO] fs: journal recovery write failed; run fsck");
            return;
        }

        crate::serial_println!("[CHRONO] fs: journal recovery complete");
    }

    fn journal_entry(&self) -> Option<(usize, DiskEntry)> {
        self.entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.matches(JOURNAL_NAME))
            .map(|(index, entry)| (index, *entry))
    }

    fn read_journal_record(&self) -> Result<JournalRecord, FsError> {
        let Some((_, entry)) = self.journal_entry() else {
            return Err(FsError::Disk);
        };
        if !journal_entry_extent_is_valid(&entry) {
            return Err(FsError::Disk);
        }

        let mut sector = [0u8; SECTOR_SIZE];
        ata::read_sector(entry.start_sector, &mut sector).map_err(|_| FsError::Disk)?;
        JournalRecord::from_sector(&sector)
    }

    fn write_journal_record(&self, record: &JournalRecord) -> Result<(), FsError> {
        let Some((_, entry)) = self.journal_entry() else {
            return Err(FsError::Disk);
        };
        if !journal_entry_extent_is_valid(&entry) {
            return Err(FsError::Disk);
        }

        let mut sector = [0u8; SECTOR_SIZE];
        record.write_to_sector(&mut sector);
        ata::write_sector(entry.start_sector, &sector).map_err(|_| FsError::Disk)
    }

    #[allow(dead_code)]
    fn clear_journal(&self) -> Result<(), FsError> {
        self.write_journal_record(&JournalRecord::empty())
    }

    fn complete_journal(&self, record: &JournalRecord) -> Result<(), FsError> {
        let mut clean_record = *record;
        clean_record.state = JOURNAL_STATE_EMPTY;
        self.write_journal_record(&clean_record)
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

pub fn file_info(name: &str) -> Result<FileInfo, FsError> {
    validate_name(name)?;

    let state: &'static FsState = unsafe { &*FS.0.get() };
    let storage = if state.disk.is_some() {
        FileStorage::PersistentDisk
    } else {
        FileStorage::HeapFallback
    };

    state
        .files
        .iter()
        .find(|file| file.name == name)
        .map(|file| FileInfo {
            name: file.name.clone(),
            size_bytes: file.content.len(),
            storage,
        })
        .ok_or(FsError::NotFound)
}

pub fn file_exists(name: &str) -> Result<bool, FsError> {
    validate_name(name)?;

    let state: &'static FsState = unsafe { &*FS.0.get() };
    Ok(state.files.iter().any(|file| file.name == name))
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

pub fn check(repair: bool) -> FsckReport {
    let state = unsafe { &mut *FS.0.get() };
    let Some(disk) = state.disk.as_mut() else {
        return FsckReport::unavailable();
    };

    check_disk(disk, repair)
}

pub fn status() -> FsStatus {
    let state: &'static FsState = unsafe { &*FS.0.get() };
    let cache_file_count = state.files.len();

    let Some(disk) = state.disk.as_ref() else {
        return FsStatus {
            disk_available: false,
            persistent: false,
            cache_file_count,
            visible_file_count: cache_file_count,
            disk_file_count: 0,
            used_file_slots: cache_file_count,
            free_file_slots: MAX_FILES.saturating_sub(cache_file_count),
            max_files: MAX_FILES,
            max_file_bytes: MAX_FILE_BYTES,
            total_sectors: TOTAL_SECTORS,
            data_start_sector: DATA_START,
            journal_present: false,
            journal: JournalStatus::unavailable("persistent ChronoFS disk is unavailable"),
        };
    };

    let used_file_slots = count_used_entries(&disk.entries);
    let journal_present = disk.journal_entry().is_some();
    let journal = match disk.read_journal_record() {
        Ok(record) => JournalStatus::from_record(record),
        Err(_) => JournalStatus {
            available: journal_present,
            clean: false,
            state: "invalid",
            operation: "unknown",
            target: String::new(),
            message: String::from("journal record is missing, corrupt, or has a bad checksum"),
        },
    };

    FsStatus {
        disk_available: true,
        persistent: true,
        cache_file_count,
        visible_file_count: cache_file_count,
        disk_file_count: disk.file_count as usize,
        used_file_slots,
        free_file_slots: MAX_FILES.saturating_sub(used_file_slots),
        max_files: MAX_FILES,
        max_file_bytes: MAX_FILE_BYTES,
        total_sectors: TOTAL_SECTORS,
        data_start_sector: DATA_START,
        journal_present,
        journal,
    }
}

pub fn journal_status() -> JournalStatus {
    let state: &'static FsState = unsafe { &*FS.0.get() };
    let Some(disk) = state.disk.as_ref() else {
        return JournalStatus::unavailable("persistent ChronoFS disk is unavailable");
    };

    match disk.read_journal_record() {
        Ok(record) => JournalStatus::from_record(record),
        Err(_) => JournalStatus {
            available: true,
            clean: false,
            state: "invalid",
            operation: "unknown",
            target: String::new(),
            message: String::from("journal record is missing, corrupt, or has a bad checksum"),
        },
    }
}

fn check_disk(disk: &mut DiskState, repair: bool) -> FsckReport {
    let mut report = FsckReport::new();
    let mut superblock = [0u8; SECTOR_SIZE];

    if ata::read_sector(0, &mut superblock).is_err() {
        report.add_error(String::from("could not read ChronoFS superblock"));
        return report;
    }

    let Some(superblock_file_count) = check_superblock(&superblock, &mut report) else {
        if repair {
            report.add_info(String::from(
                "repair refused: superblock is not trusted enough to repair safely",
            ));
        }
        return report;
    };

    let mut entries = match read_file_table() {
        Ok(entries) => entries,
        Err(_) => {
            report.add_error(String::from("could not read ChronoFS file table"));
            return report;
        }
    };
    let mut bitmap = match read_bitmap() {
        Ok(bitmap) => bitmap,
        Err(_) => {
            report.add_error(String::from("could not read ChronoFS allocation bitmap"));
            return report;
        }
    };

    let mut owners = alloc::vec![u16::MAX; TOTAL_SECTORS as usize];
    let mut duplicate_seen = alloc::vec![false; TOTAL_SECTORS as usize];
    let mut names: Vec<(String, usize)> = Vec::new();
    let mut stale_empty_slots: Vec<usize> = Vec::new();
    let mut live_entries = 0usize;

    for (index, entry) in entries.iter().enumerate() {
        report.checked_entries += 1;

        if !entry.used {
            if entry_has_stale_metadata(entry) {
                report.invalid_entries += 1;
                report.add_warning(format!(
                    "entry {} is empty but still contains stale metadata",
                    index
                ));
                stale_empty_slots.push(index);
            }
            continue;
        }

        live_entries += 1;
        report.live_entries += 1;

        let Some(name) = check_entry_name(index, entry, &mut report) else {
            continue;
        };

        if let Some((_, previous_index)) = names
            .iter()
            .find(|(existing_name, _)| existing_name.as_str() == name)
        {
            report.invalid_entries += 1;
            report.add_error(format!(
                "entries {} and {} both use filename '{}'",
                previous_index,
                index,
                name
            ));
        } else {
            names.push((String::from(name), index));
        }

        if !check_entry_extent(index, entry, &mut report) {
            continue;
        }

        for sector in entry.start_sector..entry.start_sector + entry.sector_count {
            let owner = &mut owners[sector as usize];
            if *owner == u16::MAX {
                *owner = index as u16;
            } else if !duplicate_seen[sector as usize] {
                duplicate_seen[sector as usize] = true;
                report.duplicate_sectors += 1;
                report.add_error(format!(
                    "sector {} is claimed by more than one file entry",
                    sector
                ));
            }
        }
    }

    if superblock_file_count as usize != live_entries {
        report.add_warning(format!(
            "superblock file count is {}, but file table has {} live entries",
            superblock_file_count, live_entries
        ));
    }

    let mut missing_reserved = 0usize;
    let mut missing_owned = 0usize;
    let mut leaked_free = 0usize;

    for sector in 0..TOTAL_SECTORS {
        let expected = sector < DATA_START || owners[sector as usize] != u16::MAX;
        let actual = bitmap_get(&bitmap, sector);

        if actual == expected {
            continue;
        }

        report.bitmap_mismatches += 1;
        if sector < DATA_START {
            missing_reserved += 1;
        } else if expected {
            missing_owned += 1;
        } else {
            leaked_free += 1;
        }
    }

    if missing_reserved > 0 {
        report.add_warning(format!(
            "{} metadata sector bitmap bit(s) are clear but should be reserved",
            missing_reserved
        ));
    }
    if missing_owned > 0 {
        report.add_warning(format!(
            "{} file-owned sector bitmap bit(s) are clear",
            missing_owned
        ));
    }
    if leaked_free > 0 {
        report.add_warning(format!(
            "{} free data sector bitmap bit(s) are still marked used",
            leaked_free
        ));
    }

    if !repair {
        return report;
    }

    if report.errors > 0 {
        report.add_info(String::from(
            "repair refused: unsafe errors require manual investigation first",
        ));
        return report;
    }

    for index in stale_empty_slots {
        entries[index] = DiskEntry::empty();
        disk.entries[index] = DiskEntry::empty();
        match disk.sync_entry_sector(index) {
            Ok(()) => {
                report.repaired = true;
                report.repaired_items += 1;
                report.add_info(format!("repaired entry {} by clearing stale metadata", index));
            }
            Err(_) => {
                report.add_error(format!("could not write repaired file table entry {}", index));
                return report;
            }
        }
    }

    if report.bitmap_mismatches > 0 {
        for sector in 0..TOTAL_SECTORS {
            let expected = sector < DATA_START || owners[sector as usize] != u16::MAX;
            bitmap_set(&mut bitmap, sector, expected);
        }

        disk.bitmap = bitmap;
        match disk.sync_bitmap() {
            Ok(()) => {
                report.repaired = true;
                report.repaired_items += report.bitmap_mismatches;
                report.add_info(format!(
                    "repaired {} allocation bitmap bit(s)",
                    report.bitmap_mismatches
                ));
            }
            Err(_) => {
                report.add_error(String::from("could not write repaired allocation bitmap"));
            }
        }
    }

    report
}

fn check_superblock(sector: &[u8; SECTOR_SIZE], report: &mut FsckReport) -> Option<u32> {
    let mut valid = true;

    if sector[..8] != MAGIC[..] {
        report.add_error(String::from("superblock magic does not match ChronoFS"));
        valid = false;
    }
    if read_u16(sector, 8) != VERSION {
        report.add_error(String::from("superblock version is unsupported"));
        valid = false;
    }
    if read_u16(sector, 10) != SECTOR_SIZE as u16 {
        report.add_error(String::from("superblock sector size does not match kernel"));
        valid = false;
    }
    if read_u32(sector, 12) != TOTAL_SECTORS
        || read_u32(sector, 20) != FILE_TABLE_START
        || read_u32(sector, 24) != FILE_TABLE_SECTORS
        || read_u32(sector, 28) != BITMAP_START
        || read_u32(sector, 32) != BITMAP_SECTORS
        || read_u32(sector, 36) != DATA_START
        || read_u32(sector, 40) != MAX_FILES as u32
        || read_u32(sector, 44) != MAX_FILE_BYTES as u32
    {
        report.add_error(String::from("superblock geometry does not match ChronoFS layout"));
        valid = false;
    }

    let expected = read_u32(sector, 508);
    if checksum(&sector[..508]) != expected {
        report.add_error(String::from("superblock checksum is invalid"));
        valid = false;
    }

    let file_count = read_u32(sector, 16);
    if file_count > MAX_FILES as u32 {
        report.add_error(String::from("superblock file count exceeds file table capacity"));
        valid = false;
    }

    if valid {
        Some(file_count)
    } else {
        None
    }
}

fn check_entry_name<'a>(
    index: usize,
    entry: &'a DiskEntry,
    report: &mut FsckReport,
) -> Option<&'a str> {
    if entry.name_len == 0 || entry.name_len as usize > MAX_FILENAME_LEN {
        report.invalid_entries += 1;
        report.add_error(format!("entry {} has an invalid filename length", index));
        return None;
    }

    let name_bytes = &entry.name[..entry.name_len as usize];
    let Ok(name) = core::str::from_utf8(name_bytes) else {
        report.invalid_entries += 1;
        report.add_error(format!("entry {} filename is not valid UTF-8", index));
        return None;
    };

    if name.bytes().any(|byte| byte.is_ascii_whitespace()) {
        report.invalid_entries += 1;
        report.add_error(format!("entry {} filename contains whitespace", index));
        return None;
    }

    Some(name)
}

fn check_entry_extent(index: usize, entry: &DiskEntry, report: &mut FsckReport) -> bool {
    let expected_sectors = sectors_for(entry.size as usize);
    let Some(end_sector) = entry.start_sector.checked_add(entry.sector_count) else {
        report.invalid_entries += 1;
        report.add_error(format!("entry {} file extent overflows sector numbers", index));
        return false;
    };

    if entry.size as usize > MAX_FILE_BYTES {
        report.invalid_entries += 1;
        report.add_error(format!("entry {} is larger than the ChronoFS file limit", index));
        return false;
    }

    if entry.sector_count != expected_sectors {
        report.invalid_entries += 1;
        report.add_error(format!(
            "entry {} has {} sector(s) but size requires {}",
            index, entry.sector_count, expected_sectors
        ));
        return false;
    }

    if entry.start_sector < DATA_START || end_sector > TOTAL_SECTORS {
        report.invalid_entries += 1;
        report.add_error(format!("entry {} file extent is outside the data area", index));
        return false;
    }

    true
}

fn entry_has_stale_metadata(entry: &DiskEntry) -> bool {
    entry.name_len != 0
        || entry.size != 0
        || entry.start_sector != 0
        || entry.sector_count != 0
        || entry.name.iter().any(|byte| *byte != 0)
}

fn journal_state_label(state: u8) -> &'static str {
    match state {
        JOURNAL_STATE_EMPTY => "empty",
        JOURNAL_STATE_INTENT => "intent",
        JOURNAL_STATE_COMMITTED => "committed",
        _ => "unknown",
    }
}

fn journal_operation_label(operation: u8) -> &'static str {
    match operation {
        JOURNAL_OP_WRITE => "write",
        JOURNAL_OP_REMOVE => "remove",
        _ => "none",
    }
}

fn journal_entry_extent_is_valid(entry: &DiskEntry) -> bool {
    let Some(end_sector) = entry.start_sector.checked_add(1) else {
        return false;
    };

    entry.used
        && entry.size as usize == SECTOR_SIZE
        && entry.sector_count == 1
        && entry.start_sector >= DATA_START
        && end_sector <= TOTAL_SECTORS
}

fn journal_record_is_safe(record: JournalRecord, target_entry: DiskEntry) -> bool {
    if record.entry_index as usize >= MAX_FILES {
        return false;
    }
    if record.operation != JOURNAL_OP_WRITE && record.operation != JOURNAL_OP_REMOVE {
        return false;
    }
    let Some(name) = record.target_name() else {
        return false;
    };
    if name == JOURNAL_NAME || name.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return false;
    }

    journal_snapshot_is_safe(record.old_entry, name)
        && journal_snapshot_is_safe(record.new_entry, name)
        && journal_snapshot_is_safe(target_entry, name)
}

fn journal_snapshot_is_safe(entry: DiskEntry, name: &str) -> bool {
    if !entry.used {
        return !entry_has_stale_metadata(&entry);
    }

    if entry.name_str() != Some(name) {
        return false;
    }
    if entry.size as usize > MAX_FILE_BYTES {
        return false;
    }
    let Some(end_sector) = entry.start_sector.checked_add(entry.sector_count) else {
        return false;
    };

    entry.sector_count == sectors_for(entry.size as usize)
        && entry.start_sector >= DATA_START
        && end_sector <= TOTAL_SECTORS
}

fn rebuild_bitmap_from_entries(entries: &[DiskEntry; MAX_FILES]) -> Option<Vec<u8>> {
    let mut bitmap = alloc::vec![0u8; BITMAP_BYTES];
    let mut names: Vec<String> = Vec::new();
    for sector in 0..DATA_START {
        bitmap_set(&mut bitmap, sector, true);
    }

    for entry in entries {
        if !entry.used {
            continue;
        }
        let name = entry.name_str()?;
        if names
            .iter()
            .any(|existing_name| existing_name.as_str() == name)
        {
            return None;
        }
        names.push(String::from(name));

        if entry.size as usize > MAX_FILE_BYTES {
            return None;
        }
        let end_sector = entry.start_sector.checked_add(entry.sector_count)?;
        if entry.sector_count != sectors_for(entry.size as usize)
            || entry.start_sector < DATA_START
            || end_sector > TOTAL_SECTORS
        {
            return None;
        }

        for sector in entry.start_sector..end_sector {
            if bitmap_get(&bitmap, sector) {
                return None;
            }
            bitmap_set(&mut bitmap, sector, true);
        }
    }

    Some(bitmap)
}

fn count_used_entries(entries: &[DiskEntry; MAX_FILES]) -> usize {
    entries.iter().filter(|entry| entry.used).count()
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

    if name == JOURNAL_NAME {
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
        if name == JOURNAL_NAME {
            continue;
        }
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
