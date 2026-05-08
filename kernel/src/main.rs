//! Kernel entrypoint and startup flow for Chronosapian.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::cell::UnsafeCell;

mod apps;
mod ata;
mod boot;
mod console;
mod elf;
mod framebuffer;
mod fs;
mod gdt;
mod interrupts;
mod io;
mod keyboard;
mod memory;
mod mouse;
mod museum;
mod net;
mod panic;
mod pci;
mod pic;
mod process;
mod quest;
mod ring3;
mod sched;
mod serial;
mod shell;
mod sound;
mod syscall;
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

struct BootContextCell(UnsafeCell<Option<boot::BootContext>>);

unsafe impl Sync for BootContextCell {}

static BOOT_CONTEXT: BootContextCell = BootContextCell(UnsafeCell::new(None));

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let context = boot::context_from_bootloader(boot_info);
    boot_with_context(context)
}

#[no_mangle]
pub extern "sysv64" fn chrono_custom_entry(info: *const boot::ChronoBootInfo) -> ! {
    let context = unsafe { boot::context_from_custom(info) };
    boot_with_context(context)
}

fn boot_with_context(context: boot::BootContext) -> ! {
    let boot_context = unsafe {
        *BOOT_CONTEXT.0.get() = Some(context);
        (*BOOT_CONTEXT.0.get())
            .as_ref()
            .expect("boot context was just stored")
    };

    x86_64::instructions::interrupts::disable();

    serial::init();
    serial_println!("[CHRONO] boot start");
    serial_println!("[CHRONO] serial initialized");

    theme::set_active_era(STARTUP_ERA);
    let profile = theme::active_profile();

    console::init(boot_context.framebuffer, profile);
    serial_println!("[CHRONO] console initialized");

    gdt::init();
    syscall::init();
    interrupts::init();
    interrupts::trigger_test_breakpoint();
    memory::init(boot_context);
    fs::init();
    pic::init();
    timer::init();
    mouse::init();
    net::init();
    x86_64::instructions::interrupts::enable();

    let era_name = STARTUP_ERA.name();
    serial_println!("[CHRONO] active era: {}", era_name);
    sound::play_boot_chime(theme::active_era());
    keyboard::init();

    println!("{}", profile.boot_welcome);
    println!("Era: {}", profile.name);

    sched::init();
    serial_println!("[CHRONO] boot complete");

    shell::run()
}
