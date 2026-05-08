use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const SECTOR_SIZE: usize = 512;
const IMAGE_SECTORS: u64 = 131_072;
const PARTITION_FIRST_LBA: u64 = 2048;
const GPT_ENTRY_COUNT: usize = 128;
const GPT_ENTRY_SIZE: usize = 128;
const RESERVED_SECTORS: u32 = 32;
const FAT_COUNT: u32 = 2;
const SECTORS_PER_CLUSTER: u32 = 1;
const END_OF_CHAIN: u32 = 0x0fff_ffff;

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 4 {
        eprintln!(
            "usage: {} <BOOTX64.EFI> <KERNEL.ELF> <output.img>",
            args.first().map(String::as_str).unwrap_or("uefi_image_builder")
        );
        std::process::exit(2);
    }

    let loader = fs::read(&args[1])?;
    let kernel = fs::read(&args[2])?;
    let output = Path::new(&args[3]);
    let mut image = vec![0u8; IMAGE_SECTORS as usize * SECTOR_SIZE];

    let partition_last_lba = IMAGE_SECTORS - 34;
    let partition_sectors = partition_last_lba - PARTITION_FIRST_LBA + 1;
    let fat_sectors = fat_sectors_for(partition_sectors as u32);
    let first_data_sector = RESERVED_SECTORS + FAT_COUNT * fat_sectors;
    let cluster_count =
        (partition_sectors as u32 - first_data_sector) / SECTORS_PER_CLUSTER;

    if cluster_count < 65_525 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "UEFI image must be large enough for FAT32",
        ));
    }

    let mut fat = vec![0u32; cluster_count as usize + 2];
    fat[0] = 0x0fff_fff8;
    fat[1] = END_OF_CHAIN;

    let root_cluster = 2;
    let efi_cluster = 3;
    let boot_cluster = 4;
    let chrono_cluster = 5;
    fat[root_cluster] = END_OF_CHAIN;
    fat[efi_cluster] = END_OF_CHAIN;
    fat[boot_cluster] = END_OF_CHAIN;
    fat[chrono_cluster] = END_OF_CHAIN;

    let mut next_cluster = 6u32;
    let loader_chain = allocate_chain(&mut fat, &mut next_cluster, loader.len());
    let kernel_chain = allocate_chain(&mut fat, &mut next_cluster, kernel.len());

    write_gpt(
        &mut image,
        PARTITION_FIRST_LBA,
        partition_last_lba,
        partition_sectors,
    );
    write_fat32_boot_area(
        &mut image,
        partition_sectors as u32,
        fat_sectors,
        cluster_count,
        root_cluster as u32,
    );
    write_fats(&mut image, &fat, fat_sectors);

    write_directory_cluster(
        &mut image,
        first_data_sector,
        root_cluster as u32,
        &[
            DirectoryEntry::dir(*b"EFI        ", efi_cluster as u32),
            DirectoryEntry::dir(*b"CHRONO     ", chrono_cluster as u32),
        ],
    );
    write_directory_cluster(
        &mut image,
        first_data_sector,
        efi_cluster as u32,
        &[
            DirectoryEntry::dot(efi_cluster as u32),
            DirectoryEntry::dotdot(root_cluster as u32),
            DirectoryEntry::dir(*b"BOOT       ", boot_cluster as u32),
        ],
    );
    write_directory_cluster(
        &mut image,
        first_data_sector,
        boot_cluster as u32,
        &[
            DirectoryEntry::dot(boot_cluster as u32),
            DirectoryEntry::dotdot(efi_cluster as u32),
            DirectoryEntry::file(*b"BOOTX64 EFI", loader_chain[0], loader.len() as u32),
        ],
    );
    write_directory_cluster(
        &mut image,
        first_data_sector,
        chrono_cluster as u32,
        &[
            DirectoryEntry::dot(chrono_cluster as u32),
            DirectoryEntry::dotdot(root_cluster as u32),
            DirectoryEntry::file(*b"KERNEL  ELF", kernel_chain[0], kernel.len() as u32),
        ],
    );
    write_file_clusters(&mut image, first_data_sector, &loader_chain, &loader);
    write_file_clusters(&mut image, first_data_sector, &kernel_chain, &kernel);

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(output)?;
    file.write_all(&image)?;

    println!(
        "UEFI image written to {} ({} MiB)",
        output.display(),
        image.len() / 1024 / 1024
    );
    Ok(())
}

fn fat_sectors_for(partition_sectors: u32) -> u32 {
    let mut fat_sectors = 1u32;
    loop {
        let data_sectors = partition_sectors - RESERVED_SECTORS - FAT_COUNT * fat_sectors;
        let clusters = data_sectors / SECTORS_PER_CLUSTER;
        let needed = ((clusters + 2) * 4 + SECTOR_SIZE as u32 - 1) / SECTOR_SIZE as u32;
        if needed == fat_sectors {
            return fat_sectors;
        }
        fat_sectors = needed;
    }
}

fn allocate_chain(fat: &mut [u32], next_cluster: &mut u32, byte_len: usize) -> Vec<u32> {
    let cluster_size = SECTOR_SIZE * SECTORS_PER_CLUSTER as usize;
    let clusters = byte_len.max(1).div_ceil(cluster_size);
    let start = *next_cluster;
    let mut chain = Vec::with_capacity(clusters);

    for index in 0..clusters {
        let cluster = start + index as u32;
        chain.push(cluster);
        fat[cluster as usize] = if index + 1 == clusters {
            END_OF_CHAIN
        } else {
            cluster + 1
        };
    }

    *next_cluster += clusters as u32;
    chain
}

fn write_gpt(image: &mut [u8], first_lba: u64, last_lba: u64, partition_sectors: u64) {
    let mut mbr = [0u8; SECTOR_SIZE];
    mbr[446 + 4] = 0xee;
    write_le32(&mut mbr, 446 + 8, 1);
    write_le32(
        &mut mbr,
        446 + 12,
        (IMAGE_SECTORS - 1).min(u32::MAX as u64) as u32,
    );
    mbr[510] = 0x55;
    mbr[511] = 0xaa;
    image[..SECTOR_SIZE].copy_from_slice(&mbr);

    let mut entries = vec![0u8; GPT_ENTRY_COUNT * GPT_ENTRY_SIZE];
    let entry = &mut entries[..GPT_ENTRY_SIZE];
    entry[0..16].copy_from_slice(&[
        0x28, 0x73, 0x2a, 0xc1, 0x1f, 0xf8, 0xd2, 0x11, 0xba, 0x4b, 0x00, 0xa0, 0xc9,
        0x3e, 0xc9, 0x3b,
    ]);
    entry[16..32].copy_from_slice(&[
        0x43, 0x48, 0x52, 0x4f, 0x4e, 0x4f, 0x53, 0x50, 0x49, 0x45, 0x4e, 0x55, 0x45,
        0x46, 0x49, 0x31,
    ]);
    write_le64(entry, 32, first_lba);
    write_le64(entry, 40, last_lba);
    write_utf16_name(entry, 56, "ChronoOS ESP");

    let entries_crc = crc32(&entries);
    let entries_lba = 2u64;
    let backup_entries_lba = IMAGE_SECTORS - 33;
    let entries_len = entries.len();
    lba_slice_mut(image, entries_lba, entries_len).copy_from_slice(&entries);
    lba_slice_mut(image, backup_entries_lba, entries_len).copy_from_slice(&entries);

    let mut primary = gpt_header(
        1,
        IMAGE_SECTORS - 1,
        34,
        IMAGE_SECTORS - 34,
        entries_lba,
        entries_crc,
    );
    let primary_crc = crc32(&primary[..92]);
    write_le32(&mut primary, 16, primary_crc);
    lba_slice_mut(image, 1, SECTOR_SIZE).copy_from_slice(&primary);

    let mut backup = gpt_header(
        IMAGE_SECTORS - 1,
        1,
        34,
        IMAGE_SECTORS - 34,
        backup_entries_lba,
        entries_crc,
    );
    let backup_crc = crc32(&backup[..92]);
    write_le32(&mut backup, 16, backup_crc);
    lba_slice_mut(image, IMAGE_SECTORS - 1, SECTOR_SIZE).copy_from_slice(&backup);

    assert_eq!(partition_sectors, last_lba - first_lba + 1);
}

fn gpt_header(
    current_lba: u64,
    backup_lba: u64,
    first_usable_lba: u64,
    last_usable_lba: u64,
    entries_lba: u64,
    entries_crc: u32,
) -> [u8; SECTOR_SIZE] {
    let mut header = [0u8; SECTOR_SIZE];
    header[0..8].copy_from_slice(b"EFI PART");
    write_le32(&mut header, 8, 0x0001_0000);
    write_le32(&mut header, 12, 92);
    write_le64(&mut header, 24, current_lba);
    write_le64(&mut header, 32, backup_lba);
    write_le64(&mut header, 40, first_usable_lba);
    write_le64(&mut header, 48, last_usable_lba);
    header[56..72].copy_from_slice(&[
        0x43, 0x48, 0x52, 0x4f, 0x4e, 0x4f, 0x53, 0x44, 0x49, 0x53, 0x4b, 0x55, 0x45,
        0x46, 0x49, 0x32,
    ]);
    write_le64(&mut header, 72, entries_lba);
    write_le32(&mut header, 80, GPT_ENTRY_COUNT as u32);
    write_le32(&mut header, 84, GPT_ENTRY_SIZE as u32);
    write_le32(&mut header, 88, entries_crc);
    header
}

fn write_fat32_boot_area(
    image: &mut [u8],
    partition_sectors: u32,
    fat_sectors: u32,
    cluster_count: u32,
    root_cluster: u32,
) {
    let mut boot = [0u8; SECTOR_SIZE];
    boot[0..3].copy_from_slice(&[0xeb, 0x58, 0x90]);
    boot[3..11].copy_from_slice(b"CHRONOOS");
    write_le16(&mut boot, 11, SECTOR_SIZE as u16);
    boot[13] = SECTORS_PER_CLUSTER as u8;
    write_le16(&mut boot, 14, RESERVED_SECTORS as u16);
    boot[16] = FAT_COUNT as u8;
    boot[21] = 0xf8;
    write_le16(&mut boot, 24, 63);
    write_le16(&mut boot, 26, 255);
    write_le32(&mut boot, 28, PARTITION_FIRST_LBA as u32);
    write_le32(&mut boot, 32, partition_sectors);
    write_le32(&mut boot, 36, fat_sectors);
    write_le32(&mut boot, 44, root_cluster);
    write_le16(&mut boot, 48, 1);
    write_le16(&mut boot, 50, 6);
    boot[64] = 0x80;
    boot[66] = 0x29;
    write_le32(&mut boot, 67, 0xc405_1984);
    boot[71..82].copy_from_slice(b"CHRONO ESP ");
    boot[82..90].copy_from_slice(b"FAT32   ");
    boot[510] = 0x55;
    boot[511] = 0xaa;
    partition_slice_mut(image, 0, SECTOR_SIZE).copy_from_slice(&boot);
    partition_slice_mut(image, 6, SECTOR_SIZE).copy_from_slice(&boot);

    let mut fs_info = [0u8; SECTOR_SIZE];
    write_le32(&mut fs_info, 0, 0x4161_5252);
    write_le32(&mut fs_info, 484, 0x6141_7272);
    write_le32(&mut fs_info, 488, cluster_count.saturating_sub(6));
    write_le32(&mut fs_info, 492, 6);
    fs_info[510] = 0x55;
    fs_info[511] = 0xaa;
    partition_slice_mut(image, 1, SECTOR_SIZE).copy_from_slice(&fs_info);
    partition_slice_mut(image, 7, SECTOR_SIZE).copy_from_slice(&fs_info);
}

fn write_fats(image: &mut [u8], fat: &[u32], fat_sectors: u32) {
    let fat_bytes_len = fat_sectors as usize * SECTOR_SIZE;
    let mut fat_bytes = vec![0u8; fat_bytes_len];
    for (index, value) in fat.iter().enumerate() {
        let offset = index * 4;
        if offset + 4 <= fat_bytes.len() {
            write_le32(&mut fat_bytes, offset, *value);
        }
    }

    let first_fat = RESERVED_SECTORS as u64;
    let second_fat = RESERVED_SECTORS as u64 + fat_sectors as u64;
    partition_slice_mut(image, first_fat, fat_bytes_len).copy_from_slice(&fat_bytes);
    partition_slice_mut(image, second_fat, fat_bytes_len).copy_from_slice(&fat_bytes);
}

fn write_directory_cluster(
    image: &mut [u8],
    first_data_sector: u32,
    cluster: u32,
    entries: &[DirectoryEntry],
) {
    let mut bytes = [0u8; SECTOR_SIZE * SECTORS_PER_CLUSTER as usize];
    for (index, entry) in entries.iter().enumerate() {
        let offset = index * 32;
        bytes[offset..offset + 11].copy_from_slice(&entry.name);
        bytes[offset + 11] = entry.attr;
        write_le16(&mut bytes, offset + 20, (entry.cluster >> 16) as u16);
        write_le16(&mut bytes, offset + 26, entry.cluster as u16);
        write_le32(&mut bytes, offset + 28, entry.size);
    }
    cluster_slice_mut(image, first_data_sector, cluster, bytes.len()).copy_from_slice(&bytes);
}

fn write_file_clusters(image: &mut [u8], first_data_sector: u32, chain: &[u32], bytes: &[u8]) {
    let cluster_size = SECTOR_SIZE * SECTORS_PER_CLUSTER as usize;
    for (index, cluster) in chain.iter().enumerate() {
        let start = index * cluster_size;
        let end = (start + cluster_size).min(bytes.len());
        let destination = cluster_slice_mut(image, first_data_sector, *cluster, cluster_size);
        destination.fill(0);
        if start < end {
            destination[..end - start].copy_from_slice(&bytes[start..end]);
        }
    }
}

#[derive(Clone, Copy)]
struct DirectoryEntry {
    name: [u8; 11],
    attr: u8,
    cluster: u32,
    size: u32,
}

impl DirectoryEntry {
    fn dir(name: [u8; 11], cluster: u32) -> Self {
        Self {
            name,
            attr: 0x10,
            cluster,
            size: 0,
        }
    }

    fn file(name: [u8; 11], cluster: u32, size: u32) -> Self {
        Self {
            name,
            attr: 0x20,
            cluster,
            size,
        }
    }

    fn dot(cluster: u32) -> Self {
        Self::dir(*b".          ", cluster)
    }

    fn dotdot(cluster: u32) -> Self {
        Self::dir(*b"..         ", cluster)
    }
}

fn lba_slice_mut(image: &mut [u8], lba: u64, byte_len: usize) -> &mut [u8] {
    let start = lba as usize * SECTOR_SIZE;
    &mut image[start..start + byte_len]
}

fn partition_slice_mut(image: &mut [u8], sector: u64, byte_len: usize) -> &mut [u8] {
    lba_slice_mut(image, PARTITION_FIRST_LBA + sector, byte_len)
}

fn cluster_slice_mut(
    image: &mut [u8],
    first_data_sector: u32,
    cluster: u32,
    byte_len: usize,
) -> &mut [u8] {
    let sector = first_data_sector as u64 + (cluster as u64 - 2) * SECTORS_PER_CLUSTER as u64;
    partition_slice_mut(image, sector, byte_len)
}

fn write_utf16_name(bytes: &mut [u8], offset: usize, name: &str) {
    for (index, code_unit) in name.encode_utf16().take(36).enumerate() {
        write_le16(bytes, offset + index * 2, code_unit);
    }
}

fn write_le16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_le32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_le64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}
