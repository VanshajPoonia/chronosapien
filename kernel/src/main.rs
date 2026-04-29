//! Kernel entrypoint and startup flow for Time Capsule OS.

#![no_std]
#![no_main]

mod panic;
mod serial;
mod theme;
mod vga_text;

use bootloader::{entry_point, BootInfo};
use theme::Era;

const STARTUP_ERA: Era = Era::Eighties;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    serial::init();

    let profile = STARTUP_ERA.profile();
    vga_text::init(profile.fg, profile.bg);
    vga_text::clear();

    let era_name = STARTUP_ERA.name();

    println!("TIME CAPSULE OS");
    println!("---------------");
    println!();
    println!("Era: {}", era_name);
    println!("Welcome to Time Capsule OS");

    serial_println!("Time Capsule OS booting in {} mode", era_name);
    serial_println!("Welcome to Time Capsule OS");

    loop {
        // SAFETY: The kernel has no scheduler yet, so halting in a loop is a
        // simple way to stay alive without burning CPU in a tight spin loop.
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
