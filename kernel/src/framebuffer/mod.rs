//! Framebuffer-backed text console with a tiny bitmap font.

mod font;

use core::cell::UnsafeCell;
use core::fmt;

use crate::theme::EraProfile;

const TOP_BAR_HEIGHT: usize = 16;
const TOP_BAR_TEXT_Y: usize = 4;
const TEXT_START_Y: usize = TOP_BAR_HEIGHT;
pub const TOP_BAR_RESERVED_HEIGHT: usize = TOP_BAR_HEIGHT;
pub const GLYPH_WIDTH: usize = font::FONT_WIDTH;
pub const GLYPH_HEIGHT: usize = font::FONT_HEIGHT;
const MAX_COLUMNS: usize = 128;
const MAX_ROWS: usize = 94;
const CELL_COUNT: usize = MAX_COLUMNS * MAX_ROWS;
const MOUSE_CURSOR_SIZE: usize = 8;
const MOUSE_CURSOR_PIXELS: usize = MOUSE_CURSOR_SIZE * MOUSE_CURSOR_SIZE;
const MOUSE_CURSOR_OUTLINE: [u8; MOUSE_CURSOR_SIZE] = [
    0b1100_0000,
    0b1110_0000,
    0b1111_0000,
    0b1111_1000,
    0b1111_1100,
    0b1111_0000,
    0b1101_1000,
    0b1000_1100,
];
const MOUSE_CURSOR_FILL: [u8; MOUSE_CURSOR_SIZE] = [
    0b1000_0000,
    0b1100_0000,
    0b1010_0000,
    0b1001_0000,
    0b1000_1000,
    0b1010_0000,
    0b1001_0000,
    0b0000_1000,
];

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    Rgb,
    Bgr,
    U8,
    Unknown,
}

#[derive(Clone, Copy, Debug)]
pub struct FramebufferInfo {
    pub byte_len: usize,
    pub width: usize,
    pub height: usize,
    pub pixel_format: PixelFormat,
    pub bytes_per_pixel: usize,
    pub stride: usize,
}

#[derive(Clone, Copy)]
pub struct Framebuffer {
    pub address: *mut u8,
    pub info: FramebufferInfo,
}

impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const GREEN: Self = Self::rgb(0, 255, 96);
    pub const BLUE: Self = Self::rgb(0, 58, 180);
    pub const GRAY: Self = Self::rgb(192, 192, 192);
    pub const LIGHT_GRAY: Self = Self::rgb(232, 232, 232);
    pub const DARK: Self = Self::rgb(8, 10, 14);
    pub const DARKER: Self = Self::rgb(0, 0, 0);
    pub const CYAN: Self = Self::rgb(120, 220, 255);

    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

#[derive(Clone, Copy)]
struct Cell {
    byte: u8,
    foreground: Color,
    background: Color,
}

impl Cell {
    const fn blank(foreground: Color, background: Color) -> Self {
        Self {
            byte: b' ',
            foreground,
            background,
        }
    }
}

struct MouseCursor {
    x: usize,
    y: usize,
    visible: bool,
    background: [Color; MOUSE_CURSOR_PIXELS],
}

impl MouseCursor {
    const fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            visible: false,
            background: [Color::BLACK; MOUSE_CURSOR_PIXELS],
        }
    }
}

pub struct Writer {
    buffer: *mut u8,
    buffer_len: usize,
    info: FramebufferInfo,
    column_position: usize,
    columns: usize,
    rows: usize,
    foreground: Color,
    background: Color,
    top_bar_foreground: Color,
    top_bar_background: Color,
    cells: [Cell; CELL_COUNT],
    mouse_cursor: MouseCursor,
    initialized: bool,
}

impl Writer {
    const fn new() -> Self {
        Self {
            buffer: core::ptr::null_mut(),
            buffer_len: 0,
            info: FramebufferInfo {
                byte_len: 0,
                width: 0,
                height: 0,
                pixel_format: PixelFormat::Rgb,
                bytes_per_pixel: 0,
                stride: 0,
            },
            column_position: 0,
            columns: 0,
            rows: 0,
            foreground: Color::WHITE,
            background: Color::BLACK,
            top_bar_foreground: Color::WHITE,
            top_bar_background: Color::BLUE,
            cells: [Cell::blank(Color::WHITE, Color::BLACK); CELL_COUNT],
            mouse_cursor: MouseCursor::new(),
            initialized: false,
        }
    }

    fn init(&mut self, framebuffer: Framebuffer, profile: EraProfile) {
        self.info = framebuffer.info;
        self.buffer_len = self.info.byte_len;
        self.buffer = framebuffer.address;
        match self.info.pixel_format {
            PixelFormat::Rgb | PixelFormat::Bgr if self.info.bytes_per_pixel >= 3 => {}
            PixelFormat::U8 if self.info.bytes_per_pixel >= 1 => {}
            _ => {
                crate::serial_println!(
                    "[CHRONO] fb: unsupported pixel layout {:?} {}bpp",
                    self.info.pixel_format,
                    self.info.bytes_per_pixel
                );
                panic!("unsupported framebuffer pixel layout");
            }
        }

        self.columns = (self.info.width / font::FONT_WIDTH).min(MAX_COLUMNS);
        self.rows = ((self.info.height.saturating_sub(TEXT_START_Y)) / font::FONT_HEIGHT)
            .min(MAX_ROWS);
        self.initialized = true;
        self.set_theme(profile);
        self.clear();
    }

    fn set_theme(&mut self, profile: EraProfile) {
        self.foreground = profile.fg;
        self.background = profile.bg;
        self.top_bar_foreground = profile.top_bar_fg;
        self.top_bar_background = profile.top_bar_bg;

        for cell in self.cells.iter_mut() {
            cell.foreground = self.foreground;
            cell.background = self.background;
        }
    }

    fn clear(&mut self) {
        if !self.initialized {
            return;
        }

        self.fill_rect(0, 0, self.info.width, self.info.height, self.background);

        for cell in self.visible_cells_mut() {
            *cell = Cell::blank(self.foreground, self.background);
        }

        self.column_position = 0;
        self.draw_top_bar();
    }

    fn refresh_top_bar(&mut self) {
        if self.initialized {
            self.draw_top_bar();
        }
    }

    fn redraw_console_base(&mut self) {
        if !self.initialized {
            return;
        }

        self.fill_rect(0, 0, self.info.width, self.info.height, self.background);

        for row in 0..self.rows {
            for col in 0..self.columns {
                let cell = self.cells[self.cell_index(row, col)];
                self.draw_cell(row, col, cell);
            }
        }

        self.draw_top_bar();
    }

    fn write_byte(&mut self, byte: u8) {
        if !self.initialized || self.columns == 0 || self.rows == 0 {
            return;
        }

        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= self.columns {
                    self.new_line();
                }

                let row = self.rows - 1;
                let col = self.column_position;
                let cell = Cell {
                    byte,
                    foreground: self.foreground,
                    background: self.background,
                };

                self.set_cell(row, col, cell);
                self.column_position += 1;
            }
        }

        self.draw_top_bar();
    }

    fn backspace(&mut self) {
        if !self.initialized || self.column_position == 0 || self.rows == 0 {
            return;
        }

        self.column_position -= 1;
        self.set_cell(
            self.rows - 1,
            self.column_position,
            Cell::blank(self.foreground, self.background),
        );
        self.draw_top_bar();
    }

    fn new_line(&mut self) {
        for row in 1..self.rows {
            for col in 0..self.columns {
                let from = self.cell_index(row, col);
                let to = self.cell_index(row - 1, col);
                self.cells[to] = self.cells[from];
            }
        }

        for col in 0..self.columns {
            let index = self.cell_index(self.rows - 1, col);
            self.cells[index] = Cell::blank(self.foreground, self.background);
        }

        self.redraw_text_region();
        self.column_position = 0;
    }

    fn set_cell(&mut self, row: usize, col: usize, cell: Cell) {
        let index = self.cell_index(row, col);
        self.cells[index] = cell;
        self.draw_cell(row, col, cell);
    }

    fn redraw_text_region(&mut self) {
        self.fill_rect(
            0,
            TEXT_START_Y,
            self.info.width,
            self.info.height.saturating_sub(TEXT_START_Y),
            self.background,
        );

        for row in 0..self.rows {
            for col in 0..self.columns {
                let cell = self.cells[self.cell_index(row, col)];
                self.draw_cell(row, col, cell);
            }
        }
    }

    fn draw_cell(&mut self, row: usize, col: usize, cell: Cell) {
        let x = col * font::FONT_WIDTH;
        let y = TEXT_START_Y + row * font::FONT_HEIGHT;

        self.draw_glyph(x, y, cell.byte, cell.foreground, cell.background);
    }

    fn draw_top_bar(&mut self) {
        self.fill_rect(
            0,
            0,
            self.info.width,
            TOP_BAR_HEIGHT,
            self.top_bar_background,
        );

        self.draw_text_at(
            8,
            TOP_BAR_TEXT_Y,
            "Chronosapian | Era: ",
            self.top_bar_foreground,
            self.top_bar_background,
        );
        self.draw_text_at(
            8 + 22 * font::FONT_WIDTH,
            TOP_BAR_TEXT_Y,
            crate::theme::active_profile().name,
            self.top_bar_foreground,
            self.top_bar_background,
        );
        self.draw_text_at(
            8 + 26 * font::FONT_WIDTH,
            TOP_BAR_TEXT_Y,
            " | Uptime: ",
            self.top_bar_foreground,
            self.top_bar_background,
        );
        self.draw_u64_at(
            8 + 37 * font::FONT_WIDTH,
            TOP_BAR_TEXT_Y,
            crate::timer::uptime_seconds(),
            self.top_bar_foreground,
            self.top_bar_background,
        );
        self.draw_text_at(
            8 + 47 * font::FONT_WIDTH,
            TOP_BAR_TEXT_Y,
            "s",
            self.top_bar_foreground,
            self.top_bar_background,
        );
    }

    fn draw_text_at(&mut self, mut x: usize, y: usize, text: &str, fg: Color, bg: Color) {
        for byte in text.bytes() {
            self.draw_glyph(x, y, byte, fg, bg);
            x += font::FONT_WIDTH;
        }
    }

    fn draw_u64_at(&mut self, x: usize, y: usize, value: u64, fg: Color, bg: Color) {
        let mut digits = [0u8; 20];
        let mut len = 0;
        let mut value = value;

        if value == 0 {
            digits[0] = b'0';
            len = 1;
        } else {
            while value > 0 {
                digits[len] = b'0' + (value % 10) as u8;
                value /= 10;
                len += 1;
            }
        }

        for index in 0..len {
            self.draw_glyph(
                x + index * font::FONT_WIDTH,
                y,
                digits[len - index - 1],
                fg,
                bg,
            );
        }
    }

    fn draw_glyph(&mut self, x: usize, y: usize, byte: u8, fg: Color, bg: Color) {
        let glyph = font::glyph(byte);

        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..font::FONT_WIDTH {
                let color = if bits & (0x80 >> col) != 0 { fg } else { bg };
                self.write_pixel(x + col, y + row, color);
            }
        }
    }

    fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: Color) {
        let x_end = x.saturating_add(width).min(self.info.width);
        let y_end = y.saturating_add(height).min(self.info.height);

        for pixel_y in y..y_end {
            for pixel_x in x..x_end {
                self.write_pixel(pixel_x, pixel_y, color);
            }
        }
    }

    fn stroke_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: Color) {
        if width == 0 || height == 0 {
            return;
        }

        self.fill_rect(x, y, width, 1, color);
        self.fill_rect(
            x,
            y.saturating_add(height.saturating_sub(1)),
            width,
            1,
            color,
        );
        self.fill_rect(x, y, 1, height, color);
        self.fill_rect(
            x.saturating_add(width.saturating_sub(1)),
            y,
            1,
            height,
            color,
        );
    }

    fn screen_size(&self) -> Option<(usize, usize)> {
        if !self.initialized {
            return None;
        }

        Some((self.info.width, self.info.height))
    }

    fn set_mouse_cursor_position(&mut self, x: usize, y: usize) {
        if !self.initialized || self.info.width == 0 || self.info.height == 0 {
            return;
        }

        self.hide_mouse_cursor();
        self.mouse_cursor.x = x.min(self.info.width - 1);
        self.mouse_cursor.y = y.min(self.info.height - 1);
        self.show_mouse_cursor();
    }

    fn with_mouse_cursor_hidden(&mut self, action: impl FnOnce(&mut Self)) {
        let was_visible = self.mouse_cursor.visible;

        if was_visible {
            self.hide_mouse_cursor();
        }

        action(self);

        if was_visible {
            self.show_mouse_cursor();
        }
    }

    fn show_mouse_cursor(&mut self) {
        if !self.initialized || self.mouse_cursor.visible {
            return;
        }

        self.save_mouse_background();
        self.draw_mouse_cursor();
        self.mouse_cursor.visible = true;
    }

    fn hide_mouse_cursor(&mut self) {
        if !self.initialized || !self.mouse_cursor.visible {
            return;
        }

        self.restore_mouse_background();
        self.mouse_cursor.visible = false;
    }

    fn save_mouse_background(&mut self) {
        let cursor_x = self.mouse_cursor.x;
        let cursor_y = self.mouse_cursor.y;

        for row in 0..MOUSE_CURSOR_SIZE {
            for col in 0..MOUSE_CURSOR_SIZE {
                let index = row * MOUSE_CURSOR_SIZE + col;
                let x = cursor_x + col;
                let y = cursor_y + row;

                self.mouse_cursor.background[index] = self.read_pixel(x, y);
            }
        }
    }

    fn restore_mouse_background(&mut self) {
        let cursor_x = self.mouse_cursor.x;
        let cursor_y = self.mouse_cursor.y;

        for row in 0..MOUSE_CURSOR_SIZE {
            for col in 0..MOUSE_CURSOR_SIZE {
                let index = row * MOUSE_CURSOR_SIZE + col;
                let x = cursor_x + col;
                let y = cursor_y + row;
                let color = self.mouse_cursor.background[index];

                self.write_pixel(x, y, color);
            }
        }
    }

    fn draw_mouse_cursor(&mut self) {
        let cursor_x = self.mouse_cursor.x;
        let cursor_y = self.mouse_cursor.y;

        for row in 0..MOUSE_CURSOR_SIZE {
            for col in 0..MOUSE_CURSOR_SIZE {
                let bit = 0x80 >> col;
                let x = cursor_x + col;
                let y = cursor_y + row;

                if MOUSE_CURSOR_FILL[row] & bit != 0 {
                    self.write_pixel(x, y, Color::WHITE);
                } else if MOUSE_CURSOR_OUTLINE[row] & bit != 0 {
                    self.write_pixel(x, y, Color::DARKER);
                }
            }
        }
    }

    fn read_pixel(&self, x: usize, y: usize) -> Color {
        if x >= self.info.width || y >= self.info.height {
            return Color::BLACK;
        }

        let offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;

        if offset + self.info.bytes_per_pixel > self.buffer_len {
            return Color::BLACK;
        }

        let ptr = unsafe { self.buffer.add(offset) };

        match self.info.pixel_format {
            PixelFormat::Rgb => unsafe {
                Color::rgb(
                    core::ptr::read_volatile(ptr),
                    core::ptr::read_volatile(ptr.add(1)),
                    core::ptr::read_volatile(ptr.add(2)),
                )
            },
            PixelFormat::Bgr => unsafe {
                Color::rgb(
                    core::ptr::read_volatile(ptr.add(2)),
                    core::ptr::read_volatile(ptr.add(1)),
                    core::ptr::read_volatile(ptr),
                )
            },
            PixelFormat::U8 => unsafe {
                let gray = core::ptr::read_volatile(ptr);
                Color::rgb(gray, gray, gray)
            },
            _ => Color::BLACK,
        }
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;

        if offset + self.info.bytes_per_pixel > self.buffer_len {
            return;
        }

        let ptr = unsafe { self.buffer.add(offset) };

        match self.info.pixel_format {
            PixelFormat::Rgb => unsafe {
                core::ptr::write_volatile(ptr, color.red);
                core::ptr::write_volatile(ptr.add(1), color.green);
                core::ptr::write_volatile(ptr.add(2), color.blue);
            },
            PixelFormat::Bgr => unsafe {
                core::ptr::write_volatile(ptr, color.blue);
                core::ptr::write_volatile(ptr.add(1), color.green);
                core::ptr::write_volatile(ptr.add(2), color.red);
            },
            PixelFormat::U8 => unsafe {
                let gray = ((color.red as u16 + color.green as u16 + color.blue as u16) / 3) as u8;
                core::ptr::write_volatile(ptr, gray);
            },
            _ => {}
        }
    }

    fn visible_cells_mut(&mut self) -> &mut [Cell] {
        let len = self.rows * self.columns;

        &mut self.cells[..len]
    }

    fn cell_index(&self, row: usize, col: usize) -> usize {
        row * self.columns + col
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(b'?'),
            }
        }

        Ok(())
    }
}

struct GlobalWriter(UnsafeCell<Writer>);

unsafe impl Sync for GlobalWriter {}

static WRITER: GlobalWriter = GlobalWriter(UnsafeCell::new(Writer::new()));

pub fn init(framebuffer: Framebuffer, profile: EraProfile) {
    // SAFETY: The framebuffer writer is initialized once during early boot
    // before interrupts are enabled.
    unsafe {
        (*WRITER.0.get()).init(framebuffer, profile);
    }

    let info = framebuffer.info;
    crate::serial_println!("[CHRONO] fb: {}x{} initialized", info.width, info.height);
}

pub fn clear() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: Framebuffer access is serialized by disabling interrupts while
        // the shell and IRQ12 mouse path can both mutate pixels.
        unsafe {
            (*WRITER.0.get()).with_mouse_cursor_hidden(|writer| writer.clear());
        }
    });
}

pub fn backspace() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: Same interrupt-exclusion model as `clear`.
        unsafe {
            (*WRITER.0.get()).with_mouse_cursor_hidden(|writer| writer.backspace());
        }
    });
}

pub fn set_theme(profile: EraProfile) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: Same interrupt-exclusion model as `clear`.
        unsafe {
            (*WRITER.0.get()).with_mouse_cursor_hidden(|writer| {
                writer.set_theme(profile);
                writer.clear();
            });
        }
    });
}

pub fn refresh_top_bar() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: Same interrupt-exclusion model as `clear`.
        unsafe {
            (*WRITER.0.get()).with_mouse_cursor_hidden(|writer| writer.refresh_top_bar());
        }
    });
}

pub fn redraw_console_base() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: Same interrupt-exclusion model as `clear`.
        unsafe {
            (*WRITER.0.get()).with_mouse_cursor_hidden(|writer| writer.redraw_console_base());
        }
    });
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;

    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: Same interrupt-exclusion model as `clear`.
        unsafe {
            (*WRITER.0.get()).with_mouse_cursor_hidden(|writer| {
                let _ = writer.write_fmt(args);
            });
        }
    });
}

pub fn screen_size() -> Option<(usize, usize)> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: Read-only access to the global writer while interrupts are
        // disabled, matching the framebuffer mutation paths.
        unsafe { (*WRITER.0.get()).screen_size() }
    })
}

pub fn fill_rect(x: usize, y: usize, width: usize, height: usize, color: Color) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: Same interrupt-exclusion model as `clear`.
        unsafe {
            (*WRITER.0.get()).with_mouse_cursor_hidden(|writer| {
                writer.fill_rect(x, y, width, height, color);
            });
        }
    });
}

pub fn stroke_rect(x: usize, y: usize, width: usize, height: usize, color: Color) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: Same interrupt-exclusion model as `clear`.
        unsafe {
            (*WRITER.0.get()).with_mouse_cursor_hidden(|writer| {
                writer.stroke_rect(x, y, width, height, color);
            });
        }
    });
}

pub fn draw_text_at(x: usize, y: usize, text: &str, fg: Color, bg: Color) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: Same interrupt-exclusion model as `clear`.
        unsafe {
            (*WRITER.0.get()).with_mouse_cursor_hidden(|writer| {
                writer.draw_text_at(x, y, text, fg, bg);
            });
        }
    });
}

pub fn set_mouse_cursor_position(x: usize, y: usize) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: The IRQ12 mouse handler is the only caller expected to move
        // this cursor after boot, and interrupts are disabled while mutating it.
        unsafe {
            (*WRITER.0.get()).set_mouse_cursor_position(x, y);
        }
    });
}
