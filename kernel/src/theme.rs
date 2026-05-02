#![allow(dead_code)]

//! Era-specific presentation data for boot and shell output.

use core::cell::UnsafeCell;

use crate::framebuffer::Color;

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
    pub screen_prompt: &'static str,
    pub boot_welcome: &'static str,
    pub fg: Color,
    pub bg: Color,
    pub top_bar_fg: Color,
    pub top_bar_bg: Color,
}

struct GlobalEra(UnsafeCell<Era>);

unsafe impl Sync for GlobalEra {}

static ACTIVE_ERA: GlobalEra = GlobalEra(UnsafeCell::new(Era::Eighties));

pub fn active_era() -> Era {
    // SAFETY: Chronosapian is currently single-core, and era changes happen
    // only from the shell loop. Future SMP or interrupt-driven UI changes
    // should replace this with a real synchronization primitive.
    unsafe { *ACTIVE_ERA.0.get() }
}

pub fn set_active_era(era: Era) {
    // SAFETY: Same reason as `active_era`: one synchronous execution context
    // owns all era reads and writes in this milestone.
    unsafe {
        *ACTIVE_ERA.0.get() = era;
    }
}

pub fn active_profile() -> EraProfile {
    active_era().profile()
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
                screen_prompt: "CHRONO/84>",
                boot_welcome: "CHRONOSAPIAN",
                fg: Color::GREEN,
                bg: Color::BLACK,
                top_bar_fg: Color::GREEN,
                top_bar_bg: Color::DARKER,
            },
            Era::Nineties => EraProfile {
                name: self.name(),
                prompt: "C:\\CHRONO>",
                screen_prompt: "C:\\CHRONO>",
                boot_welcome: "Chronosapian 95",
                fg: Color::BLACK,
                bg: Color::GRAY,
                top_bar_fg: Color::WHITE,
                top_bar_bg: Color::BLUE,
            },
            Era::TwoThousands => EraProfile {
                name: self.name(),
                prompt: "chrono@millennium:~$",
                screen_prompt: "chrono@millennium:~$",
                boot_welcome: "Chronosapian Millennium",
                fg: Color::BLACK,
                bg: Color::LIGHT_GRAY,
                top_bar_fg: Color::BLACK,
                top_bar_bg: Color::WHITE,
            },
            Era::Future => EraProfile {
                name: self.name(),
                prompt: "›",
                screen_prompt: ">",
                boot_welcome: "Chronosapian 2040",
                fg: Color::WHITE,
                bg: Color::DARK,
                top_bar_fg: Color::WHITE,
                top_bar_bg: Color::BLACK,
            },
        }
    }
}
