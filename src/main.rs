#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod theme;
mod vga;

use theme::Era;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let current_era = Era::NineteenEightyFour;

    println!("TIME CAPSULE OS");
    println!("---------------");
    println!();
    println!("Era: {}", current_era.name());
    println!("Welcome to Time Capsule OS");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Kernel panic: {}", info);

    loop {}
}
