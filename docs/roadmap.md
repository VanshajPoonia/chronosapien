# Time Capsule OS Roadmap

This roadmap keeps the project small, readable, and beginner-friendly.

## Milestone 1: boot to a welcome message

Goal:
- Build a Rust `no_std` kernel for `x86_64-unknown-none`
- Boot it in QEMU using the existing `bootloader` crate
- Print `Welcome to Time Capsule OS`

Status:
- This is the current baseline for the repo.

## Next milestones

1. **keyboard input**
   Read key presses from the keyboard controller and print simple feedback.
2. **tiny shell**
   Add a small command loop with a fixed buffer and a few built-in commands.
3. **interrupts and timer**
   Set up the IDT and a periodic timer so the kernel stops being purely synchronous.
4. **memory management**
   Introduce paging concepts, frame allocation, and the first allocator pieces.
5. **filesystem and persistence**
   Add a tiny storage layer for settings or simple text data.
6. **theme switching**
   Make era selection interactive only after the text terminal is stable.
7. **graphics later**
   Explore a simple GUI after the text-first system feels comfortable.
