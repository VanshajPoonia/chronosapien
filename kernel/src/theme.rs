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
    pub banner: &'static str,
    pub prompt: &'static str,
    pub fg: Color,
    pub bg: Color,
    pub boot_text: &'static str,
}

impl Era {
    pub const fn profile(self) -> EraProfile {
        match self {
            Era::Eighties => EraProfile {
                name: "1980s",
                banner: "== Retro boot channel open ==",
                prompt: "C:\\TIME>",
                fg: Color::LightGreen,
                bg: Color::Black,
                boot_text: "[1980s] CRT phosphor warming up...",
            },
            Era::Nineties => EraProfile {
                name: "1990s",
                banner: "== Dial-up dreams and beige boxes ==",
                prompt: "tcos_95>",
                fg: Color::LightCyan,
                bg: Color::Blue,
                boot_text: "[1990s] Initializing cyberspace terminal...",
            },
            Era::TwoThousands => EraProfile {
                name: "2000s",
                banner: "== Broadband optimism engaged ==",
                prompt: "tc-os$",
                fg: Color::Black,
                bg: Color::LightGray,
                boot_text: "[2000s] Starting millennium shell services...",
            },
            Era::Future => EraProfile {
                name: "future",
                banner: "== Chrono-systems synchronized ==",
                prompt: "future::tc>",
                fg: Color::LightMagenta,
                bg: Color::Black,
                boot_text: "[future] Loading speculative interface...",
            },
        }
    }
}
