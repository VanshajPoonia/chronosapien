//! Beginner-friendly text output layer for the kernel.
//!
//! The console module is the public place for screen printing. The low-level
//! VGA details stay in `vga_text`, so the rest of the kernel can use familiar
//! `print!` and `println!` macros.

use core::fmt;

use crate::vga_text;
use crate::vga_text::color::Color;

pub fn init(foreground: Color, background: Color) {
    vga_text::init(foreground, background);
}

pub fn clear() {
    vga_text::clear();
}

pub fn backspace() {
    vga_text::backspace();
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    vga_text::print(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($fmt:expr) => {
        $crate::print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n"), $($arg)*)
    };
}
