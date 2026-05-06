//! Minimal ATA PIO sector I/O for QEMU's primary IDE channel.

use crate::io;

pub const SECTOR_SIZE: usize = 512;

const DATA_PORT: u16 = 0x1F0;
const ERROR_PORT: u16 = 0x1F1;
const SECTOR_COUNT_PORT: u16 = 0x1F2;
const LBA_LOW_PORT: u16 = 0x1F3;
const LBA_MID_PORT: u16 = 0x1F4;
const LBA_HIGH_PORT: u16 = 0x1F5;
const DRIVE_HEAD_PORT: u16 = 0x1F6;
const STATUS_COMMAND_PORT: u16 = 0x1F7;
const ALT_STATUS_CONTROL_PORT: u16 = 0x3F6;

const STATUS_ERR: u8 = 1 << 0;
const STATUS_DRQ: u8 = 1 << 3;
const STATUS_DF: u8 = 1 << 5;
const STATUS_DRDY: u8 = 1 << 6;
const STATUS_BSY: u8 = 1 << 7;

const DRIVE_PRIMARY_SLAVE_LBA: u8 = 0xF0;
const COMMAND_READ_SECTORS: u8 = 0x20;
const COMMAND_WRITE_SECTORS: u8 = 0x30;
const COMMAND_CACHE_FLUSH: u8 = 0xE7;
const POLL_LIMIT: usize = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtaError {
    Timeout,
    DeviceFault,
    DeviceError(u8),
    LbaOutOfRange,
}

pub fn read_sector(lba: u32, buffer: &mut [u8; SECTOR_SIZE]) -> Result<(), AtaError> {
    prepare_lba(lba)?;

    unsafe {
        io::outb(STATUS_COMMAND_PORT, COMMAND_READ_SECTORS);
    }

    wait_for_data()?;

    for word_index in 0..SECTOR_SIZE / 2 {
        let word = unsafe { io::inw(DATA_PORT) };
        let [low, high] = word.to_le_bytes();
        let byte_index = word_index * 2;
        buffer[byte_index] = low;
        buffer[byte_index + 1] = high;
    }

    ata_delay();
    crate::serial_println!("[CHRONO] disk: read sector {}", lba);
    Ok(())
}

pub fn write_sector(lba: u32, buffer: &[u8; SECTOR_SIZE]) -> Result<(), AtaError> {
    prepare_lba(lba)?;

    unsafe {
        io::outb(STATUS_COMMAND_PORT, COMMAND_WRITE_SECTORS);
    }

    wait_for_data()?;

    for word_index in 0..SECTOR_SIZE / 2 {
        let byte_index = word_index * 2;
        let word = u16::from_le_bytes([buffer[byte_index], buffer[byte_index + 1]]);
        unsafe {
            io::outw(DATA_PORT, word);
        }
    }

    flush_cache()?;
    crate::serial_println!("[CHRONO] disk: write sector {}", lba);
    Ok(())
}

fn prepare_lba(lba: u32) -> Result<(), AtaError> {
    if lba >= (1 << 28) {
        return Err(AtaError::LbaOutOfRange);
    }

    wait_not_busy()?;

    unsafe {
        io::outb(
            DRIVE_HEAD_PORT,
            DRIVE_PRIMARY_SLAVE_LBA | ((lba >> 24) as u8 & 0x0F),
        );
    }
    ata_delay();
    wait_ready()?;

    unsafe {
        io::outb(SECTOR_COUNT_PORT, 1);
        io::outb(LBA_LOW_PORT, lba as u8);
        io::outb(LBA_MID_PORT, (lba >> 8) as u8);
        io::outb(LBA_HIGH_PORT, (lba >> 16) as u8);
    }

    Ok(())
}

fn flush_cache() -> Result<(), AtaError> {
    wait_not_busy()?;

    unsafe {
        io::outb(STATUS_COMMAND_PORT, COMMAND_CACHE_FLUSH);
    }

    wait_not_busy()?;
    check_error()
}

fn wait_not_busy() -> Result<(), AtaError> {
    for _ in 0..POLL_LIMIT {
        let status = status();
        if status & STATUS_BSY == 0 {
            return check_error();
        }
    }

    Err(AtaError::Timeout)
}

fn wait_ready() -> Result<(), AtaError> {
    for _ in 0..POLL_LIMIT {
        let status = status();
        if status & STATUS_BSY == 0 && status & STATUS_DRDY != 0 {
            return check_error();
        }
    }

    Err(AtaError::Timeout)
}

fn wait_for_data() -> Result<(), AtaError> {
    for _ in 0..POLL_LIMIT {
        let status = status();
        if status & STATUS_BSY != 0 {
            continue;
        }
        if status & STATUS_DF != 0 {
            return Err(AtaError::DeviceFault);
        }
        if status & STATUS_ERR != 0 {
            return Err(AtaError::DeviceError(error()));
        }
        if status & STATUS_DRQ != 0 {
            return Ok(());
        }
    }

    Err(AtaError::Timeout)
}

fn check_error() -> Result<(), AtaError> {
    let status = status();
    if status & STATUS_DF != 0 {
        return Err(AtaError::DeviceFault);
    }
    if status & STATUS_ERR != 0 {
        return Err(AtaError::DeviceError(error()));
    }
    Ok(())
}

fn status() -> u8 {
    unsafe { io::inb(STATUS_COMMAND_PORT) }
}

fn error() -> u8 {
    unsafe { io::inb(ERROR_PORT) }
}

fn ata_delay() {
    for _ in 0..4 {
        unsafe {
            io::inb(ALT_STATUS_CONTROL_PORT);
        }
    }
}
