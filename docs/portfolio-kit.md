# ChronoOS Portfolio Kit

Use this page when describing ChronoOS on a portfolio site, GitHub profile,
resume, interview loop, or build-in-public thread. Keep runtime claims aligned
with `docs/VERIFICATION_MATRIX.md`.

## 1-Sentence Pitch

ChronoOS is an educational Rust `no_std` x86_64 hobby OS that turns low-level
systems work into a shell-first time-museum learning experience.

## 2-Sentence Pitch

ChronoOS is a Rust `no_std` x86_64 educational hobby OS with boot, memory,
interrupt, framebuffer, storage, app, userspace, and networking teaching paths.
Its product layer makes the kernel feel explorable through eras, museum pages,
quests, ChronoFS tools, status screens, and conservative verification docs.

## Longer Case-Study Pitch

ChronoOS started as a low-level Rust operating-system project and grew into a
small educational product. The kernel explores real OS foundations such as BIOS
boot, serial and framebuffer output, interrupts, timers, PS/2 input paths,
heap allocation, ATA storage, ChronoFS, cooperative tasks, early SMP work, Ring
3 demos, syscalls, static ELF loading, and static IPv4 ARP/UDP networking.

The product idea is what makes it memorable: the shell is a museum lobby and
command center. Users can switch eras, follow guided learning paths, inspect
current status, open small app surfaces, try filesystem commands, and read
honest labels that separate code-present work from QEMU-tested evidence.

## Problem Statement

Operating-system projects are often impressive but hard to explain. ChronoOS
tries to solve that by pairing real low-level code with beginner-friendly
in-OS explanations, demo scripts, screenshots checklists, and a verification
matrix that makes the project easier to trust.

## What I Built

- A Rust `no_std` x86_64 educational OS with BIOS boot and optional UEFI/custom
  boot paths in the source tree.
- A shell-first product surface with eras, onboarding, guide, learn, demo, tour,
  capsule, poster, doctor, museum, and quest commands.
- ChronoFS, a tiny educational filesystem with shell commands, fsck diagnostics,
  conservative repair behavior, journal status, and hardening docs.
- Static app surfaces and a static app registry for notes, calc, sysinfo, files,
  museum, tasks, userspace, networking, timeline, and roadmap entries.
- Read-only observability surfaces for userspace, windows, networking, safe
  mode, and verification status.

## Technical Highlights

- Rust `no_std` kernel code for x86_64.
- Serial and framebuffer output paths.
- GDT/IDT/PIC/PIT setup and interrupt-oriented input paths.
- Free-list heap allocator and boot memory-map handling.
- ATA-backed ChronoFS with fixed file slots and compact consistency checks.
- Cooperative task/window lifecycle surfaces.
- Ring 3, syscall, and static ELF teaching paths.
- RTL8139/static IPv4/ARP/UDP networking observability.
- QEMU-first verification discipline with logs and screendumps recorded in docs.

## Product Highlights

- Era themes make the OS feel like a small time machine.
- Museum and learning-path commands explain boot, memory, interrupts,
  filesystems, userspace, networking, scheduling, and roadmap work.
- Safe/demo/experimental mode helps separate clean demos from risky labs.
- Demo scripts and release docs make the project easy to present honestly.
- The verification matrix keeps the portfolio story impressive without making
  unsupported runtime claims.

## Learning Outcomes

- How boot flow, kernel setup, interrupts, timers, input, memory, storage, and
  shell UX fit together in one small OS.
- How to keep a hobby OS explainable with source-truth docs and status labels.
- How to build product-minded developer tools without pretending a teaching OS
  is a production platform.
- How to separate implementation, partial implementation, QEMU evidence, blocked
  tooling, and roadmap-only work.

## Known Limitations

- ChronoOS is an educational hobby OS, not a Linux replacement.
- Runtime verification is incomplete; use `docs/VERIFICATION_MATRIX.md` for the
  exact evidence level.
- ChronoFS workflows, userspace/syscalls/ELF, ARP/UDP behavior, broad app and
  window behavior, UEFI, custom BIOS, SMP/AP, GIF capture, and hardware still
  need focused verification unless later evidence is recorded.
- TCP, DHCP, DNS, USB, a dynamic linker, package manager, full compositor, and
  production-grade preemptive scheduler are roadmap/design-only.

## Screenshots And GIF Checklist

Capture only real QEMU or hardware output. Use `docs/screenshots.md` for naming.

- Boot screen and first visible shell prompt.
- `start`, `guide quick`, `learn`, and `help`.
- Era views: `era 1984`, `era 1995`, `era 2007`, `era 2040`.
- Museum and learning pages such as `museum filesystem` and `learn userspace`.
- App launcher and app cards: `apps`, `apps info notes`, `apps roadmap`.
- ChronoFS: `fs status`, `fs info`, `fs check`, `journal`, and controlled file
  workflows.
- Status surfaces: `doctor`, `mode status`, `userspace status`, `net status`.
- Window surfaces: `open notes`, `windows list`, `windows status`, if verified.

## Suggested Demo Flow

1. Start with `mode status`, then `mode safe`.
2. Run `start`, `guide quick`, `learn`, and `about`.
3. Show eras with `era` and `travel 1998`.
4. Show education with `learn boot`, `museum filesystem`, and `tour files`.
5. Show product surfaces with `apps`, `poster`, `doctor`, and `capsule current`.
6. Show system boundaries with `fs status`, `userspace status`, and `net status`.
7. Close with `docs/VERIFICATION_MATRIX.md`: explain what has evidence and what
   remains intentionally unverified or roadmap-only.
