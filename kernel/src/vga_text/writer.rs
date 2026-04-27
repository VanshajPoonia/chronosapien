//! Direct VGA buffer writer for simple text output.

use core::cell::UnsafeCell;
use core::fmt;

use super::color::{Color, ColorCode};

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
const VGA_BUFFER_ADDRESS: usize = 0xb8000;

#[repr(C)]
#[derive(Clone, Copy)]
struct ScreenChar {
    ascii_character: u8,
    color_code: u8,
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer_address: usize,
}

impl Writer {
    const fn new() -> Self {
        Self {
            column_position: 0,
            color_code: ColorCode::new(Color::LightGray, Color::Black),
            buffer_address: VGA_BUFFER_ADDRESS,
        }
    }

    fn set_theme(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code.value(),
        };

        for col in 0..BUFFER_WIDTH {
            self.write_cell(row, col, blank);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let character = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code.value(),
                };

                self.write_cell(row, col, character);
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.read_cell(row, col);
                self.write_cell(row - 1, col, character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn write_cell(&mut self, row: usize, col: usize, character: ScreenChar) {
        let index = row * BUFFER_WIDTH + col;
        let ptr = self.buffer_address as *mut ScreenChar;

        // SAFETY: `0xb8000` is the VGA text buffer address in the boot mode
        // used by QEMU/BIOS here. The index is kept within the 80x25 buffer,
        // and writes happen through this one global writer in our current
        // single-context kernel stage.
        unsafe {
            core::ptr::write_volatile(ptr.add(index), character);
        }
    }

    fn read_cell(&self, row: usize, col: usize) -> ScreenChar {
        let index = row * BUFFER_WIDTH + col;
        let ptr = self.buffer_address as *const ScreenChar;

        // SAFETY: Reads target the same bounded VGA text buffer region as
        // `write_cell`. We use volatile access so the compiler does not remove
        // or reorder reads from this memory-mapped hardware buffer.
        unsafe { core::ptr::read_volatile(ptr.add(index)) }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }

        Ok(())
    }
}

struct GlobalWriter(UnsafeCell<Writer>);

unsafe impl Sync for GlobalWriter {}

static WRITER: GlobalWriter = GlobalWriter(UnsafeCell::new(Writer::new()));

pub fn init(foreground: Color, background: Color) {
    // SAFETY: This early kernel runs in a single execution context before any
    // interrupt-driven reentrancy is introduced, so exclusive access to this
    // global writer is valid for now.
    unsafe {
        (*WRITER.0.get()).set_theme(foreground, background);
    }
}

pub fn clear() {
    // SAFETY: Same reasoning as in `init`: VGA output is single-threaded here.
    unsafe {
        (*WRITER.0.get()).clear();
    }
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;

    // SAFETY: Same reasoning as in `init`: printing happens synchronously in a
    // single execution context during this first kernel stage.
    unsafe {
        let _ = (*WRITER.0.get()).write_fmt(args);
    }
}
