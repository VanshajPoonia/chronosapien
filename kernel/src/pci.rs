//! Minimal PCI configuration-space scanner.

use crate::io;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;
const MAX_BUS: u8 = 0;
const MAX_DEVICE: u8 = 32;
const MAX_FUNCTION: u8 = 8;

const VENDOR_DEVICE_REGISTER: u8 = 0x00;
const COMMAND_REGISTER: u8 = 0x04;
const HEADER_TYPE_REGISTER: u8 = 0x0C;
const BAR0_REGISTER: u8 = 0x10;

const COMMAND_IO_SPACE: u16 = 1 << 0;
const COMMAND_BUS_MASTER: u16 = 1 << 2;

#[derive(Clone, Copy)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
}

impl Device {
    pub fn bar0(self) -> u32 {
        read_config_u32(self.bus, self.device, self.function, BAR0_REGISTER)
    }

    pub fn enable_io_and_bus_mastering(self) {
        let command = read_config_u16(self.bus, self.device, self.function, COMMAND_REGISTER);
        write_config_u16(
            self.bus,
            self.device,
            self.function,
            COMMAND_REGISTER,
            command | COMMAND_IO_SPACE | COMMAND_BUS_MASTER,
        );
    }
}

pub fn find_device(vendor_id: u16, device_id: u16) -> Option<Device> {
    for bus in 0..=MAX_BUS {
        for device in 0..MAX_DEVICE {
            let header = read_config_u8(bus, device, 0, HEADER_TYPE_REGISTER);
            let functions = if header & 0x80 != 0 { MAX_FUNCTION } else { 1 };

            for function in 0..functions {
                let value = read_config_u32(bus, device, function, VENDOR_DEVICE_REGISTER);
                let found_vendor = value as u16;

                if found_vendor == 0xFFFF {
                    continue;
                }

                let found_device = (value >> 16) as u16;
                if found_vendor == vendor_id && found_device == device_id {
                    return Some(Device {
                        bus,
                        device,
                        function,
                        vendor_id: found_vendor,
                        device_id: found_device,
                    });
                }
            }
        }
    }

    None
}

fn read_config_u8(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    let shift = (offset & 3) * 8;

    (read_config_u32(bus, device, function, offset & !3) >> shift) as u8
}

fn read_config_u16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let shift = (offset & 2) * 8;

    (read_config_u32(bus, device, function, offset & !3) >> shift) as u16
}

fn read_config_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = config_address(bus, device, function, offset);

    unsafe {
        io::outl(CONFIG_ADDRESS, address);
        io::inl(CONFIG_DATA)
    }
}

fn write_config_u16(bus: u8, device: u8, function: u8, offset: u8, value: u16) {
    let aligned = offset & !3;
    let shift = (offset & 2) * 8;
    let mask = !(0xFFFFu32 << shift);
    let current = read_config_u32(bus, device, function, aligned);
    let next = (current & mask) | ((value as u32) << shift);

    unsafe {
        io::outl(CONFIG_ADDRESS, config_address(bus, device, function, aligned));
        io::outl(CONFIG_DATA, next);
    }
}

fn config_address(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    0x8000_0000
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC)
}
