# ChronoOS v0.1 RC - Time-Museum Shell

Date: 2026-06-02

Status: release candidate documentation package.

ChronoOS v0.1 RC presents the project as a beginner-friendly Rust `no_std`
x86_64 educational hobby OS with a product-minded shell. The release theme is
the time-museum shell: eras, guided onboarding, museum pages, quests, tiny apps,
ChronoFS tools, conservative status screens, and honest verification labels.

This is not a Linux replacement and not a production OS. It is a portfolio-ready
systems project that teaches OS concepts from inside the OS itself.

## Source-Truth Summary

- Public/product name: ChronoOS.
- Repo/package/image names: `chronosapien`, `chronosapien-bios.img`, and
  `chronosapien-uefi.img` may remain until a dedicated rename task.
- Current source of truth: `docs/CURRENT_STATUS.md`.
- Release checklist source: `docs/release-checklist.md`.
- Detailed manual checks: `docs/manual-testing.md`.

## Implemented In Code

- BIOS boot path through the `bootloader` crate.
- Optional custom BIOS bootloader path.
- Optional UEFI loader path.
- Framebuffer console, top bar, and serial logging paths.
- Shell command surface with categorized help and guided onboarding.
- Era profiles and `era` / `travel <year>` commands.
- Museum, quest, stats, inventory, demo, tour, capsule, doctor, poster, and app
  launcher product surfaces.
- IRQ1 PS/2 keyboard buffering with polling fallback.
- PS/2 mouse path and small fixed-capacity windows.
- PIT timer, GDT/IDT/PIC setup, exception paths, and PC speaker tones.
- Memory map handling, page helpers, and a 1 MiB free-list heap.
- ATA PIO storage and ChronoFS.
- ChronoFS `fsck`, conservative `fsck repair`, and one-record journal support.
- Built-in apps: notes, calculator, sysinfo, and text launcher cards.
- Cooperative task scheduler and early SMP/AP startup work.
- RTL8139 static IPv4 ARP/UDP networking paths.
- Ring 3 demo, tiny syscall layer, and static ELF execution path.

## Verified Evidence

- `cargo check -p kernel --offline --locked` passed with no warnings on
  2026-06-01.
- `cargo check -p chronosapien --target aarch64-apple-darwin --offline
  --locked` passed with no warnings on 2026-06-01.
- QEMU 11.0.1 and PowerShell 7.6.2 were installed and verified locally on
  2026-06-01.
- Single-core BIOS serial-only QEMU reached `[CHRONO] boot complete` on
  2026-06-01.
- Boot-time serial logging was observed through that point.

Important limit: the recorded QEMU run used `-display none`, so it does not
verify visible framebuffer output, shell prompt, keyboard input, apps,
filesystem workflows, windows, graphics, or screenshots.

## Still Needs Runtime Verification

- Visible BIOS framebuffer output and first shell prompt.
- Keyboard input at the shell, including IRQ path and polling fallback.
- Product commands: `start`, `guide`, `demo`, `tour`, `capsule`, `doctor`,
  `poster`, `travel <year>`, and `apps`.
- ChronoFS shell workflows: `ls`, `write`, `cat`, `rm`, `fsck`, `fsck repair`,
  and `journal`.
- App workflows: `notes`, `calc`, `sysinfo`, and app launcher routes.
- Mouse/window behavior: pointer movement, focus, drag, close, `open notes`,
  `open sysinfo`, `tasks`, and `kill <id>`.
- Heap reuse behavior across shell, app, filesystem, task, and window paths.
- UEFI boot and custom BIOS boot paths.
- Ring 3, syscalls, static ELF execution, ARP/UDP networking, SMP/AP startup,
  and hardware boot.

## Major Commands And Features

Getting started:

- `start`
- `welcome`
- `guide`
- `guide quick`
- `guide full`
- `help start`

Eras and product layer:

- `era`
- `era 1984`
- `era 1995`
- `era 2007`
- `era 2040`
- `travel <year>`
- `demo`
- `tour`
- `capsule`
- `doctor`
- `poster`

Apps and files:

- `apps`
- `notes`
- `calc 6 * 7`
- `sysinfo`
- `ls`
- `write demo.txt <content>`
- `cat demo.txt`
- `rm demo.txt`
- `fsck`
- `journal`

Advanced teaching paths:

- `ring3`
- `syshello`
- `exec <name>`
- `net`
- `net arp`
- `net send`

These advanced paths are partially implemented and should be run only during
intentional verification.

## 2-Minute Demo Path

Use this only after a visible BIOS QEMU shell run is recorded if publishing
screenshots or video.

1. `start`
2. `guide quick`
3. `help start`
4. `about`
5. `era`
6. `travel 1998`
7. `demo`
8. `poster`
9. `help system`
10. `doctor`

Short narration:

- ChronoOS is a Rust `no_std` x86_64 teaching OS.
- The shell is also the product surface.
- The repo separates implemented-in-code from runtime-verified evidence.

## v0.1 Release Checklist

| Area | v0.1 RC Status | Notes |
| --- | --- | --- |
| Build status | verified previously | Kernel and host package checks passed on 2026-06-01; rerun before tagging. |
| QEMU boot status | limited verified in QEMU | Single-core BIOS serial-only boot reached `[CHRONO] boot complete`. |
| Serial output status | limited verified in QEMU | Boot-time serial logging was observed; shell-command serial output still needs checks. |
| Keyboard status | needs runtime verification | IRQ keyboard and polling fallback exist in code. |
| Mouse/window status | needs runtime verification | PS/2 mouse and small windows exist in code. |
| Filesystem status | needs runtime verification | ChronoFS, fsck, repair, and journal exist in code. |
| Userspace status | needs runtime verification | Ring 3, syscall, and static ELF paths are teaching paths. |
| Networking status | needs runtime verification | Static IPv4 ARP/UDP paths exist; no TCP/DHCP/DNS. |
| Docs status | release-candidate ready | This file, limitations, roadmap, demo, screenshots, and checklist docs are aligned. |
| Screenshots/GIFs status | planned | Capture only after visible QEMU or hardware evidence exists. |

## Known Risks

- The current runtime evidence does not prove visible UI or shell interaction.
- SMP/AP startup is high-risk; a two-core serial-only smoke exited before
  `[CHRONO] boot complete`.
- ChronoFS repair and journal recovery need controlled disk-state tests.
- Userspace, syscalls, ELF execution, and networking are teaching paths, not
  mature subsystems.
- Mouse/window behavior is a small teaching window layer, not a compositor.

## Intentionally Not Included

- TCP, DHCP, DNS, sockets, or a full network stack.
- USB HID, USB storage, USB serial, or broad real-hardware support.
- Dynamic linker, package manager, libc, argv/env, or general userland.
- Full desktop compositor, browser, or large GUI toolkit.
- Production-grade preemptive scheduler.
- Claims of production readiness or Linux replacement behavior.

## Next Milestone

First post-v0.1 engineering goal: run a visible BIOS QEMU verification pass and
record evidence for framebuffer output, first shell prompt, keyboard input,
`help`, onboarding commands, ChronoFS basics, apps, product commands, and
screenshots.
