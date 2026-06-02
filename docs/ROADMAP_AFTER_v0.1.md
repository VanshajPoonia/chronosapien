# Roadmap After ChronoOS v0.1

This roadmap starts after the v0.1 release candidate package. The priority is
to verify and harden what already exists before adding large new systems.

## 1. Reliability And Verification

- Run a visible BIOS QEMU pass for framebuffer output, first shell prompt,
  keyboard input, categorized `help`, onboarding, and conservative status
  commands.
- Record screenshots, serial logs, exact QEMU command, date, and result in
  `docs/AI_PROGRESS_LOG.md`.
- Verify product commands: `start`, `guide quick`, `demo`, `tour`, `capsule`,
  `doctor`, `poster`, `travel <year>`, and `apps`.
- Investigate the two-core SMP/AP boot exit separately from the single-core
  release path.
- Keep status labels conservative until evidence exists.

## 2. Filesystem Hardening

- Verify ChronoFS basics in QEMU: `ls`, `write`, `cat`, `rm`, `fsck`, and
  `journal`.
- Test `fsck repair` only with controlled disk images.
- Create repeatable journal/recovery scenarios for intent, committed, corrupt,
  and ambiguous states.
- Document exact repair boundaries before expanding recovery behavior.

## 3. Userspace And Process Model

- Verify `ring3`, `syshello`, and `exec <name>` one at a time.
- Keep the model educational until return-to-shell, fault, syscall, and static
  ELF behavior is predictable.
- Improve process lifecycle documentation before adding broad userland.
- Do not add a dynamic linker, package manager, libc, argv/env, or large process
  model until static ELF behavior is proven.

## 4. Networking Expansion

- Verify `net`, `net arp`, and `net send` with a recorded QEMU RTL8139 setup.
- Add packet-observation notes before adding protocols.
- Keep current scope to static IPv4 ARP/UDP until it is runtime-proven.
- DHCP, DNS, TCP, sockets, and broader hardware support remain future work.

## 5. UI And Window Shell Polish

- Verify visible framebuffer output, cursor behavior, top bar, and shell
  redraws.
- Verify PS/2 mouse movement, window focus, drag, close, `open notes`,
  `open sysinfo`, `tasks`, and `kill <id>`.
- Keep improvements terminal-first and small.
- Defer full compositor, GUI toolkit, and windowed file explorer until the
  small window layer is reliable.

## 6. Real Hardware And USB Exploration

- Verify BIOS and UEFI boot paths in QEMU before real hardware claims.
- Record hardware experiments as experiments, not release guarantees.
- Defer USB HID, USB storage, and USB serial until boot/input/storage evidence
  is stronger.
- Prefer small, reversible hardware probes with clear logs.

## 7. Long-Term Advanced Systems

- Future networking: DHCP, DNS, TCP, sockets, and better packet tooling.
- Future userspace: richer process lifecycle, safer memory boundaries, app
  metadata, and eventually a tiny package/app model.
- Future UI: theme studio, crash lab, tiny paint, visual boot timeline, stronger
  launcher, and richer era-specific behavior.
- Future scheduler: stronger cooperative behavior first, then carefully designed
  preemption later.
- Future hardware: USB HID/storage/serial only after current QEMU and UEFI
  evidence is solid.
