#![allow(dead_code)]

//! Era-specific presentation data for the first boot experience.

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
    pub fg: Color,
    pub bg: Color,
}

impl Era {
    pub const fn name(self) -> &'static str {
        match self {
            Era::Eighties => "1984",
            Era::Nineties => "1990s",
            Era::TwoThousands => "2000s",
            Era::Future => "future",
        }
    }

    pub const fn profile(self) -> EraProfile {
        match self {
            Era::Eighties => EraProfile {
                name: self.name(),
                fg: Color::LightGreen,
                bg: Color::Black,
            },
            Era::Nineties => EraProfile {
                name: self.name(),
                fg: Color::LightCyan,
                bg: Color::Blue,
            },
            Era::TwoThousands => EraProfile {
                name: self.name(),
                fg: Color::Black,
                bg: Color::LightGray,
            },
            Era::Future => EraProfile {
                name: self.name(),
                fg: Color::LightMagenta,
                bg: Color::Black,
            },
        }
    }
}
