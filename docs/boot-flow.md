# Boot Flow

The initial boot flow is intentionally small and readable.

1. `cargo bootimage -p kernel` builds the `kernel` crate for `x86_64-unknown-none`.
2. The bootimage tooling wraps that kernel into a disk image that QEMU can boot.
3. QEMU starts an emulated x86_64 machine and loads the bootable image.
4. The borrowed Rust bootloader handles the early machine setup we are not writing yet.
5. The bootloader jumps to our kernel entrypoint in `kernel/src/main.rs`.
6. Our kernel initializes a small COM1 serial logger, configures the VGA theme, logs the boot sequence, prints the Time Capsule OS startup banner, and enters a polling keyboard input loop.

The VGA banner is:

```text
TIME CAPSULE OS
Era: 1984
CHRONO/84> _
```

The QEMU terminal shows:

```text
[CHRONO] boot start
[CHRONO] serial initialized
[CHRONO] console initialized
[CHRONO] active era: 1984
[CHRONO] boot complete
```

After that, typed keys are echoed on the VGA screen and logged to serial. Enter
submits the current fixed-size input buffer and starts a fresh prompt.

This gives us a raw kernel we control without getting stuck in assembly-heavy bootstrapping on day one.
