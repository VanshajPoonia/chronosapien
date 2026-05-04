//! Kernel entrypoint and startup flow for Chronosapian.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod apps;
mod console;
mod framebuffer;
mod fs;
mod gdt;
mod interrupts;
mod keyboard;
mod memory;
mod mouse;
mod panic;
mod pic;
mod sched;
mod serial;
mod shell;
mod theme;
mod timer;
mod wm;

use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, BootInfo};
use theme::Era;

const STARTUP_ERA: Era = Era::Eighties;

static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    serial::init();
    serial_println!("[CHRONO] boot start");
    serial_println!("[CHRONO] serial initialized");

    theme::set_active_era(STARTUP_ERA);
    let profile = theme::active_profile();
    let Some(framebuffer) = boot_info.framebuffer.as_mut() else {
        serial_println!("[CHRONO] fb: missing framebuffer");
        panic!("bootloader did not provide a framebuffer");
    };

    console::init(framebuffer, profile);
    serial_println!("[CHRONO] console initialized");

    gdt::init();
    interrupts::init();
    interrupts::trigger_test_breakpoint();
    memory::init(boot_info);
    pic::init();
    timer::init();
    mouse::init();
    x86_64::instructions::interrupts::enable();

    let era_name = STARTUP_ERA.name();
    serial_println!("[CHRONO] active era: {}", era_name);
    keyboard::init();

    println!("{}", profile.boot_welcome);
    println!("Era: {}", profile.name);

    sched::init();
    serial_println!("[CHRONO] boot complete");

    shell::run()
}
