//! Kernel entrypoint and startup flow for Time Capsule OS.

#![no_std]
#![no_main]

mod keyboard;
mod panic;
mod serial;
mod shell;
mod theme;
mod vga_text;

use bootloader::{entry_point, BootInfo};
use crate::{println, serial_println};
use theme::Era;

const STARTUP_ERA: Era = Era::Nineties;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    serial::init();

    let profile = STARTUP_ERA.profile();
    vga_text::init(profile.fg, profile.bg);
    vga_text::clear();

    serial_println!("Time Capsule OS booting in {} mode", profile.name);
    serial_println!("{}", profile.boot_text);

    println!("{}", profile.boot_text);
    println!("Welcome to Time Capsule OS");
    println!();
    println!("{}", profile.banner);
    println!("Keyboard polling enabled.");
    println!("Type 'help' to see available commands.");

    shell::run(profile)
}
