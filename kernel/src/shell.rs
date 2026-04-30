//! Tiny line-based shell for the first interactive milestone.

use crate::{print, println};

const CURSOR_BLINK_TICKS: usize = 80_000;

pub fn run(prompt: &str) -> ! {
    print_prompt(prompt);
    draw_cursor();

    loop {
        cpu_relax();
    }
}

fn print_prompt(prompt: &str) {
    print!("{} ", prompt);
}

fn draw_cursor() {
    print!("_");
}

fn cpu_relax() {
    // SAFETY: `pause` is a CPU hint used inside the polling shell loop. It does
    // not access memory or devices; it just makes the busy-wait friendlier.
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}
