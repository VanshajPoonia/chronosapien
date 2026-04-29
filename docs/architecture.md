# Architecture Notes

Time Capsule OS is split into **our code** and **borrowed infrastructure** on purpose.

## Ours

- The kernel entrypoint in `kernel/src/main.rs`
- The panic path in `kernel/src/panic.rs`
- The serial logger in `kernel/src/serial.rs`
- The public console layer in `kernel/src/console.rs`
- The polling keyboard reader in `kernel/src/keyboard.rs`
- The low-level VGA text writer in `kernel/src/vga_text/`
- The era model in `kernel/src/theme.rs`
- The startup welcome message
- The PowerShell helper scripts and documentation

## Borrowed infrastructure

- The `bootloader` crate, which performs the hard early boot steps and jumps into our kernel
- QEMU, which emulates the machine we are developing against
- The Rust compiler, core library, and bootimage tooling

## Why this split is good for learning

We keep the early boot plumbing borrowed so you can focus on kernel code first. That keeps Milestone 1 centered on a few core ideas: entrypoints, console output, panic handling, and the basic shape of a bare-metal Rust crate.

## COM1 serial logging

QEMU provides a virtual 16550-compatible serial device at COM1 port `0x3F8`.
When QEMU runs with `-serial stdio`, bytes written to that port appear in the
host terminal. Time Capsule OS uses that path for early boot logs and panic
messages because it still works even when VGA output is hard to inspect.

## What still hides low-level behavior

- `bootloader` still hides the CPU mode switch, stack setup, paging setup, and the exact boot handoff details.
- `bootimage` hides host-side image packaging, but it does not hide the runtime behavior of the kernel inside QEMU.

## What to replace later for more ownership

If you want to own more of the boot path later, replace pieces in this order:

1. Replace the `BootInfo`-based handoff with your own chosen boot protocol.
2. Replace the borrowed bootloader with a more explicit protocol such as Multiboot2 or Limine.
3. Replace that protocol with your own bootloader stages and long-mode entry path.
