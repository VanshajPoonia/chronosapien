#![allow(dead_code)]

//! Era-specific presentation data for boot, shell, sound, and windows.

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
    pub boot_lines: &'static [&'static str],
    pub boot_chime: &'static [BootTone],
    pub fg: Color,
    pub bg: Color,
    pub top_bar_fg: Color,
    pub top_bar_bg: Color,
    pub font_effect: FontEffect,
    pub scanline_overlay: bool,
    pub bottom_taskbar: bool,
    pub cursor_glyph: &'static str,
    pub cursor_blink_ticks: usize,
    pub render_log: &'static str,
    pub window: WindowProfile,
    pub text_frame: TextFrameProfile,
    pub sysinfo: SysinfoProfile,
}

#[derive(Clone, Copy, Debug)]
pub struct BootTone {
    pub frequency_hz: u32,
    pub duration_ms: u64,
    pub rest_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontEffect {
    Terminal,
    Chunky,
    Smooth,
    Thin,
}

#[derive(Clone, Copy, Debug)]
pub struct WindowProfile {
    pub body_bg: Color,
    pub body_fg: Color,
    pub title_bg: Color,
    pub title_bg_end: Color,
    pub title_fg: Color,
    pub border: Color,
    pub highlight: Color,
    pub shadow: Color,
    pub title_fill: TitleFill,
    pub border_kind: WindowBorderKind,
    pub corner_radius: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleFill {
    None,
    Solid,
    Gradient,
    Frosted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowBorderKind {
    Flat,
    ThreeD,
    Minimal,
}

#[derive(Clone, Copy, Debug)]
pub struct TextFrameProfile {
    pub top_left: &'static str,
    pub top_fill: &'static str,
    pub top_right: &'static str,
    pub side_left: &'static str,
    pub side_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_fill: &'static str,
    pub bottom_right: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct SysinfoProfile {
    pub header: &'static str,
    pub os_label: &'static str,
    pub era_label: &'static str,
    pub uptime_label: &'static str,
    pub mem_label: &'static str,
    pub separator: &'static str,
    pub compact: bool,
}

struct GlobalEra(UnsafeCell<Era>);

unsafe impl Sync for GlobalEra {}

static ACTIVE_ERA: GlobalEra = GlobalEra(UnsafeCell::new(Era::Eighties));

const FRAME_1984: TextFrameProfile = TextFrameProfile {
    top_left: "+",
    top_fill: "=",
    top_right: "+",
    side_left: "|",
    side_right: "|",
    bottom_left: "+",
    bottom_fill: "=",
    bottom_right: "+",
};
const FRAME_1995: TextFrameProfile = TextFrameProfile {
    top_left: "+",
    top_fill: "-",
    top_right: "+",
    side_left: "|",
    side_right: "|",
    bottom_left: "+",
    bottom_fill: "-",
    bottom_right: "+",
};
const FRAME_2007: TextFrameProfile = TextFrameProfile {
    top_left: "[",
    top_fill: "-",
    top_right: "]",
    side_left: "|",
    side_right: "|",
    bottom_left: "[",
    bottom_fill: "-",
    bottom_right: "]",
};
const FRAME_2040: TextFrameProfile = TextFrameProfile {
    top_left: "",
    top_fill: "-",
    top_right: "",
    side_left: "|",
    side_right: "|",
    bottom_left: "",
    bottom_fill: "-",
    bottom_right: "",
};

const BOOT_1984: [&str; 5] = [
    "IBM Personal Computer",
    "640K OK",
    "Keyboard... OK",
    "Fixed Disk... OK",
    "ChronoOS BASIC ROM shadowed",
];
const BOOT_1995: [&str; 4] = [
    "Starting Chronosapian 95...",
    "HIMEM.SYS loaded",
    "Mouse driver initialized",
    "Program Manager ready",
];
const BOOT_2007: [&str; 4] = [
    "launchd: Chronosapian services",
    "Quartz framebuffer online",
    "Airport: not configured",
    "Dock compositor ready",
];
const BOOT_2040: [&str; 3] = [
    "chrono core online",
    "visual surface: line mode",
    "ambient state: quiet",
];

const CHIME_1984: [BootTone; 3] = [
    BootTone {
        frequency_hz: 880,
        duration_ms: 90,
        rest_ms: 35,
    },
    BootTone {
        frequency_hz: 660,
        duration_ms: 90,
        rest_ms: 35,
    },
    BootTone {
        frequency_hz: 440,
        duration_ms: 140,
        rest_ms: 0,
    },
];
const CHIME_1995: [BootTone; 5] = [
    BootTone {
        frequency_hz: 523,
        duration_ms: 70,
        rest_ms: 25,
    },
    BootTone {
        frequency_hz: 659,
        duration_ms: 70,
        rest_ms: 25,
    },
    BootTone {
        frequency_hz: 784,
        duration_ms: 90,
        rest_ms: 25,
    },
    BootTone {
        frequency_hz: 659,
        duration_ms: 80,
        rest_ms: 25,
    },
    BootTone {
        frequency_hz: 1046,
        duration_ms: 140,
        rest_ms: 0,
    },
];
const CHIME_2007: [BootTone; 3] = [
    BootTone {
        frequency_hz: 523,
        duration_ms: 80,
        rest_ms: 35,
    },
    BootTone {
        frequency_hz: 659,
        duration_ms: 80,
        rest_ms: 35,
    },
    BootTone {
        frequency_hz: 784,
        duration_ms: 120,
        rest_ms: 0,
    },
];
const CHIME_2040: [BootTone; 1] = [BootTone {
    frequency_hz: 1760,
    duration_ms: 180,
    rest_ms: 0,
}];

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
                boot_lines: &BOOT_1984,
                boot_chime: &CHIME_1984,
                fg: Color::PHOSPHOR,
                bg: Color::BLACK,
                top_bar_fg: Color::PHOSPHOR,
                top_bar_bg: Color::DARKER,
                font_effect: FontEffect::Terminal,
                scanline_overlay: true,
                bottom_taskbar: false,
                cursor_glyph: "_",
                cursor_blink_ticks: 35_000,
                render_log: "[CHRONO] theme: rendering 1984 scanlines",
                window: WindowProfile {
                    body_bg: Color::BLACK,
                    body_fg: Color::PHOSPHOR,
                    title_bg: Color::BLACK,
                    title_bg_end: Color::BLACK,
                    title_fg: Color::PHOSPHOR,
                    border: Color::PHOSPHOR,
                    highlight: Color::PHOSPHOR_DIM,
                    shadow: Color::PHOSPHOR_DIM,
                    title_fill: TitleFill::None,
                    border_kind: WindowBorderKind::Flat,
                    corner_radius: 0,
                },
                text_frame: FRAME_1984,
                sysinfo: SysinfoProfile {
                    header: "== CHRONO SYSINFO 1984 ==",
                    os_label: "OS",
                    era_label: "ERA",
                    uptime_label: "UPTIME",
                    mem_label: "MEM",
                    separator: "......",
                    compact: false,
                },
            },
            Era::Nineties => EraProfile {
                name: self.name(),
                prompt: "C:\\CHRONO>",
                screen_prompt: "C:\\CHRONO>",
                boot_welcome: "Chronosapian 95",
                boot_lines: &BOOT_1995,
                boot_chime: &CHIME_1995,
                fg: Color::BLACK,
                bg: Color::GRAY,
                top_bar_fg: Color::WHITE,
                top_bar_bg: Color::BLUE,
                font_effect: FontEffect::Chunky,
                scanline_overlay: false,
                bottom_taskbar: false,
                cursor_glyph: "_",
                cursor_blink_ticks: 80_000,
                render_log: "[CHRONO] theme: rendering 1995 16-color chrome",
                window: WindowProfile {
                    body_bg: Color::LIGHT_GRAY,
                    body_fg: Color::BLACK,
                    title_bg: Color::WHITE,
                    title_bg_end: Color::GRAY,
                    title_fg: Color::BLACK,
                    border: Color::GRAY,
                    highlight: Color::WHITE,
                    shadow: Color::DARK,
                    title_fill: TitleFill::Gradient,
                    border_kind: WindowBorderKind::ThreeD,
                    corner_radius: 0,
                },
                text_frame: FRAME_1995,
                sysinfo: SysinfoProfile {
                    header: "C:\\CHRONO\\SYSINFO.EXE",
                    os_label: "OS",
                    era_label: "ERA",
                    uptime_label: "UPTIME",
                    mem_label: "MEM USED",
                    separator: "      : ",
                    compact: false,
                },
            },
            Era::TwoThousands => EraProfile {
                name: self.name(),
                prompt: "chrono@millennium:~$",
                screen_prompt: "chrono@millennium:~$",
                boot_welcome: "Chronosapian Millennium",
                boot_lines: &BOOT_2007,
                boot_chime: &CHIME_2007,
                fg: Color::BLACK,
                bg: Color::LIGHT_GRAY,
                top_bar_fg: Color::BLACK,
                top_bar_bg: Color::WHITE,
                font_effect: FontEffect::Smooth,
                scanline_overlay: false,
                bottom_taskbar: true,
                cursor_glyph: "_",
                cursor_blink_ticks: 70_000,
                render_log: "[CHRONO] theme: rendering 2007 glass taskbar",
                window: WindowProfile {
                    body_bg: Color::LIGHT_GRAY,
                    body_fg: Color::BLACK,
                    title_bg: Color::WHITE,
                    title_bg_end: Color::CYAN,
                    title_fg: Color::BLACK,
                    border: Color::DARK,
                    highlight: Color::WHITE,
                    shadow: Color::DARK,
                    title_fill: TitleFill::Frosted,
                    border_kind: WindowBorderKind::Flat,
                    corner_radius: 6,
                },
                text_frame: FRAME_2007,
                sysinfo: SysinfoProfile {
                    header: "chrono sysinfo",
                    os_label: "os",
                    era_label: "era",
                    uptime_label: "uptime",
                    mem_label: "mem_used_kb",
                    separator: ": ",
                    compact: false,
                },
            },
            Era::Future => EraProfile {
                name: self.name(),
                prompt: "›",
                screen_prompt: ">",
                boot_welcome: "Chronosapian 2040",
                boot_lines: &BOOT_2040,
                boot_chime: &CHIME_2040,
                fg: Color::WHITE,
                bg: Color::BLACK,
                top_bar_fg: Color::WHITE,
                top_bar_bg: Color::BLACK,
                font_effect: FontEffect::Thin,
                scanline_overlay: false,
                bottom_taskbar: false,
                cursor_glyph: "_",
                cursor_blink_ticks: 55_000,
                render_log: "[CHRONO] theme: rendering 2040 line interface",
                window: WindowProfile {
                    body_bg: Color::BLACK,
                    body_fg: Color::WHITE,
                    title_bg: Color::BLACK,
                    title_bg_end: Color::BLACK,
                    title_fg: Color::WHITE,
                    border: Color::WHITE,
                    highlight: Color::WHITE,
                    shadow: Color::BLACK,
                    title_fill: TitleFill::None,
                    border_kind: WindowBorderKind::Minimal,
                    corner_radius: 0,
                },
                text_frame: FRAME_2040,
                sysinfo: SysinfoProfile {
                    header: "[chrono::sysinfo]",
                    os_label: "os",
                    era_label: "era",
                    uptime_label: "uptime_s",
                    mem_label: "mem_kb",
                    separator: "=",
                    compact: true,
                },
            },
        }
    }
}
