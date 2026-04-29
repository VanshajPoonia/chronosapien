# Time Capsule OS Roadmap

This roadmap keeps the project small, readable, and beginner-friendly.

## Milestone 1: boot to a polling keyboard prompt

Goal:
- Build a Rust `no_std` kernel for `x86_64-unknown-none`
- Boot it in QEMU using the existing `bootloader` crate
- Print a structured Time Capsule OS banner through the console layer
- Poll the PS/2 keyboard and echo typed characters without interrupts

Status:
- This is the current baseline for the repo.

## Next milestones

1. **tiny shell**
   Add a small command loop with a fixed buffer and a few built-in commands.
2. **interrupts and timer**
   Set up the IDT and a periodic timer so the kernel stops being purely synchronous.
3. **memory management**
   Introduce paging concepts, frame allocation, and the first allocator pieces.
4. **filesystem and persistence**
   Add a tiny storage layer for settings or simple text data.
5. **theme switching**
   Make era selection interactive only after the text terminal is stable.
6. **graphics later**
   Explore a simple GUI after the text-first system feels comfortable.
