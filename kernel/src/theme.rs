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
    pub boot_welcome: &'static str,
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

    pub fn from_year(year: &str) -> Option<Self> {
        match year {
            "1984" => Some(Era::Eighties),
            "1995" => Some(Era::Nineties),
            "2007" => Some(Era::TwoThousands),
            "2040" => Some(Era::Future),
            _ => None,
        }
    }

    pub const fn profile(self) -> EraProfile {
        match self {
            Era::Eighties => EraProfile {
                name: self.name(),
                prompt: "CHRONO/84>",
                boot_welcome: "TIME CAPSULE OS",
                fg: Color::LightGreen,
                bg: Color::Black,
            },
            Era::Nineties => EraProfile {
                name: self.name(),
                prompt: "C:\\CHRONO>",
                boot_welcome: "Time Capsule OS 95",
                fg: Color::LightCyan,
                bg: Color::Blue,
            },
            Era::TwoThousands => EraProfile {
                name: self.name(),
                prompt: "chrono@millennium:~$",
                boot_welcome: "Time Capsule OS Millennium",
                fg: Color::Black,
                bg: Color::LightGray,
            },
            Era::Future => EraProfile {
                name: self.name(),
                prompt: ">",
                boot_welcome: "Time Capsule OS 2040",
                fg: Color::LightMagenta,
                bg: Color::Black,
            },
        }
    }
}
