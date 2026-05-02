# Chronosapian Roadmap

This roadmap keeps the project small, readable, and beginner-friendly.

## Milestone 1: boot to a polling keyboard prompt

Goal:
- Build a Rust `no_std` kernel for `x86_64-unknown-none`
- Boot it in QEMU using the existing `bootloader` crate
- Print a structured Chronosapian banner through the console layer
- Poll the PS/2 keyboard and echo typed characters without interrupts

Status:
- This is the current baseline for the repo.

## Next milestones

1. **tiny shell commands**
   Add a small command dispatcher with a fixed buffer and built-ins like `help`, `clear`, and `about`.
2. **interrupts and timer**
   Set up the IDT and a periodic timer so the kernel stops being purely synchronous.
3. **memory management**
   Introduce paging concepts, frame allocation, and the first allocator pieces.
4. **filesystem and persistence**
   Grow the current in-memory file list into persistent storage.
5. **theme switching**
   Make era selection interactive only after the text terminal is stable.
6. **graphics shell**
   Grow the current framebuffer console into richer graphical surfaces.
