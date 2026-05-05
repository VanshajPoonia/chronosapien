//! Beginner-friendly text output layer for the kernel.
//!
//! The console module is the public place for screen printing. The low-level
//! framebuffer details stay in `framebuffer`, so the rest of the kernel can use
//! familiar `print!` and `println!` macros.

use core::fmt;

use crate::framebuffer::{self, Framebuffer};
use crate::theme::EraProfile;

pub fn init(framebuffer: Framebuffer, profile: EraProfile) {
    framebuffer::init(framebuffer, profile);
}

pub fn clear() {
    framebuffer::clear();
}

pub fn backspace() {
    framebuffer::backspace();
}

pub fn set_theme(profile: EraProfile) {
    framebuffer::set_theme(profile);
}

pub fn refresh_top_bar() {
    framebuffer::refresh_top_bar();
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    framebuffer::print(args);
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
