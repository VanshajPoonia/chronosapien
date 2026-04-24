# Architecture Notes

Time Capsule OS is split into **our code** and **borrowed infrastructure** on purpose.

## Ours

- The kernel entrypoint in `kernel/src/main.rs`
- The panic path in `kernel/src/panic.rs`
- The VGA text writer in `kernel/src/vga_text/`
- The era model in `kernel/src/theme.rs`
- The startup messages, banner, and prompt preview
- The PowerShell helper scripts and all documentation

## Borrowed infrastructure

- The `bootloader` crate, which performs the hard early boot steps and jumps into our kernel
- QEMU, which emulates the machine we are developing against
- The Rust compiler, core library, and bootimage tooling
- The `uart_16550` crate, which hides some serial-port register setup
- The `spin` crate, which provides a tiny lock for sharing the VGA and serial writers safely

## Why this split is good for learning

We keep the early boot plumbing borrowed so you can focus on kernel code first: output, input, interrupts, memory, and the shell. As the project grows, you can choose to replace more infrastructure once the core ideas feel comfortable.

## What still hides low-level behavior

- `bootloader` is still the main abstraction. It hides the CPU mode switch, stack setup, paging setup, and the exact boot image handoff details.
- `bootimage` hides some host-side image packaging, but it does not hide runtime kernel logic inside QEMU.

## What to replace later for more ownership

If you want to own more of the boot path later, replace pieces in this order:

1. Replace the `BootInfo`-based handoff with your own chosen boot protocol.
2. Replace the borrowed bootloader with a protocol like Multiboot2 or Limine that is still external but more explicit.
3. Replace that protocol with your own bootloader stages and your own long-mode entry path.
