# Boot Flow

The initial boot flow is intentionally small and readable.

## BIOS Path

1. `.\scripts\build.ps1` builds the `kernel` crate for `x86_64-unknown-none`.
2. The bootloader 0.11 BIOS builder wraps that kernel into a disk image that QEMU can boot.
3. QEMU starts an emulated x86_64 machine and loads the bootable image.
4. The borrowed Rust bootloader handles the early machine setup we are not writing yet.
5. The bootloader jumps to our kernel entrypoint in `kernel/src/main.rs`.
6. Our kernel initializes a small COM1 serial logger, configures the framebuffer console, loads the GDT and IDT, triggers one test breakpoint exception, initializes the heap from the bootloader memory map, remaps the PIC, starts the PIT timer, plays an era-specific PC speaker chime, logs the boot sequence, prints the Chronosapian startup banner, and enters a polling keyboard input loop.

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
[CHRONO] smp: BSP online (core 0)
[CHRONO] timer: PIT initialized at 100Hz
[CHRONO] active era: 1984
[CHRONO] smp: core 1 online
[CHRONO] smp: 2 cores ready
[CHRONO] sound: beep 880hz 90ms
[CHRONO] sound: beep 660hz 90ms
[CHRONO] sound: beep 440hz 140ms
[CHRONO] keyboard initialized
[CHRONO] boot complete
```

After that, typed keys are rendered in the framebuffer shell region and logged
to serial. Enter submits the current fixed-size input buffer and starts a fresh
prompt.

The `uptime` command reports elapsed seconds from the 100 Hz PIT tick counter,
and the `clock` command reports the raw tick count.

The `beep <hz>` command programs PIT channel 2, opens the PC speaker gate on
port `0x61`, and plays the requested tone for 500ms.

The `cores` command reports the number of online CPU cores and how many
cooperative tasks are assigned to each one.

The `mem` command reports total memory from the bootloader memory map and shows
the fixed 1MiB bump heap at `0x200000`.

This gives us a raw kernel we control without getting stuck in assembly-heavy bootstrapping on day one.

## UEFI Path

The UEFI build keeps the same kernel but replaces BIOS startup with a Rust UEFI
application:

1. `.\scripts\build-uefi.ps1` builds `kernel`, builds `uefi-loader` for
   `x86_64-unknown-uefi`, and creates a GPT/FAT32 image.
2. The image contains an EFI System Partition with `EFI\BOOT\BOOTX64.EFI` and
   `CHRONO\KERNEL.ELF`.
3. UEFI firmware or QEMU OVMF loads `BOOTX64.EFI` from the ESP.
4. The loader uses Boot Services to read the kernel ELF, allocate `PT_LOAD`
   segments, configure a GOP framebuffer, read the ACPI RSDP, and prepare
   ChronoOS page tables.
5. The loader calls `ExitBootServices()` only after all firmware allocations are
   finished. The final memory map returned by that call is copied into the v2
   ChronoOS handoff.
6. The loader logs the handoff, loads the new `CR3`, and jumps to
   `chrono_custom_entry`.

UEFI serial output starts like:

```text
[CHRONO] uefi: loader start
[CHRONO] uefi: framebuffer at 0x...
[CHRONO] uefi: handoff ok
[CHRONO] boot start
[CHRONO] serial initialized
...
```

BIOS starts with a tiny real-mode program and firmware interrupts. UEFI starts a
PE/COFF application with file protocols and GOP, then requires the loader to
leave firmware ownership with `ExitBootServices()` before the kernel takes over.
