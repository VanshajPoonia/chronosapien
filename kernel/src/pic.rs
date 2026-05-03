//! Legacy 8259 Programmable Interrupt Controller setup.
//!
//! The first 32 IDT vectors belong to CPU exceptions, so Chronosapian remaps the
//! master PIC to vector 32 and the slave PIC to vector 40 before enabling IRQs.

pub const MASTER_PIC_OFFSET: u8 = 32;
pub const SLAVE_PIC_OFFSET: u8 = 40;
pub const TIMER_IRQ: u8 = 0;
pub const MOUSE_IRQ: u8 = 12;

const PIC_EOI: u8 = 0x20;

const MASTER_COMMAND_PORT: u16 = 0x20;
const MASTER_DATA_PORT: u16 = 0x21;
const SLAVE_COMMAND_PORT: u16 = 0xA0;
const SLAVE_DATA_PORT: u16 = 0xA1;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

const MASTER_HAS_SLAVE_ON_IRQ2: u8 = 0x04;
const SLAVE_CASCADE_IDENTITY: u8 = 0x02;

const MASTER_MASK_TIMER_AND_SLAVE: u8 = 0b1111_1010;
const SLAVE_MASK_MOUSE_ONLY: u8 = 0b1110_1111;

pub fn init() {
    // SAFETY: These are the standard command/data ports for the legacy 8259
    // PICs on the PC-compatible machine QEMU emulates.
    unsafe {
        outb(MASTER_COMMAND_PORT, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(SLAVE_COMMAND_PORT, ICW1_INIT | ICW1_ICW4);
        io_wait();

        outb(MASTER_DATA_PORT, MASTER_PIC_OFFSET);
        io_wait();
        outb(SLAVE_DATA_PORT, SLAVE_PIC_OFFSET);
        io_wait();

        outb(MASTER_DATA_PORT, MASTER_HAS_SLAVE_ON_IRQ2);
        io_wait();
        outb(SLAVE_DATA_PORT, SLAVE_CASCADE_IDENTITY);
        io_wait();

        outb(MASTER_DATA_PORT, ICW4_8086);
        io_wait();
        outb(SLAVE_DATA_PORT, ICW4_8086);
        io_wait();

        outb(MASTER_DATA_PORT, MASTER_MASK_TIMER_AND_SLAVE);
        outb(SLAVE_DATA_PORT, SLAVE_MASK_MOUSE_ONLY);
    }
}

pub fn end_of_interrupt(irq: u8) {
    // SAFETY: EOI is the standard 8259 command. For IRQs from the slave PIC,
    // the slave must be acknowledged before the master cascade line.
    unsafe {
        if irq >= 8 {
            outb(SLAVE_COMMAND_PORT, PIC_EOI);
        }

        outb(MASTER_COMMAND_PORT, PIC_EOI);
    }
}

unsafe fn outb(port: u16, value: u8) {
    // SAFETY: The caller must ensure that the selected port belongs to the PIC.
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

unsafe fn io_wait() {
    // SAFETY: Port 0x80 is traditionally used as a tiny I/O delay on
    // PC-compatible hardware. The written value is ignored.
    outb(0x80, 0);
}
