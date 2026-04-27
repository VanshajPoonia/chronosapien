//! Minimal VGA text output for the first terminal-first kernel milestone.

pub mod color;
mod writer;

pub use writer::{clear, init};

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::vga_text::_print(format_args!($($arg)*))
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

pub fn _print(args: core::fmt::Arguments) {
    writer::print(args);
}
