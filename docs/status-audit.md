# ChronoOS Status Audit

Date: 2026-06-01

This file is the Phase 2 source-truth audit for systems that are implemented in
code, partially implemented, risky, stale, or unverified. It does not claim
runtime success. Upgrade any row to `verified in QEMU` or `verified on hardware`
only after recording actual evidence in `docs/AI_PROGRESS_LOG.md`.

## Status Labels

- implemented in code: source paths or shell commands exist.
- partially implemented: useful teaching version with known limits.
- risky: low-level, timing-sensitive, hardware-sensitive, or build-blocked.
- stale docs only: documentation exists without matching source.
- needs runtime verification: QEMU or hardware evidence is still required.
- verified in QEMU: actual QEMU evidence is recorded.
- verified on hardware: actual hardware evidence is recorded.
- roadmap/design-only: intentionally not implemented yet.

## Phase 2 Systems

| System | Repo status | Runtime status | Notes |
| --- | --- | --- | --- |
| Custom BIOS bootloader path | partially implemented, risky | needs runtime verification | Stage 1/Stage 2 code, scripts, docs, and `chrono_custom_entry` handoff support exist. Do not claim boot success without QEMU serial/framebuffer evidence. |
| Window manager and app platform | partially implemented | needs runtime verification | Small fixed-capacity windows, notes/sysinfo window paths, focus, drag, close, task wiring, and app launcher code exist. This is not a full desktop, compositor, or GUI toolkit. |
| Cooperative scheduler | implemented in code, partially implemented | needs runtime verification | Fixed task slots, spawn/kill/yield, task listing, and simple core assignment exist. It is cooperative only. |
| SMP/AP startup | partially implemented, risky | needs runtime verification | MADT discovery and INIT-SIPI-SIPI startup code exist, but AP startup is hardware-sensitive and should be tested separately from the scheduler. |
| Ring 3 demo | partially implemented, risky | needs runtime verification | The `ring3` path demonstrates a teaching transition and fault path. It is not a general process model. |
| Syscall layer | partially implemented, risky | needs runtime verification | Tiny syscall ABI exists for write/read/exit/uptime. Previous progress notes record build issues in low-level userspace/syscall paths, so build verification comes before runtime claims. |
| Static ELF execution | partially implemented, risky | needs runtime verification | Static ELF64 parsing/loading exists for one foreground program. No dynamic linker, argv/env, libc, permissions model, or general process table. |
| ChronoFS journal/recovery | implemented in code, risky | needs runtime verification | One-record journal, mount recovery, bitmap rebuild, `fsck`, and conservative repair exist. Recovery refuses unsafe/corrupt cases and still needs controlled crash-state testing. |
| Networking | partially implemented | needs runtime verification | RTL8139, static IPv4, ARP, UDP send, and polling receive exist. No DHCP, DNS, TCP, sockets, or broad hardware support. |
| Notes v2 | implemented in code | needs runtime verification | Shell notes commands and notes window now use the `notes` ChronoFS file. The shell handles direct `notes` commands before the app router. |
| Museum/quest content | implemented in code | needs runtime verification | Product/museum/quest commands exist and use conservative status language for unverified systems. |
| Mouse/window interaction | partially implemented | needs runtime verification | PS/2 IRQ12 packet decoding, cursor drawing, window focus, drag, and close paths exist. No runtime evidence is recorded yet. |
| Heap allocator correctness | implemented in code, risky | needs runtime verification | Free-list allocation, free, sorted reinsertion, and coalescing exist. Double-free/corruption guards are not present. |
| IRQ keyboard behavior | implemented in code, risky | needs runtime verification | IRQ1 buffering and polling fallback exist. Needs real shell typing, shifted characters, backspace, enter, and fallback checks. |

## Runtime Verification Audit

Build sanity on 2026-06-01:

- `cargo check -p kernel --offline --locked`: passed with no warnings.
- `cargo check -p chronosapien --target aarch64-apple-darwin --offline --locked`: passed with no warnings.
- Runtime tools installed and verified: QEMU 11.0.1 and PowerShell 7.6.2.

Runtime evidence on 2026-06-01:

- Single-core BIOS serial-only QEMU smoke reached `[CHRONO] boot complete`.
- The smoke used `-display none`, so framebuffer rendering and visible shell prompt were not verified.
- A two-core BIOS serial-only QEMU smoke exited before `[CHRONO] boot complete`, after `[CHRONO] active era: 1984`; keep SMP/AP startup marked risky and unverified.

| Audit item | Repo status | Verification status | Next evidence needed |
| --- | --- | --- | --- |
| BIOS boot path | implemented in code | verified in QEMU, needs broader runtime verification | Single-core serial-only QEMU reached `[CHRONO] boot complete`. Framebuffer screenshot, shell prompt, keyboard input, and multi-core boot remain unverified. |
| UEFI boot path | implemented in code | needs runtime verification | QEMU OVMF boot log, GOP framebuffer handoff, shell prompt, and serial handoff lines. |
| Framebuffer output | implemented in code | needs runtime verification | Visible text, top bar, clear/redraw behavior, and screenshot. |
| Serial output | implemented in code | verified in QEMU, needs broader runtime verification | Serial-only QEMU captured bootloader and kernel boot logs through `[CHRONO] boot complete`. Shell command output still needs verification. |
| PIT/timer behavior | implemented in code | needs runtime verification | `uptime`/`clock` advancing, no timer regression under shell idle, and serial timer init. |
| Shell startup | implemented in code | needs runtime verification | Prompt appears and basic commands execute after boot. |
| Basic app launch | implemented in code, partially implemented | needs runtime verification | `apps`, `notes`, `calc`, `sysinfo`, `open notes`, and `open sysinfo` observed. |
| Filesystem shell commands | implemented in code | needs runtime verification | `ls`, `write`, `cat`, `rm`, persistence, `fsck`, `fsck repair`, and `journal` observed. |
| Product/demo commands | implemented in code | needs runtime verification | `demo`, `tour`, `capsule`, `doctor`, `poster`, `travel <year>`, and `apps` observed. |

## Current Boundaries

- `verified in QEMU`: single-core BIOS serial boot and boot-time serial logging reached `[CHRONO] boot complete`.
- `verified on hardware`: no Phase 2 target system has recorded proof in this repo.
- `stale docs only`: no Phase 2 target system should remain docs-only after this audit; future stale claims should be fixed or logged here.
- `roadmap/design-only`: TCP, DHCP, DNS, USB HID/storage/serial, dynamic linker, package manager, full compositor, and production-grade preemptive scheduling.
