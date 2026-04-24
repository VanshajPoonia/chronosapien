# Boot Flow

The initial boot flow is intentionally small and readable.

1. `cargo bootimage -p kernel` builds the `kernel` crate for `x86_64-unknown-none`.
2. The bootimage tooling wraps that kernel into a disk image that QEMU can boot.
3. QEMU starts an emulated x86_64 machine and loads the bootable image.
4. The borrowed Rust bootloader handles the early machine setup we are not writing yet.
5. The bootloader jumps to our kernel entrypoint in `kernel/src/main.rs`.
6. Our kernel initializes a hand-written COM1 serial logger, configures the VGA theme, prints the boot text and welcome banner, and then enters a tiny shell loop.
7. The shell polls the PS/2 controller status port `0x64`, reads keyboard bytes from data port `0x60`, decodes a small set of keyboard set-1 scancodes, and fills a fixed command buffer.
8. Pressing Enter runs a built-in command such as `help`, `clear`, or `about`.

When interrupts are introduced later, this polling loop should move behind a keyboard interrupt handler and a queue so input is no longer read synchronously in the shell.

This gives us a raw kernel we control without getting stuck in assembly-heavy bootstrapping on day one.
