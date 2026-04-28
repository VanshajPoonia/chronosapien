use core::fmt::{self, Write};

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_BUFFER_ADDRESS: usize = 0xb8000;
const COLOR_LIGHT_GREEN_ON_BLACK: u8 = 0x0a;

#[repr(C)]
#[derive(Clone, Copy)]
struct ScreenChar {
    ascii_character: u8,
    color_code: u8,
}

pub struct Writer {
    row: usize,
    column: usize,
    color_code: u8,
}

impl Writer {
    const fn new() -> Self {
        Self {
            row: 0,
            column: 0,
            color_code: COLOR_LIGHT_GREEN_ON_BLACK,
        }
    }

    pub fn write_string(&mut self, text: &str) {
        for byte in text.bytes() {
            match byte {
                b'\n' => self.new_line(),
                0x20..=0x7e => self.write_byte(byte),
                _ => self.write_byte(b'?'),
            }
        }
    }

    fn write_byte(&mut self, byte: u8) {
        if self.column >= BUFFER_WIDTH {
            self.new_line();
        }

        let character = ScreenChar {
            ascii_character: byte,
            color_code: self.color_code,
        };

        self.write_screen_char(self.row, self.column, character);
        self.column += 1;
    }

    fn new_line(&mut self) {
        self.column = 0;

        if self.row + 1 < BUFFER_HEIGHT {
            self.row += 1;
        }
    }

    fn write_screen_char(&mut self, row: usize, column: usize, character: ScreenChar) {
        let index = row * BUFFER_WIDTH + column;
        let buffer = VGA_BUFFER_ADDRESS as *mut ScreenChar;

        // SAFETY: In VGA text mode, physical memory at 0xb8000 is the screen's
        // text buffer. Each cell is two bytes: an ASCII character and its color.
        // The row/column values are kept inside the 80x25 text area before this
        // write, so the pointer stays within the VGA buffer.
        unsafe {
            buffer.add(index).write_volatile(character);
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.write_string(text);
        Ok(())
    }
}

static mut WRITER: Writer = Writer::new();

pub fn _print(args: fmt::Arguments) {
    // SAFETY: This early kernel has only one CPU path and interrupts are not
    // enabled yet, so no other code can access WRITER at the same time.
    unsafe {
        WRITER.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::vga::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($format:expr) => {
        $crate::print!(concat!($format, "\n"))
    };
    ($format:expr, $($arg:tt)*) => {
        $crate::print!(concat!($format, "\n"), $($arg)*)
    };
}
