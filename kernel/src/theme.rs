#![allow(dead_code)]

//! Era-specific presentation data for boot and shell output.

use crate::vga_text::color::Color;

#[derive(Clone, Copy, Debug)]
pub enum Era {
    Eighties,
    Nineties,
    TwoThousands,
    Future,
}

#[derive(Clone, Copy, Debug)]
pub struct EraProfile {
    pub name: &'static str,
    pub prompt: &'static str,
    pub fg: Color,
    pub bg: Color,
}

impl Era {
    pub const fn name(self) -> &'static str {
        match self {
            Era::Eighties => "1984",
            Era::Nineties => "1995",
            Era::TwoThousands => "2007",
            Era::Future => "2040",
        }
    }

    pub const fn profile(self) -> EraProfile {
        match self {
            Era::Eighties => EraProfile {
                name: self.name(),
                prompt: "TCOS/84>",
                fg: Color::LightGreen,
                bg: Color::Black,
            },
            Era::Nineties => EraProfile {
                name: self.name(),
                prompt: "C:\\TCOS>",
                fg: Color::LightCyan,
                bg: Color::Blue,
            },
            Era::TwoThousands => EraProfile {
                name: self.name(),
                prompt: "tcos@millennium:~$",
                fg: Color::Black,
                bg: Color::LightGray,
            },
            Era::Future => EraProfile {
                name: self.name(),
                prompt: ">",
                fg: Color::LightMagenta,
                bg: Color::Black,
            },
        }
    }
}
