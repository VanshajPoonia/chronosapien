//! Interrupt-driven PS/2 mouse support.
//!
//! A PC PS/2 controller exposes its command/status register at port `0x64` and
//! its shared data register at port `0x60`. The keyboard uses the first PS/2
//! port directly; mouse commands go to the second PS/2 port by first writing
//! controller command `0xD4`, then writing the mouse command byte to `0x60`.
//! During initialization we enable the second port, enable its IRQ line in the
//! controller config byte, set mouse defaults, and finally enable packet
//! streaming.
//!
//! The standard mouse packet is three bytes. Byte 0 contains buttons and packet
//! metadata: bit 0 left, bit 1 right, bit 2 middle, bit 3 always set, bit 4 X
//! sign, bit 5 Y sign, bit 6 X overflow, and bit 7 Y overflow. Bytes 1 and 2
//! are signed X/Y movement deltas. PS/2 Y is positive upward, so screen-space Y
//! subtracts the packet delta.

use core::cell::UnsafeCell;

use crate::framebuffer;

const STATUS_PORT: u16 = 0x64;
const COMMAND_PORT: u16 = 0x64;
const DATA_PORT: u16 = 0x60;

const OUTPUT_BUFFER_FULL: u8 = 1 << 0;
const INPUT_BUFFER_FULL: u8 = 1 << 1;
const AUXILIARY_OUTPUT: u8 = 1 << 5;

const ENABLE_SECOND_PORT: u8 = 0xA8;
const READ_CONFIG_BYTE: u8 = 0x20;
const WRITE_CONFIG_BYTE: u8 = 0x60;
const WRITE_TO_MOUSE: u8 = 0xD4;

const CONFIG_SECOND_PORT_IRQ: u8 = 1 << 1;
const CONFIG_SECOND_PORT_CLOCK_DISABLED: u8 = 1 << 5;

const MOUSE_ACK: u8 = 0xFA;
const MOUSE_SET_DEFAULTS: u8 = 0xF6;
const MOUSE_ENABLE_DATA_REPORTING: u8 = 0xF4;

const PACKET_ALWAYS_SET: u8 = 1 << 3;
const PACKET_LEFT_BUTTON: u8 = 1 << 0;
const PACKET_X_OVERFLOW: u8 = 1 << 6;
const PACKET_Y_OVERFLOW: u8 = 1 << 7;

const WAIT_LIMIT: usize = 100_000;

#[derive(Clone, Copy)]
pub struct MouseEvent {
    pub x: usize,
    pub y: usize,
    pub left_down: bool,
    pub left_pressed: bool,
    pub left_released: bool,
    pub moved: bool,
}

#[derive(Clone, Copy)]
struct MouseState {
    x: i32,
    y: i32,
    screen_width: usize,
    screen_height: usize,
    packet: [u8; 3],
    packet_index: usize,
    left_button_down: bool,
    pending_event: Option<MouseEvent>,
    initialized: bool,
}

impl MouseState {
    const fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            screen_width: 0,
            screen_height: 0,
            packet: [0; 3],
            packet_index: 0,
            left_button_down: false,
            pending_event: None,
            initialized: false,
        }
    }
}

struct GlobalMouse(UnsafeCell<MouseState>);

unsafe impl Sync for GlobalMouse {}

static MOUSE: GlobalMouse = GlobalMouse(UnsafeCell::new(MouseState::new()));

pub fn init() {
    let Some((width, height)) = framebuffer::screen_size() else {
        crate::serial_println!("[CHRONO] mouse: init failed");
        return;
    };

    if width == 0 || height == 0 {
        crate::serial_println!("[CHRONO] mouse: init failed");
        return;
    }

    // SAFETY: The mouse state is initialized during early boot before
    // interrupts are enabled.
    let state = unsafe { &mut *MOUSE.0.get() };
    state.x = (width / 2) as i32;
    state.y = (height / 2) as i32;
    state.screen_width = width;
    state.screen_height = height;
    state.packet = [0; 3];
    state.packet_index = 0;
    state.left_button_down = false;
    state.pending_event = None;
    state.initialized = false;

    drain_output_buffer();

    if !initialize_controller_and_device() {
        crate::serial_println!("[CHRONO] mouse: init failed");
        return;
    }

    state.initialized = true;
    framebuffer::set_mouse_cursor_position(state.x as usize, state.y as usize);
    crate::serial_println!("[CHRONO] mouse: initialized");
}

pub fn handle_interrupt() {
    // SAFETY: IRQ12 fires because the PS/2 controller has a byte waiting for
    // the auxiliary device. Reading port `0x60` acknowledges that byte at the
    // controller level; the PIC EOI is sent by the interrupt wrapper.
    let status = unsafe { inb(STATUS_PORT) };
    if status & OUTPUT_BUFFER_FULL == 0 {
        return;
    }

    // If the controller presents a non-auxiliary byte on IRQ12, consume it so
    // the interrupt line can settle, but do not feed it to the mouse decoder.
    let byte = unsafe { inb(DATA_PORT) };
    if status & AUXILIARY_OUTPUT == 0 {
        return;
    }

    // SAFETY: The handler runs with interrupts disabled on one CPU.
    let state = unsafe { &mut *MOUSE.0.get() };
    if !state.initialized {
        return;
    }

    push_packet_byte(state, byte);
}

fn initialize_controller_and_device() -> bool {
    if !send_controller_command(ENABLE_SECOND_PORT) {
        return false;
    }

    if !send_controller_command(READ_CONFIG_BYTE) {
        return false;
    }

    let Some(mut config) = read_data() else {
        return false;
    };

    config |= CONFIG_SECOND_PORT_IRQ;
    config &= !CONFIG_SECOND_PORT_CLOCK_DISABLED;

    if !send_controller_command(WRITE_CONFIG_BYTE) || !write_data(config) {
        return false;
    }

    send_mouse_command(MOUSE_SET_DEFAULTS) && send_mouse_command(MOUSE_ENABLE_DATA_REPORTING)
}

fn push_packet_byte(state: &mut MouseState, byte: u8) {
    if state.packet_index == 0 && byte & PACKET_ALWAYS_SET == 0 {
        return;
    }

    state.packet[state.packet_index] = byte;
    state.packet_index += 1;

    if state.packet_index == state.packet.len() {
        state.packet_index = 0;
        handle_packet(state);
    }
}

fn handle_packet(state: &mut MouseState) {
    let flags = state.packet[0];
    if flags & (PACKET_X_OVERFLOW | PACKET_Y_OVERFLOW) != 0 {
        return;
    }

    let dx = state.packet[1] as i8 as i32;
    let dy = state.packet[2] as i8 as i32;
    let max_x = state.screen_width.saturating_sub(1) as i32;
    let max_y = state.screen_height.saturating_sub(1) as i32;

    state.x = clamp(state.x + dx, 0, max_x);
    state.y = clamp(state.y - dy, 0, max_y);

    framebuffer::set_mouse_cursor_position(state.x as usize, state.y as usize);

    let left_button_down = flags & PACKET_LEFT_BUTTON != 0;
    if left_button_down && !state.left_button_down {
        crate::serial_println!("[CHRONO] mouse: click at {},{}", state.x, state.y);
    }

    state.left_button_down = left_button_down;
}

fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn send_mouse_command(command: u8) -> bool {
    if !send_controller_command(WRITE_TO_MOUSE) || !write_data(command) {
        return false;
    }

    matches!(read_data(), Some(MOUSE_ACK))
}

fn send_controller_command(command: u8) -> bool {
    if !wait_input_clear() {
        return false;
    }

    // SAFETY: Port `0x64` is the PS/2 controller command port.
    unsafe {
        outb(COMMAND_PORT, command);
    }

    true
}

fn write_data(value: u8) -> bool {
    if !wait_input_clear() {
        return false;
    }

    // SAFETY: Port `0x60` is the PS/2 controller data port.
    unsafe {
        outb(DATA_PORT, value);
    }

    true
}

fn read_data() -> Option<u8> {
    if !wait_output_full() {
        return None;
    }

    // SAFETY: Port `0x60` is ready because the output buffer is full.
    Some(unsafe { inb(DATA_PORT) })
}

fn drain_output_buffer() {
    for _ in 0..WAIT_LIMIT {
        // SAFETY: Reading the PS/2 status register is side-effect free.
        if unsafe { inb(STATUS_PORT) } & OUTPUT_BUFFER_FULL == 0 {
            return;
        }

        // SAFETY: The output buffer is full, so one data byte can be consumed.
        let _ = unsafe { inb(DATA_PORT) };
    }
}

fn wait_input_clear() -> bool {
    for _ in 0..WAIT_LIMIT {
        // SAFETY: Reading the PS/2 status register is side-effect free.
        if unsafe { inb(STATUS_PORT) } & INPUT_BUFFER_FULL == 0 {
            return true;
        }
    }

    false
}

fn wait_output_full() -> bool {
    for _ in 0..WAIT_LIMIT {
        // SAFETY: Reading the PS/2 status register is side-effect free.
        if unsafe { inb(STATUS_PORT) } & OUTPUT_BUFFER_FULL != 0 {
            return true;
        }
    }

    false
}

unsafe fn outb(port: u16, value: u8) {
    // SAFETY: `out` writes one byte to the specified I/O port. The caller must
    // ensure the port belongs to the intended hardware.
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    // SAFETY: `in` reads one byte from the specified I/O port. The caller must
    // ensure the port belongs to the intended hardware.
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags)
    );

    value
}
