//! Panic handling for a `no_std` kernel.

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // In a `no_std` kernel there is no operating system to unwind into, so the
    // panic path must report the error itself and then stop execution safely.
    crate::serial_println!("[panic] {}", info);
    crate::println!();
    crate::println!("[panic] {}", info);

    loop {
        // SAFETY: After a panic the kernel should stop doing useful work.
        // Halting avoids spinning the CPU at 100% while keeping the machine in
        // a stable, inspectable state under QEMU.
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
