# ChronoOS Roadmap

ChronoOS should stay small, readable, beginner-friendly, and honest about what
has only been implemented in code versus what has been runtime-verified.

## Current Status

- source of truth: use `docs/CURRENT_STATUS.md` for the post-Phase-4 feature
  table and verification state.
- verified in QEMU: single-core BIOS serial-only boot reached `[CHRONO] boot
  complete`; boot-time serial logging was observed through that point.
- implemented in code: BIOS boot path through the `bootloader` crate, optional custom BIOS handoff, optional UEFI loader, framebuffer console, serial logging, era themes, shell commands, IRQ keyboard buffering with polling fallback, PS/2 mouse input, small windows, PIT timer, GDT/IDT/PIC, basic memory management, reusable free-list heap, ATA storage, ChronoFS, `fsck`, tiny journal, apps, museum pages, quests, cooperative scheduler, SMP work, ARP/UDP networking, ring 3 demo, syscall layer, and static ELF execution.
- partially implemented: graphics shell, app platform, process model, scheduler, SMP, networking, ChronoFS recovery, and real hardware support.
- needs runtime verification: visible framebuffer output, shell prompt/input, custom BIOS boot, UEFI boot, input devices, windows, heap reuse, ChronoFS recovery, userspace, networking, product/app commands, and SMP behavior.
- roadmap/design-only: TCP, DHCP, DNS, USB, dynamic linker, package manager, full compositor, and preemptive scheduler.

## Immediate Milestone: Stabilize And Verify

1. Build the kernel and fix any compile errors narrowly.
2. Boot the normal BIOS image in QEMU.
3. Verify framebuffer and serial output.
4. Verify shell input, IRQ keyboard behavior, and polling fallback.
5. Verify ChronoFS basics, `fsck`, and journal status.
6. Record evidence in `docs/AI_PROGRESS_LOG.md`.

## Product Milestone: Show ChronoOS Clearly

1. Verify `demo`, `tour`, `capsule`, `doctor`, `poster`, `travel <year>`, and `apps`.
2. Capture screenshots only after runtime behavior is actually observed.
3. Keep the showcase conservative until real evidence exists.
4. Continue separating implemented-in-code from runtime-verified.

## Later Technical Milestones

1. Strengthen ChronoFS recovery tests and documentation.
2. Improve userspace examples without adding a dynamic linker.
3. Expand shell apps in small terminal-first steps.
4. Improve window/file browsing only within the current lightweight UI model.
5. Design, but do not rush, preemption and broader networking.

## Phase 4 Long-Term Tracks

These tracks are deliberately roadmap/design-only unless a future audit records
matching code and real verification evidence.

### Track A: Networking

- Current boundary: static IPv4 ARP/UDP over RTL8139 is partially implemented.
- Next safe work: verify `net status`, `net config`, `net arp`, `net log`, and
  `net send` in QEMU before adding protocols.
- Later ideas: DHCP, DNS, TCP, a small socket-like API, and better packet
  observability.
- Do not overbuild yet: no TCP/DHCP/DNS work before ARP/UDP is runtime-proven.

### Track B: USB And Real Hardware

- Current boundary: BIOS and UEFI teaching paths exist; no USB stack is
  implemented.
- Next safe work: verify BIOS/UEFI boot and document hardware-specific failures.
- Later ideas: USB HID, USB storage, USB serial, and real-hardware boot
  hardening.
- Do not overbuild yet: keep USB as design-only until QEMU boot/input/storage
  evidence is strong.

### Track C: Scheduler, Process, And User Mode

- Current boundary: cooperative scheduler, ring 3 demo, tiny syscall ABI, and
  static ELF teaching paths exist.
- Next safe work: verify `tasks`, `kill`, `ring3`, `syshello`, and `exec` one at
  a time.
- Later ideas: stronger process lifecycle, safer user memory boundaries,
  multi-process demos, and preemptive scheduling.
- Do not overbuild yet: preemption and broad userland should wait until current
  transitions are stable.

### Track D: App Loading And Package Model

- Current boundary: built-in apps and static ELF execution exist; `apps` is a
  text-first launcher.
- Next safe work: verify notes/calc/sysinfo and static `exec` behavior.
- Later ideas: richer app metadata, app files, and a tiny package/app manager.
- Do not overbuild yet: no dynamic linker or package manager before static ELF
  and ChronoFS workflows are proven.

### Track E: GUI And Compositor

- Current boundary: framebuffer text UI, mouse path, and small fixed-capacity
  windows exist.
- Next safe work: verify visible framebuffer output, pointer movement, window
  focus, drag, close, and `open notes` / `open sysinfo`.
- Later ideas: file explorer window mode, tiny paint, theme studio, visual boot
  timeline, and richer launcher.
- Do not overbuild yet: no full compositor or GUI toolkit before the small
  window layer is runtime-proven.

## Not Current Goals

- TCP, DHCP, DNS, or a full socket stack
- USB HID/storage/serial
- Dynamic linker
- Package manager
- Full desktop compositor
- Browser
- Production-grade preemptive scheduler
