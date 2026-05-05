//! Tiny x86 port I/O helpers shared by device drivers.

pub unsafe fn outb(port: u16, value: u8) {
    // SAFETY: The caller must ensure `port` belongs to a byte-wide device
    // register and that writing `value` is valid for the current hardware.
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

pub unsafe fn outw(port: u16, value: u16) {
    // SAFETY: Same as `outb`, but for word-wide port registers.
    core::arch::asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") value,
        options(nomem, nostack, preserves_flags)
    );
}

pub unsafe fn outl(port: u16, value: u32) {
    // SAFETY: Same as `outb`, but for doubleword-wide port registers.
    core::arch::asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") value,
        options(nomem, nostack, preserves_flags)
    );
}

pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    // SAFETY: The caller must ensure `port` belongs to a byte-wide device
    // register and that reading it is valid for the current hardware.
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags)
    );

    value
}

pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;

    // SAFETY: Same as `inb`, but for word-wide port registers.
    core::arch::asm!(
        "in ax, dx",
        in("dx") port,
        out("ax") value,
        options(nomem, nostack, preserves_flags)
    );

    value
}

pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;

    // SAFETY: Same as `inb`, but for doubleword-wide port registers.
    core::arch::asm!(
        "in eax, dx",
        in("dx") port,
        out("eax") value,
        options(nomem, nostack, preserves_flags)
    );

    value
}
