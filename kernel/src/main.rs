//! Kernel entrypoint and startup flow for Time Capsule OS.

#![no_std]
#![no_main]

mod console;
mod keyboard;
mod panic;
mod serial;
mod shell;
mod theme;
mod vga_text;

use bootloader::{entry_point, BootInfo};
use theme::Era;

const STARTUP_ERA: Era = Era::Eighties;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    serial::init();
    serial_println!("[TCOS] boot start");
    serial_println!("[TCOS] serial initialized");

    let profile = STARTUP_ERA.profile();
    console::init(profile.fg, profile.bg);
    console::clear();
    serial_println!("[TCOS] console initialized");

    let era_name = STARTUP_ERA.name();
    serial_println!("[TCOS] active era: {}", era_name);

    println!("{}", profile.boot_welcome);
    println!("Era: {}", profile.name);

    serial_println!("[TCOS] boot complete");

    shell::run(STARTUP_ERA)
}
