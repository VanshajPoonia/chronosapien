//! Tiny line-based shell for the first interactive milestone.

use crate::keyboard::{self, KeyEvent};
use crate::theme::EraProfile;
use crate::vga_text;
use crate::{print, println, serial_print, serial_println};

const COMMAND_BUFFER_CAPACITY: usize = 64;

pub fn run(profile: EraProfile) -> ! {
    let mut buffer = CommandBuffer::new();

    print_prompt(profile);

    loop {
        match keyboard::read_key() {
            Some(KeyEvent::Char(byte)) => {
                if buffer.push(byte) {
                    print!("{}", byte as char);
                    serial_print!("{}", byte as char);
                }
            }
            Some(KeyEvent::Backspace) => {
                if buffer.pop().is_some() {
                    vga_text::backspace();
                    serial_print!("\u{8} \u{8}");
                }
            }
            Some(KeyEvent::Enter) => {
                println!();
                serial_println!();

                execute_command(buffer.as_str(), profile);
                buffer.clear();
                print_prompt(profile);
            }
            None => cpu_relax(),
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

fn print_prompt(profile: EraProfile) {
    print!("{} ", profile.prompt);
    serial_print!("{} ", profile.prompt);
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
