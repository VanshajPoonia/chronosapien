# Boot Flow

The initial boot flow is intentionally small and readable.

1. `.\scripts\build.ps1` builds the `kernel` crate for `x86_64-unknown-none`.
2. The bootloader 0.11 BIOS builder wraps that kernel into a disk image that QEMU can boot.
3. QEMU starts an emulated x86_64 machine and loads the bootable image.
4. The borrowed Rust bootloader handles the early machine setup we are not writing yet.
5. The bootloader jumps to our kernel entrypoint in `kernel/src/main.rs`.
6. Our kernel initializes a small COM1 serial logger, configures the framebuffer console, loads the GDT and IDT, triggers one test breakpoint exception, initializes the heap from the bootloader memory map, remaps the PIC, starts the PIT timer, logs the boot sequence, prints the Chronosapian startup banner, and enters a polling keyboard input loop.

The graphical console starts with a top bar and shell region:

```text
Chronosapian | Era: 1984 | Uptime: 0s

EXCEPTION: BREAKPOINT
CHRONOSAPIAN
Era: 1984
CHRONO/84> _
```

The QEMU terminal shows:

```text
[CHRONO] boot start
[CHRONO] serial initialized
[CHRONO] fb: 1024x768 initialized
[CHRONO] console initialized
[CHRONO] GDT loaded
[CHRONO] IDT loaded
[CHRONO] interrupt: breakpoint at 0x...
[CHRONO] breakpoint resolved
[CHRONO] mem: heap initialized at 0x200000 size 1MB
[CHRONO] timer: PIT initialized at 100Hz
[CHRONO] active era: 1984
[CHRONO] keyboard initialized
[CHRONO] boot complete
```

After that, typed keys are rendered in the framebuffer shell region and logged
to serial. Enter submits the current fixed-size input buffer and starts a fresh
prompt.

The `uptime` command reports elapsed seconds from the 100 Hz PIT tick counter,
and the `clock` command reports the raw tick count.

The `mem` command reports total memory from the bootloader memory map and shows
the fixed 1MiB bump heap at `0x200000`.

This gives us a raw kernel we control without getting stuck in assembly-heavy bootstrapping on day one.
