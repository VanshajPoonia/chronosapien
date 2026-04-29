//! Minimal VGA text output for the first terminal-first kernel milestone.

pub mod color;
mod writer;

pub use writer::{clear, init};
pub use writer::print;
