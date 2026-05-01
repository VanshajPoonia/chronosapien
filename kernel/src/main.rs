//! Kernel entrypoint and startup flow for Time Capsule OS.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod apps;
mod console;
mod gdt;
mod interrupts;
mod keyboard;
mod memory;
mod panic;
mod pic;
mod serial;
mod shell;
mod theme;
mod timer;
mod vga_text;

use bootloader::{entry_point, BootInfo};
use theme::Era;

const STARTUP_ERA: Era = Era::Eighties;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    serial::init();
    serial_println!("[CHRONO] boot start");
    serial_println!("[CHRONO] serial initialized");

    let profile = STARTUP_ERA.profile();
    console::init(profile.fg, profile.bg);
    console::clear();
    serial_println!("[CHRONO] console initialized");

    gdt::init();
    interrupts::init();
    interrupts::trigger_test_breakpoint();
    memory::init(boot_info);
    pic::init();
    timer::init();
    x86_64::instructions::interrupts::enable();

    let era_name = STARTUP_ERA.name();
    serial_println!("[CHRONO] active era: {}", era_name);
    theme::set_active_era(STARTUP_ERA);
    keyboard::init();

    println!("{}", profile.boot_welcome);
    println!("Era: {}", profile.name);

    serial_println!("[CHRONO] boot complete");

    shell::run()
}
