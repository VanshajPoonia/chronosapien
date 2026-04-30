//! Tiny line-based shell for the first interactive milestone.

use crate::console;
use crate::keyboard::{self, KeyEvent};
use crate::theme::EraProfile;
use crate::{print, println, serial_println};

const COMMAND_BUFFER_CAPACITY: usize = 80;
const CURSOR_BLINK_TICKS: usize = 80_000;
const PROMPT: &str = "TCOS/84>";

pub fn run(profile: EraProfile) -> ! {
    let mut buffer = CommandBuffer::new();
    let mut cursor_visible = true;
    let mut idle_ticks = 0;

    print_prompt();
    draw_cursor();

    loop {
        match keyboard::read_key() {
            Some(KeyEvent::Char(byte)) => {
                hide_cursor(&mut cursor_visible);

                if buffer.push(byte) {
                    print!("{}", byte as char);
                } else {
                    serial_println!("[TCOS] input buffer full");
                }

                show_cursor(&mut cursor_visible);
                idle_ticks = 0;
            }
            Some(KeyEvent::Backspace) => {
                hide_cursor(&mut cursor_visible);

                if buffer.pop().is_some() {
                    console::backspace();
                }

                show_cursor(&mut cursor_visible);
                idle_ticks = 0;
            }
            Some(KeyEvent::Enter) => {
                hide_cursor(&mut cursor_visible);
                println!();
                serial_println!();

                execute_command(buffer.as_str(), profile);
                buffer.clear();
                print_prompt();
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

struct CommandBuffer {
    bytes: [u8; COMMAND_BUFFER_CAPACITY],
    len: usize,
}

impl CommandBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; COMMAND_BUFFER_CAPACITY],
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
        // SAFETY: The command buffer only accepts ASCII bytes from the keyboard
        // decoder, so the occupied prefix is always valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }
}

fn execute_command(command: &str, profile: EraProfile) {
    match command {
        "" => {}
        "help" => print_help(),
        "clear" => clear_screen(profile),
        "about" => print_about(profile),
        _ => {
            println!("Unknown command: {}", command);
            println!("Try 'help'.");
            serial_println!("Unknown command: {}", command);
            serial_println!("Try 'help'.");
        }
    }
}

fn print_prompt() {
    print!("{} ", PROMPT);
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

fn print_help() {
    println!("Available commands:");
    println!("  help  - show the command list");
    println!("  clear - clear the screen");
    println!("  about - show information about Time Capsule OS");

    serial_println!("Available commands:");
    serial_println!("  help  - show the command list");
    serial_println!("  clear - clear the screen");
    serial_println!("  about - show information about Time Capsule OS");
}

fn clear_screen(profile: EraProfile) {
    vga_text::clear();
    println!("Welcome to Time Capsule OS");
    println!("{}", profile.banner);
    println!("Current era: {}", profile.name);

    serial_println!("Screen cleared.");
}

fn print_about(profile: EraProfile) {
    println!("Time Capsule OS is a learning-first Rust hobby kernel.");
    println!("Current era profile: {}", profile.name);
    println!("Future shell expansion will add era switching.");

    serial_println!("Time Capsule OS is a learning-first Rust hobby kernel.");
    serial_println!("Current era profile: {}", profile.name);
    serial_println!("Future shell expansion will add era switching.");
}

fn cpu_relax() {
    // SAFETY: `pause` is a CPU hint for tight polling loops. We use it here
    // because keyboard input is polled without interrupts in this milestone.
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}
