//! Tiny fixed-capacity framebuffer window manager.

use core::cell::UnsafeCell;
use core::fmt::{self, Write};

use crate::framebuffer::{self, Color};
use crate::mouse::MouseEvent;
use crate::theme::Era;

const MAX_WINDOWS: usize = 4;
pub const WINDOW_CONTENT_CAPACITY: usize = 512;
const TITLE_BAR_HEIGHT: usize = 16;
const CLOSE_SIZE: usize = 8;
const WINDOW_PADDING: usize = 8;

#[derive(Clone, Copy, Eq, PartialEq)]
enum WindowKind {
    Notes,
    Sysinfo,
}

impl WindowKind {
    const fn title(self) -> &'static str {
        match self {
            Self::Notes => "notes",
            Self::Sysinfo => "sysinfo",
        }
    }

    const fn log_name(self) -> &'static str {
        self.title()
    }
}

#[derive(Clone, Copy)]
pub struct Window {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub title: &'static str,
    pub content: [u8; WINDOW_CONTENT_CAPACITY],
    pub content_len: usize,
    kind: WindowKind,
}

impl Window {
    const fn empty() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            title: "",
            content: [0; WINDOW_CONTENT_CAPACITY],
            content_len: 0,
            kind: WindowKind::Notes,
        }
    }
}

#[derive(Clone, Copy)]
struct DragState {
    window_index: usize,
    offset_x: usize,
    offset_y: usize,
}

struct WindowManager {
    windows: [Window; MAX_WINDOWS],
    count: usize,
    drag: Option<DragState>,
}

impl WindowManager {
    const fn new() -> Self {
        Self {
            windows: [Window::empty(); MAX_WINDOWS],
            count: 0,
            drag: None,
        }
    }

    fn open(&mut self, kind: WindowKind, content: ContentBuffer) -> bool {
        if self.count >= MAX_WINDOWS {
            return false;
        }

        let (screen_width, screen_height) = framebuffer::screen_size().unwrap_or((640, 480));
        let index = self.count;
        let width = 360.min(screen_width.saturating_sub(32)).max(160);
        let height = match kind {
            WindowKind::Notes => 152,
            WindowKind::Sysinfo => 176,
        }
        .min(screen_height.saturating_sub(framebuffer::TOP_BAR_RESERVED_HEIGHT + 16))
        .max(80);
        let x = clamp_usize(32 + index * 28, 0, screen_width.saturating_sub(width));
        let y = clamp_usize(
            framebuffer::TOP_BAR_RESERVED_HEIGHT + 24 + index * 24,
            framebuffer::TOP_BAR_RESERVED_HEIGHT,
            screen_height.saturating_sub(height),
        );

        self.windows[index] = Window {
            x,
            y,
            width,
            height,
            title: kind.title(),
            content: content.bytes,
            content_len: content.len,
            kind,
        };
        self.count += 1;
        self.redraw();

        crate::serial_println!("[CHRONO] wm: open {}", kind.log_name());
        true
    }

    fn redraw(&self) {
        framebuffer::redraw_console_base();

        for index in 0..self.count {
            draw_window(self.windows[index]);
        }
    }
}

struct GlobalWindowManager(UnsafeCell<WindowManager>);

unsafe impl Sync for GlobalWindowManager {}

static WINDOW_MANAGER: GlobalWindowManager =
    GlobalWindowManager(UnsafeCell::new(WindowManager::new()));

pub fn open_notes() -> bool {
    let mut content = ContentBuffer::new();

    match crate::fs::read("note.txt") {
        Ok(text) => content.push_str(text),
        Err(_) => content.push_str("No note stored."),
    }

    with_manager(|manager| manager.open(WindowKind::Notes, content))
}

pub fn open_sysinfo() -> bool {
    let mut content = ContentBuffer::new();
    let profile = crate::theme::active_profile();
    let memory = crate::memory::stats();
    let uptime = crate::timer::uptime_seconds();
    let used_kb = memory.heap_used_bytes / 1024;

    let _ = write!(content, "OS: Chronosapian\n");
    let _ = write!(content, "Era: {}\n", profile.name);
    let _ = write!(content, "Uptime: {} seconds\n", uptime);
    let _ = write!(content, "Memory used: {} KB", used_kb);

    with_manager(|manager| manager.open(WindowKind::Sysinfo, content))
}

pub fn redraw_if_open() {
    with_manager(|manager| {
        if manager.count > 0 {
            manager.redraw();
        }
    });
}

fn with_manager<R>(action: impl FnOnce(&mut WindowManager) -> R) -> R {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: The shell loop is the only WM mutator. Interrupts are kept
        // disabled so mouse-event consumption cannot interleave with redraws.
        unsafe { action(&mut *WINDOW_MANAGER.0.get()) }
    })
}

fn clamp_usize(value: usize, min: usize, max: usize) -> usize {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

struct ContentBuffer {
    bytes: [u8; WINDOW_CONTENT_CAPACITY],
    len: usize,
}

impl ContentBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; WINDOW_CONTENT_CAPACITY],
            len: 0,
        }
    }

    fn push_str(&mut self, text: &str) {
        let _ = self.write_str(text);
    }
}

impl Write for ContentBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if self.len >= self.bytes.len() {
                break;
            }

            self.bytes[self.len] = match byte {
                b'\n' | 0x20..=0x7e => byte,
                _ => b'?',
            };
            self.len += 1;
        }

        Ok(())
    }
}
