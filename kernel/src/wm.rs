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
}

struct GlobalWindowManager(UnsafeCell<WindowManager>);

unsafe impl Sync for GlobalWindowManager {}

static WINDOW_MANAGER: GlobalWindowManager =
    GlobalWindowManager(UnsafeCell::new(WindowManager::new()));
