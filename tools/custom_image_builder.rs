use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SECTOR_SIZE: usize = 512;
const STAGE2_RESERVED_SECTORS: u64 = 64;
const MAX_SEGMENTS: usize = 8;

#[derive(Clone)]
struct LoadSegment {
    destination: u64,
    data: Vec<u8>,
    lba: u64,
    sectors: u16,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!(
            "usage: custom_image_builder <stage1.bin> <stage2.bin> <kernel-elf> <output.img>"
        );
        std::process::exit(2);
    }

    let stage1_path = PathBuf::from(&args[1]);
    let stage2_path = PathBuf::from(&args[2]);
    let kernel_path = PathBuf::from(&args[3]);
    let output_path = PathBuf::from(&args[4]);

    let stage1 = fs::read(&stage1_path).expect("read stage1");
    let mut stage2 = fs::read(&stage2_path).expect("read stage2");
    let kernel = fs::read(&kernel_path).expect("read kernel elf");

    assert_stage1(&stage1, &stage1_path);
    assert_stage2(&stage2, &stage2_path);

    let entry = find_symbol(&kernel, "chrono_custom_entry").expect("chrono_custom_entry symbol");
    let mut segments = load_segments(&kernel);
    if segments.len() > MAX_SEGMENTS {
        panic!("too many loadable kernel segments: {}", segments.len());
    }

    let kernel_addr = segments
        .iter()
        .map(|segment| segment.destination)
        .min()
        .unwrap_or(0);
    let kernel_end = segments
        .iter()
        .map(|segment| segment.destination + segment.data.len() as u64)
        .max()
        .unwrap_or(kernel_addr);
    let kernel_len = kernel_end.saturating_sub(kernel_addr);

    let mut next_lba = 1 + STAGE2_RESERVED_SECTORS;
    for segment in segments.iter_mut() {
        segment.lba = next_lba;
        segment.sectors = sectors_for(segment.data.len()) as u16;
        next_lba += segment.sectors as u64;
    }

    patch_stage2_manifest(&mut stage2, entry, kernel_addr, kernel_len, &segments);

    let mut image = Vec::new();
    image.extend_from_slice(&stage1);
    append_padded(&mut image, &stage2, STAGE2_RESERVED_SECTORS as usize * SECTOR_SIZE);
    for segment in &segments {
        append_sector_padded(&mut image, &segment.data);
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    fs::write(&output_path, image).expect("write image");
    write_manifest_sidecar(&output_path, entry, kernel_addr, kernel_len, &segments);
}

fn assert_stage1(stage1: &[u8], path: &Path) {
    if stage1.len() != SECTOR_SIZE {
        panic!("{} must be exactly 512 bytes", path.display());
    }

    if stage1[510] != 0x55 || stage1[511] != 0xAA {
        panic!("{} is missing the 55 AA boot signature", path.display());
    }
}

fn assert_stage2(stage2: &[u8], path: &Path) {
    let max_len = STAGE2_RESERVED_SECTORS as usize * SECTOR_SIZE;
    if stage2.len() > max_len {
        panic!(
            "{} is {} bytes, but Stage 1 reserves only {} bytes",
            path.display(),
            stage2.len(),
            max_len
        );
    }
}

fn patch_stage2_manifest(
    stage2: &mut [u8],
    entry: u64,
    kernel_addr: u64,
    kernel_len: u64,
    segments: &[LoadSegment],
) {
    let offset = find_bytes(stage2, b"CHRONO2M").expect("stage2 manifest magic");
    write_u64(stage2, offset + 8, entry);
    write_u64(stage2, offset + 16, kernel_addr);
    write_u64(stage2, offset + 24, kernel_len);
    write_u16(stage2, offset + 32, segments.len() as u16);

    let mut entry_offset = offset + 40;
    for segment in segments {
        write_u64(stage2, entry_offset, segment.lba);
        write_u16(stage2, entry_offset + 8, segment.sectors);
        write_u64(stage2, entry_offset + 16, segment.destination);
        write_u64(stage2, entry_offset + 24, segment.data.len() as u64);
        entry_offset += 32;
    }
}

fn write_manifest_sidecar(
    output_path: &Path,
    entry: u64,
    kernel_addr: u64,
    kernel_len: u64,
    segments: &[LoadSegment],
) {
    let mut text = String::new();
    text.push_str("ChronoOS custom boot image manifest\n");
    text.push_str(&format!("entry=0x{entry:x}\n"));
    text.push_str(&format!("kernel_addr=0x{kernel_addr:x}\n"));
    text.push_str(&format!("kernel_len={kernel_len}\n"));
    text.push_str(&format!("checksum=0x{:08x}\n", checksum(segments)));
    for (index, segment) in segments.iter().enumerate() {
        text.push_str(&format!(
            "segment{} lba={} sectors={} dest=0x{:x} bytes={}\n",
            index,
            segment.lba,
            segment.sectors,
            segment.destination,
            segment.data.len()
        ));
    }
    fs::write(output_path.with_extension("manifest.txt"), text).expect("write manifest sidecar");
}

fn append_padded(image: &mut Vec<u8>, data: &[u8], reserved_len: usize) {
    image.extend_from_slice(data);
    image.resize(image.len() + reserved_len.saturating_sub(data.len()), 0);
}

fn append_sector_padded(image: &mut Vec<u8>, data: &[u8]) {
    image.extend_from_slice(data);
    let padding = (SECTOR_SIZE - image.len() % SECTOR_SIZE) % SECTOR_SIZE;
    image.resize(image.len() + padding, 0);
}

fn sectors_for(byte_len: usize) -> u64 {
    ((byte_len + SECTOR_SIZE - 1) / SECTOR_SIZE) as u64
}

fn checksum(segments: &[LoadSegment]) -> u32 {
    segments
        .iter()
        .flat_map(|segment| segment.data.iter())
        .fold(0u32, |sum, byte| sum.wrapping_add(*byte as u32))
}

fn load_segments(elf: &[u8]) -> Vec<LoadSegment> {
    let phoff = read_u64(elf, 32) as usize;
    let phentsize = read_u16(elf, 54) as usize;
    let phnum = read_u16(elf, 56) as usize;
    let mut segments = Vec::new();

    for index in 0..phnum {
        let offset = phoff + index * phentsize;
        let p_type = read_u32(elf, offset);
        if p_type != 1 {
            continue;
        }

        let p_offset = read_u64(elf, offset + 8) as usize;
        let p_paddr = read_u64(elf, offset + 24);
        let p_filesz = read_u64(elf, offset + 32) as usize;
        let p_memsz = read_u64(elf, offset + 40) as usize;
        let mut data = vec![0; p_memsz];
        data[..p_filesz].copy_from_slice(&elf[p_offset..p_offset + p_filesz]);

        segments.push(LoadSegment {
            destination: p_paddr,
            data,
            lba: 0,
            sectors: 0,
        });
    }

    segments
}

fn find_symbol(elf: &[u8], name: &str) -> Option<u64> {
    let shoff = read_u64(elf, 40) as usize;
    let shentsize = read_u16(elf, 58) as usize;
    let shnum = read_u16(elf, 60) as usize;

    for index in 0..shnum {
        let section = shoff + index * shentsize;
        let sh_type = read_u32(elf, section + 4);
        if sh_type != 2 && sh_type != 11 {
            continue;
        }

        let strtab_index = read_u32(elf, section + 40) as usize;
        let strtab = shoff + strtab_index * shentsize;
        let strtab_offset = read_u64(elf, strtab + 24) as usize;
        let strtab_size = read_u64(elf, strtab + 32) as usize;
        let sym_offset = read_u64(elf, section + 24) as usize;
        let sym_size = read_u64(elf, section + 32) as usize;
        let sym_entsize = read_u64(elf, section + 56) as usize;

        for sym in (sym_offset..sym_offset + sym_size).step_by(sym_entsize) {
            let name_offset = read_u32(elf, sym) as usize;
            let symbol_name = cstr_at(&elf[strtab_offset..strtab_offset + strtab_size], name_offset);
            if symbol_name == name {
                return Some(read_u64(elf, sym + 8));
            }
        }
    }

    None
}

fn cstr_at(bytes: &[u8], offset: usize) -> &str {
    let end = bytes[offset..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|position| offset + position)
        .unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[offset..end]).expect("symbol name utf8")
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
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

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
