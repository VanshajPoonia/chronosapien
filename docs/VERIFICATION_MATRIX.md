# ChronoOS Verification Matrix

Date: 2026-06-02

This is the compact evidence lookup for ChronoOS v0.1. It summarizes what has
actual QEMU or hardware evidence, what is only implemented in code, what is
blocked, and what remains roadmap/design-only.

Use this together with `docs/CURRENT_STATUS.md`, `docs/AI_PROGRESS_LOG.md`, and
`docs/release-checklist.md`. Do not upgrade a row unless the repo records the
actual command, log, screenshot, or hardware evidence.

## Verification Levels

- verified in QEMU
- partially verified in QEMU
- verified on hardware
- implemented in code, not verified
- blocked by tooling
- blocked by environment
- roadmap/design-only

No current feature is `verified on hardware`.

## Matrix

| Feature | Status | Verification level | Evidence | Command/test used | Notes |
| --- | --- | --- | --- | --- | --- |
| BIOS boot | implemented in code | verified in QEMU | Single-core BIOS serial-only and visible QEMU runs reached `[CHRONO] boot complete`. | `qemu-system-x86_64 -smp 1` with `chronosapien-bios.img`; logs `/private/tmp/chronoos-qemu-20260602-013807.serial.log` and related UI/input logs. | Multi-core BIOS remains separate from normal single-core boot. |
| UEFI boot | implemented in code | blocked by tooling | OVMF exists, but `uefi-loader` failed to compile because `uefi::boot::MemoryMap` is unresolved with the current `uefi` crate API. | `pwsh -NoLogo -NoProfile -File scripts/build-uefi.ps1`. | No UEFI QEMU boot was attempted after the build failure. |
| custom BIOS boot | partially implemented | blocked by tooling | `nasm` was not on PATH during preflight. | `command -v nasm`. | `scripts/build-custom.ps1` and `scripts/run-custom.ps1` were intentionally not run. |
| framebuffer shell | implemented in code | verified in QEMU | QEMU screendumps showed the ChronoOS top bar, boot text, prompt, and shell output. | QEMU monitor `screendump`; `/private/tmp/chronoos-qemu-20260602-013807-screendump.png` and `/private/tmp/chronoos-ui-input-20260602-150049-boot.png`. | Broader redraw and app/window behavior still need checks. |
| serial output | implemented in code | verified in QEMU | Serial logs show boot start, subsystem initialization, and boot complete. | `-serial file:/private/tmp/...serial.log` on single-core BIOS QEMU runs. | Shell-command serial output is only partially observed. |
| keyboard | implemented in code | partially verified in QEMU | QEMU monitor input submitted narrow shell commands including `help`, `help start`, `apps`, `notes`, `calc 6 - 7`, and `open notes`. | QEMU monitor `sendkey`. | Manual typing, Backspace, shifted input, and polling fallback remain unverified; some injected commands garbled. |
| mouse/windowing | partially implemented | partially verified in QEMU | Serial logged `mouse: click at 740,410`; `open notes` spawned a task, logged `wm: open notes`, and produced a visible window boundary. Shell lifecycle commands exist in code. | QEMU monitor mouse/key input; `/private/tmp/chronoos-ui-input-20260602-150049-open-notes-window.png`. | `windows list/status/focus/close`, cursor movement, drag, close, focus, and `open sysinfo` remain unverified. |
| apps | implemented in code | partially verified in QEMU | `apps` launcher, notes home screen, and `calc 6 - 7` result were observed; static app registry now exists in code. | QEMU monitor `sendkey`; screenshots `/private/tmp/chronoos-ui-input-20260602-150049-apps.png`, `...-notes-attempt.png`, and `...-calc.png`. | `apps list`, `apps info`, `apps launch`, `sysinfo`, notes read/write, and persistence remain unverified. |
| ChronoFS | implemented in code | implemented in code, not verified | Shell commands, read-only `fs` inspection, and ATA-backed filesystem paths exist in source/docs. | Not runtime-tested in recorded QEMU passes. | `fs status`, `fs info`, `ls`, `write`, `cat`, `rm`, and persistence still need evidence. |
| fsck/journal | implemented in code | implemented in code, not verified | `fs check`, `fsck`, `fsck repair`, `fs journal`, journal, and recovery paths exist in source/docs. | Not runtime-tested in recorded QEMU passes. | Repair/recovery should only be tested with controlled disk images; clean journal is not full filesystem proof. |
| heap allocator | implemented in code | implemented in code, not verified | Free-list allocator with split/free/reinsert/coalesce behavior exists in code. | Not runtime-tested in recorded QEMU passes. | Needs reuse/stress checks across shell, apps, tasks, and filesystem workflows. |
| scheduler | implemented in code | implemented in code, not verified | Cooperative task slots and task lifecycle paths exist in code. | Not runtime-tested beyond incidental task spawn during `open notes`. | Task scheduling behavior needs a focused `tasks`/`kill` verification pass. |
| SMP/AP | partially implemented, high-risk | partially verified in QEMU | Two-core smoke reached `smp: BSP online (core 0)` and `active era: 1984`, but no AP-online or two-core-ready evidence. | `qemu-system-x86_64 -smp 2 ... -display none`; log `/private/tmp/chronoos-smp-20260602-162000.serial.log`. | Keep high-risk until AP startup is actually observed. |
| Ring 3 | partially implemented | implemented in code, not verified | Ring 3 teaching path and `userspace status` inspection exist. | `ring3` was not run in recorded QEMU passes. | Fixed demo only; do not imply general userland. |
| syscalls | partially implemented | implemented in code, not verified | Tiny syscall layer and `userspace syscalls` table exist for write/read/exit/uptime. | `syshello` was not run in recorded QEMU passes. | Needs controlled userspace verification. |
| static ELF exec | partially implemented | implemented in code, not verified | Static ELF execution path and `userspace elf` boundary docs exist. | `exec <name>` was not run in recorded QEMU passes. | No dynamic linker, package model, argv/env, or libc. |
| networking | partially implemented | partially verified in QEMU | RTL8139 discovery and MAC `52:54:00:12:34:56` were observed; host UDP log was 0 bytes. New `net status/config/log/demo/roadmap` observability commands are implemented in code, not runtime-verified. | Single-core QEMU with RTL8139; log `/private/tmp/chronoos-net-20260602-162000.serial.log`; host log `/private/tmp/chronoos-net-20260602-162000.host-udp.log`. | ARP/UDP behavior was not verified because host forwarding conflicted and `net` input garbled. New counters are real code-path counters, not packet capture. |
| theme studio | roadmap/design-only | roadmap/design-only | Era profiles and `apps theme` preview exist, but no studio/editor workflow exists. | Not applicable. | Keep as future product polish. |
| crash lab | roadmap/design-only | roadmap/design-only | No crash lab command/app is implemented. | Not applicable. | Defer until controlled panic/fault evidence is stronger. |
| tiny paint | roadmap/design-only | roadmap/design-only | No paint canvas or drawing app is implemented. | Not applicable. | Future app idea only. |
| file explorer | roadmap/design-only | roadmap/design-only | `apps files` points to shell file commands; no windowed file explorer exists. | Not applicable. | Current file UX is shell-first. |
| boot chime | roadmap/design-only | roadmap/design-only | Era tones exist, but no user-facing boot chime selector exists. | Not applicable. | Future product polish only. |
| network demo | roadmap/design-only | roadmap/design-only | `net demo` is now a read-only shell guide for existing ARP/UDP commands; no packet-demo app or verified networking demo mode exists. | Not applicable. | DHCP, DNS, TCP, sockets, and packet capture remain roadmap/design-only. |
| user-space showcase | partially implemented | implemented in code, not verified | Ring 3, syscall, static ELF, museum, and tour teaching paths exist; no polished showcase app is verified. | `ring3`, `syshello`, and `exec` were not run in recorded QEMU passes. | Treat as teaching paths, not a general process platform. |
| visual boot timeline | partially implemented | implemented in code, not verified | `capsule` and `poster boot` text surfaces exist. | `capsule`/`poster boot` were not verified in recorded QEMU passes. | Graphical timeline remains roadmap/design-only. |
| onboarding/guide | implemented in code | implemented in code, not verified | `start`, `welcome`, and `guide` topic pages exist; `help start` was observed. | `help start` observed by QEMU monitor input; full `start`/`guide` flow not run. | Keep full onboarding as unverified until commands are observed. |
| status/verify center | implemented in code | partially verified in QEMU | `help system`/status-oriented help surfaces exist; `doctor`, `capsule current`, and poster status commands are not fully tested. | `help start` and grouped `help` observed; product status commands not cleanly observed. | There is no broad runtime certification command. |
| screenshots/GIFs | partially implemented process | partially verified in QEMU | QEMU `screendump` PNG capture worked; no GIF encoder was available. | QEMU monitor `screendump`; PNG paths listed in `docs/screenshots.md`. | GIF capture remains unverified/manual. |
| hardware | documented/testing path only | implemented in code, not verified | No hardware run, image write, or hardware serial log is recorded. | Not run. | Real hardware remains manual verification only; USB HID/storage/serial are roadmap/design-only. |

## Best Next Verification Focus

The safest next engineering target is not a new feature. It is a narrow
build-compatibility or verification step:

1. Fix the UEFI loader API mismatch, then retry single-core UEFI QEMU.
2. Or install NASM and run a custom BIOS build-only pass before booting it.
3. Or improve the QEMU shell input method, then verify ChronoFS and ARP/UDP one
   command at a time.
