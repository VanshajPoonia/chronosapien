//! Programmable Interval Timer setup and monotonic tick counter.
//!
//! The PIT has a fixed input clock. We program channel 0 as a divider so it
//! raises IRQ0 roughly 100 times per second.

use core::sync::atomic::{AtomicU64, Ordering};

pub const PIT_HZ: u64 = 100;

const PIT_INPUT_HZ: u32 = 1_193_182;
const PIT_COMMAND_PORT: u16 = 0x43;
const PIT_CHANNEL_0_PORT: u16 = 0x40;
const PIT_CHANNEL_0_MODE_3: u8 = 0b0011_0110;

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    let divisor = (PIT_INPUT_HZ / PIT_HZ as u32) as u16;
    let [low_byte, high_byte] = divisor.to_le_bytes();

    // SAFETY: Ports 0x43 and 0x40 are the PIT command register and channel 0
    // data register on QEMU's PC-compatible hardware.
    unsafe {
        outb(PIT_COMMAND_PORT, PIT_CHANNEL_0_MODE_3);
        outb(PIT_CHANNEL_0_PORT, low_byte);
        outb(PIT_CHANNEL_0_PORT, high_byte);
    }

    crate::serial_println!("[CHRONO] timer: PIT initialized at 100Hz");
}

pub fn handle_tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn uptime_seconds() -> u64 {
    ticks() / PIT_HZ
}

unsafe fn outb(port: u16, value: u8) {
    // SAFETY: The caller must ensure that the selected port belongs to the PIT.
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}
