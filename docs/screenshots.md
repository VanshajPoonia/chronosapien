# ChronoOS Screenshot And GIF Checklist

Use this checklist to capture portfolio screenshots only after behavior has been
observed. Every screenshot or GIF should be tied to an entry in
`docs/AI_PROGRESS_LOG.md` with the date, image, QEMU or hardware environment,
and verification status.

## Status Tags

Use one of these tags in notes and filenames:

- `verified-qemu`: observed in QEMU.
- `verified-hardware`: observed on real hardware.
- `planned`: capture target, not observed yet.
- `serial-only`: serial evidence only; do not use for framebuffer claims.

## Naming Convention

Screenshots:

```text
chronoos-YYYY-MM-DD-area-topic-status.png
```

GIFs:

```text
chronoos-YYYY-MM-DD-area-flow.gif
```

Examples:

```text
chronoos-2026-06-01-boot-serial-verified-qemu.png
chronoos-2026-06-01-era-1995-planned.png
chronoos-2026-06-01-filesystem-write-cat-flow.gif
```

## v0.1 Capture Set

Capture these after visible QEMU or hardware evidence exists:

- [ ] Boot: BIOS boot screen and first shell prompt.
- [ ] Onboarding: `start`, `guide quick`, `help start`.
- [ ] Command map: `help`, `help apps`, `help fs`, `help system`.
- [ ] Era: `era`, `travel 1998`, `poster eras`.
- [ ] Status: `doctor`, `capsule current`, `poster system`.
- [ ] Apps: `apps`, `notes write <text>`, `notes read`, `calc 6 * 7`, `sysinfo`.
- [ ] Filesystem: `ls`, `write demo.txt <content>`, `cat demo.txt`, `fsck`, `journal`.
- [ ] Showcase: `demo`, `tour`, `poster`, `poster roadmap`.

Do not label any v0.1 screenshot `verified-qemu` or `verified-hardware` unless
the behavior was actually observed and recorded in `docs/AI_PROGRESS_LOG.md`.

## 2026-06-02 Captured Evidence

These files were captured during the UI/input QEMU verification pass and are
recorded in `docs/AI_PROGRESS_LOG.md`.

| File | Status | Evidence |
| --- | --- | --- |
| `/private/tmp/chronoos-ui-input-20260602-150049-boot.png` | verified-qemu | Visible framebuffer shell, top bar, prompt, and cursor. |
| `/private/tmp/chronoos-ui-input-20260602-150049-apps.png` | verified-qemu | `apps` launcher screen. |
| `/private/tmp/chronoos-ui-input-20260602-150049-notes-attempt.png` | verified-qemu | Notes home screen. |
| `/private/tmp/chronoos-ui-input-20260602-150049-calc.png` | verified-qemu | `calc 6 - 7` result. |
| `/private/tmp/chronoos-ui-input-20260602-150049-open-notes-window.png` | verified-qemu | Partial `open notes` window/task path. |
| `/private/tmp/chronoos-ui-input-20260602-150049-mouse-move.png` | planned | Captured after a mouse move attempt, but cursor movement was not clearly verified. |
| `/private/tmp/chronoos-ui-input-20260602-150049-drag-attempt.png` | planned | Captured after a drag attempt, but drag/close behavior was not clearly verified. |

The matching serial log is
`/private/tmp/chronoos-ui-input-20260602-150049.serial.log`. It includes
`[CHRONO] boot complete`, `cmd: apps`, `cmd: notes`, `cmd: calc 6 - 7`,
`app: calc launched`, `cmd: open notes`, `wm: open notes`, and
`mouse: click at 740,410`.

## Manual GIF Capture Steps

Animated GIF capture still needs manual verification. During the 2026-06-02
UI/input pass, QEMU `screendump` PNG capture worked, but `ffmpeg`, ImageMagick
`magick`, and `gifsicle` were not available on PATH.

For a future GIF pass:

1. Run the visible single-core BIOS QEMU path with a monitor socket and serial
   log, as described in `docs/release-checklist.md`.
2. Capture the visible QEMU window with a trusted local screen recorder or
   install a GIF/video encoder before the pass.
3. Record short flows only: boot to prompt, `start`, `help`, `apps`, `notes`,
   `calc`, `open notes`, and an intentional mouse/window interaction test.
4. Save files with the naming pattern
   `chronoos-YYYY-MM-DD-area-flow-verified-qemu.gif` only after the behavior is
   actually observed.
5. Add the exact command path, tool used, output file, and limitations to
   `docs/AI_PROGRESS_LOG.md` before using the GIF in portfolio material.

## Boot Screen Screenshot Checklist

- [ ] BIOS boot banner visible.
- [ ] First framebuffer shell prompt visible.
- [ ] Serial log captured through `[CHRONO] boot complete`.
- [ ] Screenshot notes distinguish visible framebuffer evidence from
      serial-only evidence.
- [ ] Multi-core boot screenshots are labeled separately from single-core boot.

## Era And Theme Screenshot Checklist

- [ ] `era 1984`
- [ ] `era 1995`
- [ ] `era 2007`
- [ ] `era 2040`
- [ ] `travel 1987`
- [ ] `travel 1998`
- [ ] `travel 2004`
- [ ] `travel 2049`
- [ ] `poster eras`

## Shell Command Screenshot Checklist

- [ ] `about`
- [ ] `help`
- [ ] `uptime`
- [ ] `clock`
- [ ] `mem`
- [ ] `cores`
- [ ] `doctor`
- [ ] Invalid command showing friendly failure behavior.

## Apps Launcher Screenshot Checklist

- [ ] `apps`
- [ ] `apps notes`
- [ ] `apps calc`
- [ ] `apps sysinfo`
- [ ] `apps files`
- [ ] `apps theme`
- [ ] `notes write <text>`
- [ ] `notes read`
- [ ] `calc 6 * 7`
- [ ] `sysinfo`

## Museum And Quest Screenshot Checklist

- [ ] `museum boot`
- [ ] `museum kernel`
- [ ] `museum memory`
- [ ] `museum filesystem`
- [ ] `museum userspace`
- [ ] `museum networking`
- [ ] `museum scheduler`
- [ ] `quest list`
- [ ] `quest status`
- [ ] `stats`
- [ ] `inventory`

## Filesystem Screenshot Checklist

- [ ] `ls`
- [ ] `write demo.txt <content>`
- [ ] `cat demo.txt`
- [ ] `rm demo.txt`
- [ ] `fsck`
- [ ] `journal`
- [ ] Persistence across reboot, if tested.
- [ ] `fsck repair`, only with a controlled disk image.

## Userspace Screenshot Checklist

Userspace paths are partially implemented and risky. Capture only during an
intentional verification pass.

- [ ] `tour userspace`
- [ ] `museum userspace`
- [ ] `museum syscalls`
- [ ] `museum elf`
- [ ] `ring3`
- [ ] `syshello`
- [ ] `exec hello.elf`
- [ ] Return-to-shell or failure behavior recorded honestly.

## Networking Screenshot Checklist

Networking is static IPv4 ARP/UDP only. Do not imply DHCP, DNS, TCP, sockets, or
broad hardware support.

- [ ] `net`
- [ ] `net arp`
- [ ] Gateway MAC learned, if observed.
- [ ] `net send`
- [ ] `net send <ip> <port> <text>`
- [ ] Serial log for incoming/outgoing UDP, if observed.

## Crash Lab Warning Screenshot Checklist

Crash lab is roadmap/design-only unless a future source audit finds an
implementation.

- [ ] Do not capture crash lab screenshots as implemented unless the command or
      app exists in source.
- [ ] If a controlled panic/fault is tested, label it as a low-level verification
      test, not a product feature.
- [ ] Do not run crash/fault paths during casual portfolio demos.

## Poster And Showcase Screenshot Checklist

- [ ] `demo`
- [ ] `tour`
- [ ] `capsule`
- [ ] `capsule milestones`
- [ ] `capsule current`
- [ ] `capsule next`
- [ ] `poster`
- [ ] `poster boot`
- [ ] `poster system`
- [ ] `poster roadmap`
- [ ] `poster eras`

## Capture Notes Template

```text
File:
Date:
Status: planned | serial-only | verified-qemu | verified-hardware
Image:
Command:
Host:
QEMU/hardware details:
Observed behavior:
Limitations:
Progress-log entry:
```
