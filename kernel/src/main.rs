//! Kernel entrypoint and startup flow for Time Capsule OS.

#![no_std]
#![no_main]

mod console;
mod keyboard;
mod panic;
mod serial;
mod theme;
mod vga_text;

use bootloader::{entry_point, BootInfo};
use keyboard::KeyEvent;
use theme::Era;

const STARTUP_ERA: Era = Era::Eighties;
const INPUT_BUFFER_CAPACITY: usize = 80;
const CURSOR_BLINK_TICKS: usize = 80_000;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    serial::init();
    serial_println!("[CHRONO] boot start");
    serial_println!("[CHRONO] serial initialized");

    let profile = STARTUP_ERA.profile();
    console::init(profile.fg, profile.bg);
    console::clear();
    serial_println!("[CHRONO] console initialized");

    let era_name = STARTUP_ERA.name();
    serial_println!("[CHRONO] active era: {}", era_name);
    keyboard::init();

    println!("{}", profile.boot_welcome);
    println!("Era: {}", profile.name);

    serial_println!("[CHRONO] boot complete");

    read_keyboard_input(profile.prompt)
}

fn read_keyboard_input(prompt: &str) -> ! {
    let mut buffer = InputBuffer::new();
    let mut cursor_visible = true;
    let mut idle_ticks = 0;

    print_prompt(prompt);
    draw_cursor();

    loop {
        match keyboard::read_key() {
            Some(KeyEvent::Char(byte)) => {
                hide_cursor(&mut cursor_visible);

                if buffer.push(byte) {
                    print!("{}", byte as char);
                    serial_println!("[CHRONO] key: {}", byte as char);
                } else {
                    serial_println!("[CHRONO] input buffer full");
                }

                show_cursor(&mut cursor_visible);
                idle_ticks = 0;
            }
            Some(KeyEvent::Backspace) => {
                hide_cursor(&mut cursor_visible);

                if buffer.pop().is_some() {
                    console::backspace();
                    serial_println!("[CHRONO] key: backspace");
                }

                show_cursor(&mut cursor_visible);
                idle_ticks = 0;
            }
            Some(KeyEvent::Enter) => {
                hide_cursor(&mut cursor_visible);
                println!();
                serial_println!("[CHRONO] key: enter");
                serial_println!("[CHRONO] line submitted: {}", buffer.as_str());
                buffer.clear();
                print_prompt(prompt);
                show_cursor(&mut cursor_visible);
                idle_ticks = 0;
            }
            None => {
                idle_ticks += 1;

                if idle_ticks >= CURSOR_BLINK_TICKS {
                    toggle_cursor(&mut cursor_visible);
                    idle_ticks = 0;
                }

                cpu_relax();
            }
        }
    }
}

struct InputBuffer {
    bytes: [u8; INPUT_BUFFER_CAPACITY],
    len: usize,
}

impl InputBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; INPUT_BUFFER_CAPACITY],
            len: 0,
        }
    }

    fn push(&mut self, byte: u8) -> bool {
        if self.len >= self.bytes.len() {
            return false;
        }

        self.bytes[self.len] = byte;
        self.len += 1;
        true
    }

    fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        Some(self.bytes[self.len])
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn as_str(&self) -> &str {
        // SAFETY: The keyboard decoder only returns printable ASCII bytes, and
        // ASCII is always valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }
}

fn print_prompt(prompt: &str) {
    print!("{} ", prompt);
}

fn draw_cursor() {
    print!("_");
}

fn erase_cursor() {
    console::backspace();
}

fn show_cursor(cursor_visible: &mut bool) {
    if !*cursor_visible {
        draw_cursor();
        *cursor_visible = true;
    }
}

fn hide_cursor(cursor_visible: &mut bool) {
    if *cursor_visible {
        erase_cursor();
        *cursor_visible = false;
    }
}

fn toggle_cursor(cursor_visible: &mut bool) {
    if *cursor_visible {
        erase_cursor();
        *cursor_visible = false;
    } else {
        draw_cursor();
        *cursor_visible = true;
    }
}

fn cpu_relax() {
    // SAFETY: `pause` is a CPU hint used inside a polling loop. It does not
    // access memory or devices; it just makes the busy-wait friendlier to the
    // processor while we avoid interrupts for this milestone.
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}
