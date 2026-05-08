//! Serial logging for early debugging over QEMU's virtual COM1 port.

use core::fmt;

use crate::spinlock::SpinLock;

const COM1_PORT: u16 = 0x3F8;

struct SerialPort {
    base: u16,
    initialized: bool,
}

impl SerialPort {
    const fn empty() -> Self {
        Self {
            base: COM1_PORT,
            initialized: false,
        }
    }

    fn init(&mut self) {
        if self.initialized {
            return;
        }

        // SAFETY: These are the standard 16550 UART register offsets for COM1.
        // QEMU exposes this device at the classic PC port base `0x3F8`, so
        // writing these values configures the emulated serial port directly.
        unsafe {
            outb(self.base + 1, 0x00);
            outb(self.base + 3, 0x80);
            outb(self.base + 0, 0x03);
            outb(self.base + 1, 0x00);
            outb(self.base + 3, 0x03);
            outb(self.base + 2, 0xC7);
            outb(self.base + 4, 0x0B);
        }

        self.initialized = true;
    }

    fn write_byte(&mut self, byte: u8) {
        if !self.initialized {
            return;
        }

        while !self.transmit_ready() {}

        // SAFETY: Once COM1 is initialized, writing to the base port sends the
        // next byte to the UART transmit register.
        unsafe {
            outb(self.base, byte);
        }
    }

    fn transmit_ready(&self) -> bool {
        // SAFETY: Reading line status from `base + 5` is part of the standard
        // 16550 UART programming model and is safe in this single-core setup.
        unsafe { inb(self.base + 5) & 0x20 != 0 }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            match byte {
                b'\n' => {
                    self.write_byte(b'\r');
                    self.write_byte(b'\n');
                }
                other => self.write_byte(other),
            }
        }

        Ok(())
    }
}

static SERIAL1: SpinLock<SerialPort> = SpinLock::new(SerialPort::empty());

pub fn init() {
    SERIAL1.lock_irq().init();
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    let _ = SERIAL1.lock_irq().write_fmt(args);
}

unsafe fn outb(port: u16, value: u8) {
    // SAFETY: `out` writes one byte to the specified I/O port. Callers must
    // ensure the port is valid for the active hardware configuration.
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    // SAFETY: `in` reads one byte from the specified I/O port. Callers must
    // ensure the port is valid for the active hardware configuration.
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags)
    );

    value
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\n")
    };
    ($fmt:expr) => {
        $crate::serial_print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::serial_print!(concat!($fmt, "\n"), $($arg)*)
    };
}
