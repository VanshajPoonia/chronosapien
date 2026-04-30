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

## CPU exception handling

ChronoOS loads its own Global Descriptor Table (GDT) during early boot. In
x86_64 long mode, old-style segmentation is mostly disabled, but the CPU still
needs a valid code segment. The GDT also contains the Task State Segment (TSS),
which gives the CPU a dedicated stack to use for especially dangerous
exceptions.

The Interrupt Descriptor Table (IDT) is the CPU's exception and interrupt
vector table. ChronoOS registers Rust handlers for breakpoint, page fault, and
double fault entries, then loads the table with `lidt`. These handlers use
Rust's `extern "x86-interrupt"` ABI so the compiler preserves the stack layout
that the CPU expects.

A double fault happens when the CPU encounters a second exception while trying
to deliver or handle an earlier one. If the normal stack is already corrupted,
handling the double fault on that same stack can immediately become a triple
fault and reset the machine. ChronoOS assigns double faults a separate TSS
Interrupt Stack Table entry so the handler has a clean stack to report the
failure.

## What still hides low-level behavior

- `bootloader` still hides the CPU mode switch, stack setup, paging setup, and the exact boot handoff details.
- `bootimage` hides host-side image packaging, but it does not hide the runtime behavior of the kernel inside QEMU.

## What to replace later for more ownership

If you want to own more of the boot path later, replace pieces in this order:

1. Replace the `BootInfo`-based handoff with your own chosen boot protocol.
2. Replace the borrowed bootloader with a more explicit protocol such as Multiboot2 or Limine.
3. Replace that protocol with your own bootloader stages and long-mode entry path.
