//! Tiny line-based shell for the first interactive milestone.

use crate::console;
use crate::keyboard::{self, KeyEvent};
use crate::theme::Era;
use crate::{print, println, serial_println};

const COMMAND_BUFFER_CAPACITY: usize = 80;
const CURSOR_BLINK_TICKS: usize = 80_000;
const PROMPT: &str = "TCOS/84>";
const RESET_COMMAND_PORT: u16 = 0x64;
const CPU_RESET_COMMAND: u8 = 0xFE;

pub fn run(startup_era: Era) -> ! {
    let state = ShellState::new(startup_era);
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

                execute_command(buffer.as_str());
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

struct ShellState {
    active_era: Era,
}

impl ShellState {
    const fn new(active_era: Era) -> Self {
        Self { active_era }
    }

    fn active_era(&self) -> Era {
        self.active_era
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

fn execute_command(command: &str) {
    let command = command.trim();

    if !command.is_empty() {
        serial_println!("[TCOS] command: {}", command);
    }

    match command {
        "" => {}
        "help" => print_help(),
        "clear" => console::clear(),
        "about" => print_about(),
        "reboot" => reboot(),
        _ => println!("Unknown command: {}", command),
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
    println!("Commands: help, clear, about, reboot");
}

fn print_about() {
    println!("Time Capsule OS | Era: 1984 | v0.1");
}

fn reboot() -> ! {
    serial_println!("[TCOS] reboot requested");

    // SAFETY: Port 0x64 is the PS/2 controller command port on the
    // PC-compatible machine QEMU emulates. Command 0xFE requests a CPU reset.
    unsafe {
        outb(RESET_COMMAND_PORT, CPU_RESET_COMMAND);
    }

    halt_forever()
}

fn cpu_relax() {
    // SAFETY: `pause` is a CPU hint for tight polling loops. We use it here
    // because keyboard input is polled without interrupts in this milestone.
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}

fn halt_forever() -> ! {
    loop {
        // SAFETY: `hlt` stops the CPU until the next external interrupt. This
        // fallback is only reached if the reboot command does not reset.
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

unsafe fn outb(port: u16, value: u8) {
    // SAFETY: The caller must ensure `port` belongs to the intended hardware
    // device and `value` is a valid command for that port.
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}
