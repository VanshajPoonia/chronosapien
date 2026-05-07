use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

const SECTOR_SIZE: usize = 512;
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

#[derive(Clone, Copy)]
struct Entry {
    used: bool,
    name_len: u8,
    name: [u8; 32],
    size: u32,
    start_sector: u32,
    sector_count: u32,
}

impl Entry {
    const fn empty() -> Self {
        Self {
            used: false,
            name_len: 0,
            name: [0; 32],
            size: 0,
            start_sector: 0,
            sector_count: 0,
        }
    }

    fn name(&self) -> Option<&str> {
        if !self.used {
            return None;
        }

        std::str::from_utf8(&self.name[..self.name_len as usize]).ok()
    }

    fn matches(&self, name: &str) -> bool {
        self.name() == Some(name)
    }

    fn set(&mut self, name: &str, size: usize, start_sector: u32, sector_count: u32) {
        self.used = true;
        self.name_len = name.len() as u8;
        self.name = [0; 32];
        self.name[..name.len()].copy_from_slice(name.as_bytes());
        self.size = size as u32;
        self.start_sector = start_sector;
        self.sector_count = sector_count;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: chronofs_put <chronofs-data.img> <host-file> <chrono-name>");
        std::process::exit(2);
    }

    let image_path = Path::new(&args[1]);
    let host_path = Path::new(&args[2]);
    let chrono_name = &args[3];

    if chrono_name.is_empty()
        || chrono_name.len() > 32
        || chrono_name.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        eprintln!("invalid ChronoFS filename");
        std::process::exit(2);
    }

    let content = fs::read(host_path).expect("read host file");
    if content.len() > MAX_FILE_BYTES {
        eprintln!("file too large: {} bytes", content.len());
        std::process::exit(2);
    }

    if !image_path.exists() {
        let file = fs::File::create(image_path).expect("create data image");
        file.set_len(TOTAL_SECTORS as u64 * SECTOR_SIZE as u64)
            .expect("size data image");
    }

    let mut image = OpenOptions::new()
        .read(true)
        .write(true)
        .open(image_path)
        .expect("open data image");

    if image.metadata().expect("metadata").len() < TOTAL_SECTORS as u64 * SECTOR_SIZE as u64 {
        image
            .set_len(TOTAL_SECTORS as u64 * SECTOR_SIZE as u64)
            .expect("extend data image");
    }

    if read_superblock(&mut image).is_none() {
        format_disk(&mut image);
    }

    let mut entries = read_entries(&mut image);
    let mut bitmap = read_bitmap(&mut image);
    put_file(&mut image, &mut entries, &mut bitmap, chrono_name, &content);
    println!(
        "wrote {} bytes to {} as {}",
        content.len(),
        image_path.display(),
        chrono_name
    );
}

fn put_file(
    image: &mut fs::File,
    entries: &mut [Entry; MAX_FILES],
    bitmap: &mut [u8],
    name: &str,
    content: &[u8],
) {
    let sector_count = sectors_for(content.len());
    let existing = entries.iter().position(|entry| entry.matches(name));
    let index = existing.unwrap_or_else(|| {
        entries
            .iter()
            .position(|entry| !entry.used)
            .expect("no free file entries")
    });

    let old = entries[index];
    let same_location = old.used && old.sector_count >= sector_count;
    let start_sector = if same_location {
        old.start_sector
    } else {
        find_free_run(bitmap, sector_count).expect("not enough ChronoFS space")
    };

    write_content(image, start_sector, sector_count, content);

    if same_location && old.sector_count > sector_count {
        mark_range(
            bitmap,
            old.start_sector + sector_count,
            old.sector_count - sector_count,
            false,
        );
    }
    if !same_location && old.used {
        mark_range(bitmap, old.start_sector, old.sector_count, false);
    }
    if !same_location {
        mark_range(bitmap, start_sector, sector_count, true);
    }

    entries[index].set(name, content.len(), start_sector, sector_count);
    write_bitmap(image, bitmap);
    write_entry_sector(image, entries, index);
    write_superblock(image, entries.iter().filter(|entry| entry.used).count() as u32);
}

fn format_disk(image: &mut fs::File) {
    let empty = [0u8; SECTOR_SIZE];
    for sector in FILE_TABLE_START..FILE_TABLE_START + FILE_TABLE_SECTORS {
        write_sector(image, sector, &empty);
    }

    let mut bitmap = vec![0u8; BITMAP_BYTES];
    for sector in 0..DATA_START {
        bitmap_set(&mut bitmap, sector, true);
    }
    write_bitmap(image, &bitmap);
    write_superblock(image, 0);
}

fn read_superblock(image: &mut fs::File) -> Option<u32> {
    let sector = read_sector(image, 0);
    if sector[..8] != MAGIC[..] {
        return None;
    }
    if read_u16(&sector, 8) != VERSION
        || read_u16(&sector, 10) != SECTOR_SIZE as u16
        || read_u32(&sector, 12) != TOTAL_SECTORS
        || read_u32(&sector, 20) != FILE_TABLE_START
        || read_u32(&sector, 24) != FILE_TABLE_SECTORS
        || read_u32(&sector, 28) != BITMAP_START
        || read_u32(&sector, 32) != BITMAP_SECTORS
        || read_u32(&sector, 36) != DATA_START
        || read_u32(&sector, 40) != MAX_FILES as u32
        || read_u32(&sector, 44) != MAX_FILE_BYTES as u32
    {
        return None;
    }
    let expected = read_u32(&sector, 508);
    if checksum(&sector[..508]) != expected {
        return None;
    }
    Some(read_u32(&sector, 16))
}

fn write_superblock(image: &mut fs::File, file_count: u32) {
    let mut sector = [0u8; SECTOR_SIZE];
    sector[..8].copy_from_slice(&MAGIC);
    write_u16(&mut sector, 8, VERSION);
    write_u16(&mut sector, 10, SECTOR_SIZE as u16);
    write_u32(&mut sector, 12, TOTAL_SECTORS);
    write_u32(&mut sector, 16, file_count);
    write_u32(&mut sector, 20, FILE_TABLE_START);
    write_u32(&mut sector, 24, FILE_TABLE_SECTORS);
    write_u32(&mut sector, 28, BITMAP_START);
    write_u32(&mut sector, 32, BITMAP_SECTORS);
    write_u32(&mut sector, 36, DATA_START);
    write_u32(&mut sector, 40, MAX_FILES as u32);
    write_u32(&mut sector, 44, MAX_FILE_BYTES as u32);
    let sum = checksum(&sector[..508]);
    write_u32(&mut sector, 508, sum);
    write_sector(image, 0, &sector);
}

fn read_entries(image: &mut fs::File) -> [Entry; MAX_FILES] {
    let mut entries = [Entry::empty(); MAX_FILES];

    for sector_index in 0..FILE_TABLE_SECTORS as usize {
        let sector = read_sector(image, FILE_TABLE_START + sector_index as u32);
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

    entries
}

fn write_entry_sector(image: &mut fs::File, entries: &[Entry; MAX_FILES], entry_index: usize) {
    let sector_index = entry_index / ENTRIES_PER_SECTOR;
    let mut sector = [0u8; SECTOR_SIZE];
    let first_entry = sector_index * ENTRIES_PER_SECTOR;

    for index in 0..ENTRIES_PER_SECTOR {
        let Some(entry) = entries.get(first_entry + index) else {
            break;
        };
        write_entry(
            entry,
            &mut sector[index * FILE_ENTRY_SIZE..(index + 1) * FILE_ENTRY_SIZE],
        );
    }

    write_sector(image, FILE_TABLE_START + sector_index as u32, &sector);
}

fn read_entry(bytes: &[u8]) -> Entry {
    let mut name = [0u8; 32];
    name.copy_from_slice(&bytes[16..48]);
    Entry {
        used: bytes[0] == 1,
        name_len: bytes[1],
        size: read_u32(bytes, 4),
        start_sector: read_u32(bytes, 8),
        sector_count: read_u32(bytes, 12),
        name,
    }
}

fn write_entry(entry: &Entry, bytes: &mut [u8]) {
    bytes.fill(0);
    bytes[0] = u8::from(entry.used);
    bytes[1] = entry.name_len;
    write_u32(bytes, 4, entry.size);
    write_u32(bytes, 8, entry.start_sector);
    write_u32(bytes, 12, entry.sector_count);
    bytes[16..48].copy_from_slice(&entry.name);
}

fn read_bitmap(image: &mut fs::File) -> Vec<u8> {
    let mut bitmap = vec![0u8; BITMAP_BYTES];
    for index in 0..BITMAP_SECTORS as usize {
        let sector = read_sector(image, BITMAP_START + index as u32);
        let start = index * SECTOR_SIZE;
        bitmap[start..start + SECTOR_SIZE].copy_from_slice(&sector);
    }
    bitmap
}

fn write_bitmap(image: &mut fs::File, bitmap: &[u8]) {
    for index in 0..BITMAP_SECTORS as usize {
        let mut sector = [0u8; SECTOR_SIZE];
        let start = index * SECTOR_SIZE;
        sector.copy_from_slice(&bitmap[start..start + SECTOR_SIZE]);
        write_sector(image, BITMAP_START + index as u32, &sector);
    }
}

fn write_content(image: &mut fs::File, start_sector: u32, sector_count: u32, content: &[u8]) {
    for index in 0..sector_count {
        let mut sector = [0u8; SECTOR_SIZE];
        let start = index as usize * SECTOR_SIZE;
        let end = (start + SECTOR_SIZE).min(content.len());
        if start < content.len() {
            sector[..end - start].copy_from_slice(&content[start..end]);
        }
        write_sector(image, start_sector + index, &sector);
    }
}

fn find_free_run(bitmap: &[u8], sector_count: u32) -> Option<u32> {
    let mut run_start = DATA_START;
    let mut run_len = 0u32;

    for sector in DATA_START..TOTAL_SECTORS {
        if bitmap_get(bitmap, sector) {
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

fn mark_range(bitmap: &mut [u8], start: u32, count: u32, used: bool) {
    for sector in start..start + count {
        bitmap_set(bitmap, sector, used);
    }
}

fn sectors_for(byte_len: usize) -> u32 {
    ((byte_len + SECTOR_SIZE - 1) / SECTOR_SIZE).max(1) as u32
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
        .fold(0u32, |acc, byte| acc.wrapping_add(*byte as u32))
}

fn read_sector(image: &mut fs::File, sector: u32) -> [u8; SECTOR_SIZE] {
    let mut bytes = [0u8; SECTOR_SIZE];
    image
        .seek(SeekFrom::Start(sector as u64 * SECTOR_SIZE as u64))
        .expect("seek sector");
    image.read_exact(&mut bytes).expect("read sector");
    bytes
}

fn write_sector(image: &mut fs::File, sector: u32, bytes: &[u8; SECTOR_SIZE]) {
    image
        .seek(SeekFrom::Start(sector as u64 * SECTOR_SIZE as u64))
        .expect("seek sector");
    image.write_all(bytes).expect("write sector");
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
