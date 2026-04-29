//! Tiny PS/2 keyboard polling for the first shell milestone.

use core::cell::UnsafeCell;

const STATUS_PORT: u16 = 0x64;
const DATA_PORT: u16 = 0x60;
const OUTPUT_BUFFER_FULL: u8 = 1 << 0;

#[derive(Clone, Copy, Debug)]
pub enum KeyEvent {
    Char(u8),
    Enter,
    Backspace,
}

#[derive(Clone, Copy)]
struct KeyboardState {
    shift_pressed: bool,
}

struct GlobalKeyboard(UnsafeCell<KeyboardState>);

unsafe impl Sync for GlobalKeyboard {}

static KEYBOARD_STATE: GlobalKeyboard = GlobalKeyboard(UnsafeCell::new(KeyboardState {
    shift_pressed: false,
}));

pub fn read_key() -> Option<KeyEvent> {
    // SAFETY: Port `0x64` is the PS/2 controller status port on the standard
    // PC-compatible machine that QEMU emulates. We read it first to check
    // whether a keyboard byte is waiting before touching the data port.
    let status = unsafe { inb(STATUS_PORT) };
    if status & OUTPUT_BUFFER_FULL == 0 {
        return None;
    }

    // SAFETY: Port `0x60` is the PS/2 controller data port. Reading it is only
    // correct after the status register says the output buffer is full.
    let scancode = unsafe { inb(DATA_PORT) };

    // SAFETY: This early kernel is still single-core and non-preemptive, so
    // keeping a tiny mutable keyboard state in an `UnsafeCell` is safe here.
    let state = unsafe { &mut *KEYBOARD_STATE.0.get() };

    match scancode {
        0x2A | 0x36 => {
            state.shift_pressed = true;
            None
        }
        0xAA | 0xB6 => {
            state.shift_pressed = false;
            None
        }
        0x80..=0xFF => None,
        0x1C => Some(KeyEvent::Enter),
        0x0E => Some(KeyEvent::Backspace),
        make_code => decode_scancode(make_code, state.shift_pressed).map(KeyEvent::Char),
    }
}

fn decode_scancode(scancode: u8, shift_pressed: bool) -> Option<u8> {
    // These are the common set-1 scancodes produced by a standard PC keyboard
    // in QEMU's default PS/2-compatible path. We only decode the subset needed
    // for text input so the table stays readable. Each match arm maps one raw
    // keyboard scancode to the ASCII byte the console should display.
    let byte = match (scancode, shift_pressed) {
        (0x02, false) => b'1',
        (0x03, false) => b'2',
        (0x04, false) => b'3',
        (0x05, false) => b'4',
        (0x06, false) => b'5',
        (0x07, false) => b'6',
        (0x08, false) => b'7',
        (0x09, false) => b'8',
        (0x0A, false) => b'9',
        (0x0B, false) => b'0',
        (0x0C, false) => b'-',
        (0x0D, false) => b'=',
        (0x10, false) => b'q',
        (0x11, false) => b'w',
        (0x12, false) => b'e',
        (0x13, false) => b'r',
        (0x14, false) => b't',
        (0x15, false) => b'y',
        (0x16, false) => b'u',
        (0x17, false) => b'i',
        (0x18, false) => b'o',
        (0x19, false) => b'p',
        (0x1A, false) => b'[',
        (0x1B, false) => b']',
        (0x1E, false) => b'a',
        (0x1F, false) => b's',
        (0x20, false) => b'd',
        (0x21, false) => b'f',
        (0x22, false) => b'g',
        (0x23, false) => b'h',
        (0x24, false) => b'j',
        (0x25, false) => b'k',
        (0x26, false) => b'l',
        (0x27, false) => b';',
        (0x28, false) => b'\'',
        (0x29, false) => b'`',
        (0x2B, false) => b'\\',
        (0x2C, false) => b'z',
        (0x2D, false) => b'x',
        (0x2E, false) => b'c',
        (0x2F, false) => b'v',
        (0x30, false) => b'b',
        (0x31, false) => b'n',
        (0x32, false) => b'm',
        (0x33, false) => b',',
        (0x34, false) => b'.',
        (0x35, false) => b'/',
        (0x39, false) => b' ',
        (0x02, true) => b'!',
        (0x03, true) => b'@',
        (0x04, true) => b'#',
        (0x05, true) => b'$',
        (0x06, true) => b'%',
        (0x07, true) => b'^',
        (0x08, true) => b'&',
        (0x09, true) => b'*',
        (0x0A, true) => b'(',
        (0x0B, true) => b')',
        (0x0C, true) => b'_',
        (0x0D, true) => b'+',
        (0x10, true) => b'Q',
        (0x11, true) => b'W',
        (0x12, true) => b'E',
        (0x13, true) => b'R',
        (0x14, true) => b'T',
        (0x15, true) => b'Y',
        (0x16, true) => b'U',
        (0x17, true) => b'I',
        (0x18, true) => b'O',
        (0x19, true) => b'P',
        (0x1A, true) => b'{',
        (0x1B, true) => b'}',
        (0x1E, true) => b'A',
        (0x1F, true) => b'S',
        (0x20, true) => b'D',
        (0x21, true) => b'F',
        (0x22, true) => b'G',
        (0x23, true) => b'H',
        (0x24, true) => b'J',
        (0x25, true) => b'K',
        (0x26, true) => b'L',
        (0x27, true) => b':',
        (0x28, true) => b'"',
        (0x29, true) => b'~',
        (0x2B, true) => b'|',
        (0x2C, true) => b'Z',
        (0x2D, true) => b'X',
        (0x2E, true) => b'C',
        (0x2F, true) => b'V',
        (0x30, true) => b'B',
        (0x31, true) => b'N',
        (0x32, true) => b'M',
        (0x33, true) => b'<',
        (0x34, true) => b'>',
        (0x35, true) => b'?',
        (0x39, true) => b' ',
        _ => return None,
    };

    Some(byte)
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    // SAFETY: `in` reads one byte from the specified I/O port. The caller is
    // responsible for using a port that belongs to the active hardware.
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags)
    );

    value
}
