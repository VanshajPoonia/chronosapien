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

    fn handle_mouse_event(&mut self, event: MouseEvent) {
        if event.left_pressed {
            self.handle_left_press(event.x, event.y);
        } else if event.left_down && event.moved {
            self.handle_drag(event.x, event.y);
        }

        if event.left_released {
            self.drag = None;
        }
    }

    fn handle_left_press(&mut self, x: usize, y: usize) {
        let Some(index) = self.hit_test(x, y) else {
            self.drag = None;
            return;
        };

        if self.in_close_button(index, x, y) {
            self.close(index);
            return;
        }

        if self.in_title_bar(index, x, y) {
            let front_index = self.bring_to_front(index);
            let window = self.windows[front_index];
            self.drag = Some(DragState {
                window_index: front_index,
                offset_x: x.saturating_sub(window.x),
                offset_y: y.saturating_sub(window.y),
            });
            self.redraw();
            return;
        }

        self.bring_to_front(index);
        self.redraw();
    }

    fn handle_drag(&mut self, x: usize, y: usize) {
        let Some(drag) = self.drag else {
            return;
        };

        if drag.window_index >= self.count {
            self.drag = None;
            return;
        }

        let (screen_width, screen_height) = framebuffer::screen_size().unwrap_or((640, 480));
        let window = &mut self.windows[drag.window_index];
        let max_x = screen_width.saturating_sub(window.width);
        let max_y = screen_height.saturating_sub(window.height);

        window.x = clamp_usize(x.saturating_sub(drag.offset_x), 0, max_x);
        window.y = clamp_usize(
            y.saturating_sub(drag.offset_y),
            framebuffer::TOP_BAR_RESERVED_HEIGHT,
            max_y,
        );
        self.redraw();
    }

    fn redraw(&self) {
        framebuffer::redraw_console_base();

        for index in 0..self.count {
            draw_window(self.windows[index]);
        }
    }

    fn close(&mut self, index: usize) {
        if index >= self.count {
            return;
        }

        let kind = self.windows[index].kind;

        for move_index in index..self.count - 1 {
            self.windows[move_index] = self.windows[move_index + 1];
        }

        self.count -= 1;
        self.windows[self.count] = Window::empty();
        self.drag = None;
        self.redraw();

        crate::serial_println!("[CHRONO] wm: close {}", kind.log_name());
    }

    fn bring_to_front(&mut self, index: usize) -> usize {
        if index >= self.count {
            return index;
        }

        let window = self.windows[index];

        for move_index in index..self.count - 1 {
            self.windows[move_index] = self.windows[move_index + 1];
        }

        let front_index = self.count - 1;
        self.windows[front_index] = window;

        front_index
    }

    fn hit_test(&self, x: usize, y: usize) -> Option<usize> {
        let mut index = self.count;

        while index > 0 {
            index -= 1;
            let window = self.windows[index];

            if x >= window.x
                && x < window.x.saturating_add(window.width)
                && y >= window.y
                && y < window.y.saturating_add(window.height)
            {
                return Some(index);
            }
        }

        None
    }

    fn in_title_bar(&self, index: usize, x: usize, y: usize) -> bool {
        let window = self.windows[index];

        x >= window.x
            && x < window.x.saturating_add(window.width)
            && y >= window.y
            && y < window.y.saturating_add(TITLE_BAR_HEIGHT)
    }

    fn in_close_button(&self, index: usize, x: usize, y: usize) -> bool {
        let window = self.windows[index];
        let button_x = close_button_x(window);
        let button_y = window.y + 4;

        x >= button_x
            && x < button_x.saturating_add(CLOSE_SIZE)
            && y >= button_y
            && y < button_y.saturating_add(CLOSE_SIZE)
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

pub fn handle_mouse_event(event: MouseEvent) {
    with_manager(|manager| {
        manager.handle_mouse_event(event);
    });
}

fn with_manager<R>(action: impl FnOnce(&mut WindowManager) -> R) -> R {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: The shell loop is the only WM mutator. Interrupts are kept
        // disabled so mouse-event consumption cannot interleave with redraws.
        unsafe { action(&mut *WINDOW_MANAGER.0.get()) }
    })
}

fn draw_window(window: Window) {
    let style = WindowStyle::active();
    let title_y = window.y + 4;
    let body_y = window.y + TITLE_BAR_HEIGHT;
    let body_height = window.height.saturating_sub(TITLE_BAR_HEIGHT);

    framebuffer::fill_rect(window.x, window.y, window.width, window.height, style.body_bg);

    if style.title_fill {
        framebuffer::fill_rect(
            window.x + 1,
            window.y + 1,
            window.width.saturating_sub(2),
            TITLE_BAR_HEIGHT.saturating_sub(1),
            style.title_bg,
        );
    }

    draw_border(window, style);
    draw_title(window, style, title_y);
    draw_close_button(window, style);
    draw_content(window, style, body_y, body_height);
}

fn draw_border(window: Window, style: WindowStyle) {
    match style.border_kind {
        BorderKind::Flat => {
            framebuffer::stroke_rect(window.x, window.y, window.width, window.height, style.border);
        }
        BorderKind::ThreeD => {
            framebuffer::stroke_rect(window.x, window.y, window.width, window.height, style.shadow);
            framebuffer::stroke_rect(
                window.x + 1,
                window.y + 1,
                window.width.saturating_sub(2),
                window.height.saturating_sub(2),
                style.highlight,
            );
        }
        BorderKind::Future => {
            framebuffer::stroke_rect(window.x, window.y, window.width, window.height, style.border);
            framebuffer::fill_rect(
                window.x + 1,
                window.y + TITLE_BAR_HEIGHT,
                window.width.saturating_sub(2),
                1,
                style.title_fg,
            );
        }
    }
}

fn draw_title(window: Window, style: WindowStyle, title_y: usize) {
    let max_chars = window.width.saturating_sub(40) / framebuffer::GLYPH_WIDTH;

    draw_text_clipped(
        window.x + WINDOW_PADDING,
        title_y,
        window.title.as_bytes(),
        max_chars,
        style.title_fg,
        style.title_bg,
    );
}

fn draw_close_button(window: Window, style: WindowStyle) {
    let x = close_button_x(window);
    let y = window.y + 4;

    framebuffer::stroke_rect(x, y, CLOSE_SIZE, CLOSE_SIZE, style.title_fg);
    framebuffer::draw_text_at(x + 1, y, "X", style.title_fg, style.title_bg);
}

fn draw_content(window: Window, style: WindowStyle, body_y: usize, body_height: usize) {
    let max_cols = window.width.saturating_sub(WINDOW_PADDING * 2) / framebuffer::GLYPH_WIDTH;
    let max_rows = body_height.saturating_sub(WINDOW_PADDING) / framebuffer::GLYPH_HEIGHT;
    let mut start = 0;
    let mut len = 0;
    let mut row = 0;
    let content = &window.content[..window.content_len];

    for (index, byte) in content.iter().copied().enumerate() {
        if byte == b'\n' {
            draw_content_line(window, style, body_y, row, &content[start..start + len], max_cols);
            row += 1;
            start = index + 1;
            len = 0;

            if row >= max_rows {
                return;
            }
        } else if len < max_cols {
            len += 1;
        }
    }

    if row < max_rows {
        draw_content_line(window, style, body_y, row, &content[start..start + len], max_cols);
    }
}

fn draw_content_line(
    window: Window,
    style: WindowStyle,
    body_y: usize,
    row: usize,
    bytes: &[u8],
    max_cols: usize,
) {
    if bytes.is_empty() || max_cols == 0 {
        return;
    }

    let visible_len = bytes.len().min(max_cols);
    let text = unsafe { core::str::from_utf8_unchecked(&bytes[..visible_len]) };

    framebuffer::draw_text_at(
        window.x + WINDOW_PADDING,
        body_y + WINDOW_PADDING + row * framebuffer::GLYPH_HEIGHT,
        text,
        style.body_fg,
        style.body_bg,
    );
}

fn draw_text_clipped(
    x: usize,
    y: usize,
    bytes: &[u8],
    max_chars: usize,
    fg: Color,
    bg: Color,
) {
    if max_chars == 0 {
        return;
    }

    let visible_len = bytes.len().min(max_chars);
    let text = unsafe { core::str::from_utf8_unchecked(&bytes[..visible_len]) };

    framebuffer::draw_text_at(x, y, text, fg, bg);
}

fn close_button_x(window: Window) -> usize {
    window.x + window.width.saturating_sub(CLOSE_SIZE + 6)
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

#[derive(Clone, Copy)]
struct WindowStyle {
    body_bg: Color,
    body_fg: Color,
    title_bg: Color,
    title_fg: Color,
    border: Color,
    highlight: Color,
    shadow: Color,
    title_fill: bool,
    border_kind: BorderKind,
}

impl WindowStyle {
    fn active() -> Self {
        match crate::theme::active_era() {
            Era::Eighties => Self {
                body_bg: Color::BLACK,
                body_fg: Color::GREEN,
                title_bg: Color::BLACK,
                title_fg: Color::GREEN,
                border: Color::GREEN,
                highlight: Color::GREEN,
                shadow: Color::GREEN,
                title_fill: false,
                border_kind: BorderKind::Flat,
            },
            Era::Nineties => Self {
                body_bg: Color::LIGHT_GRAY,
                body_fg: Color::BLACK,
                title_bg: Color::GRAY,
                title_fg: Color::BLACK,
                border: Color::GRAY,
                highlight: Color::WHITE,
                shadow: Color::DARK,
                title_fill: true,
                border_kind: BorderKind::ThreeD,
            },
            Era::TwoThousands => Self {
                body_bg: Color::LIGHT_GRAY,
                body_fg: Color::BLACK,
                title_bg: Color::WHITE,
                title_fg: Color::BLACK,
                border: Color::DARK,
                highlight: Color::WHITE,
                shadow: Color::DARK,
                title_fill: true,
                border_kind: BorderKind::Flat,
            },
            Era::Future => Self {
                body_bg: Color::DARK,
                body_fg: Color::WHITE,
                title_bg: Color::DARK,
                title_fg: Color::WHITE,
                border: Color::WHITE,
                highlight: Color::WHITE,
                shadow: Color::DARKER,
                title_fill: false,
                border_kind: BorderKind::Future,
            },
        }
    }
}

#[derive(Clone, Copy)]
enum BorderKind {
    Flat,
    ThreeD,
    Future,
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
