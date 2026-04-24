# Time Capsule OS Roadmap

This roadmap keeps the project moving in a learning-first order.

1. **boot**
   Reach a reliable QEMU boot into our Rust kernel and understand the boot handoff.
2. **text output**
   Print clear status text to the VGA buffer and serial output for debugging.
3. **keyboard input**
   Read key presses from the keyboard controller, decode scancodes, and feed a shell input buffer.
4. **shell**
   Add a tiny command loop with built-in commands like `help`, `clear`, and `about`.
5. **era switching**
   Let the shell switch between era profiles without rebooting the machine.
6. **interrupts**
   Set up the IDT and handle hardware/software interrupts safely.
7. **timer**
   Configure a periodic timer so the kernel can track time and schedule periodic work.
8. **memory management**
   Build paging and frame-allocation concepts into explicit, readable modules.
9. **allocator**
   Add a simple heap allocator so dynamic data structures become possible.
10. **filesystem**
    Create a tiny readable filesystem layer for storing commands, text, or settings.
11. **simple GUI later**
    Explore a small graphical desktop only after the terminal-first kernel is comfortable.
