//! Framebuffer-backed text console with a tiny bitmap font.

mod font;

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use core::cell::UnsafeCell;
use core::fmt;

use crate::theme::EraProfile;

const TOP_BAR_HEIGHT: usize = 16;
const TOP_BAR_TEXT_Y: usize = 4;
const TEXT_START_Y: usize = TOP_BAR_HEIGHT;
const MAX_COLUMNS: usize = 128;
const MAX_ROWS: usize = 94;
const CELL_COUNT: usize = MAX_COLUMNS * MAX_ROWS;

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
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

pub struct Writer {
    buffer: *mut u8,
    buffer_len: usize,
    info: FrameBufferInfo,
    column_position: usize,
    columns: usize,
    rows: usize,
    foreground: Color,
    background: Color,
    top_bar_foreground: Color,
    top_bar_background: Color,
    cells: [Cell; CELL_COUNT],
    initialized: bool,
}

impl Writer {
    const fn new() -> Self {
        Self {
            buffer: core::ptr::null_mut(),
            buffer_len: 0,
            info: FrameBufferInfo {
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
            initialized: false,
        }
    }

    fn init(&mut self, framebuffer: &mut FrameBuffer, profile: EraProfile) {
        self.info = framebuffer.info();
        self.buffer_len = self.info.byte_len;
        self.buffer = framebuffer.buffer_mut().as_mut_ptr();
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

pub fn init(framebuffer: &mut FrameBuffer, profile: EraProfile) {
    // SAFETY: Screen output is serialized through the single shell loop in this
    // milestone; timer interrupts do not print to the framebuffer.
    unsafe {
        (*WRITER.0.get()).init(framebuffer, profile);
    }

    let info = framebuffer.info();
    crate::serial_println!("[CHRONO] fb: {}x{} initialized", info.width, info.height);
}

pub fn clear() {
    // SAFETY: Same single-writer console model as `init`.
    unsafe {
        (*WRITER.0.get()).clear();
    }
}

pub fn backspace() {
    // SAFETY: Same single-writer console model as `init`.
    unsafe {
        (*WRITER.0.get()).backspace();
    }
}

pub fn set_theme(profile: EraProfile) {
    // SAFETY: Same single-writer console model as `init`.
    unsafe {
        (*WRITER.0.get()).set_theme(profile);
        (*WRITER.0.get()).clear();
    }
}

pub fn refresh_top_bar() {
    // SAFETY: Same single-writer console model as `init`.
    unsafe {
        (*WRITER.0.get()).refresh_top_bar();
    }
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;

    // SAFETY: Same single-writer console model as `init`.
    unsafe {
        let _ = (*WRITER.0.get()).write_fmt(args);
    }
}
