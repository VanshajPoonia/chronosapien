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
    pub const fn profile(self) -> EraProfile {
        match self {
            Era::Eighties => EraProfile {
                name: "1984",
                fg: Color::LightGreen,
                bg: Color::Black,
            },
            Era::Nineties => EraProfile {
                name: "1990s",
                fg: Color::LightCyan,
                bg: Color::Blue,
            },
            Era::TwoThousands => EraProfile {
                name: "2000s",
                fg: Color::Black,
                bg: Color::LightGray,
            },
            Era::Future => EraProfile {
                name: "future",
                fg: Color::LightMagenta,
                bg: Color::Black,
            },
        }
    }
}
