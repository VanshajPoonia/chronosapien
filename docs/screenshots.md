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

## 2026-06-13 Captured Evidence

These files were captured during the visible single-core BIOS product
verification pass and are recorded in `docs/AI_PROGRESS_LOG.md`.

| File | Status | Evidence |
| --- | --- | --- |
| `/private/tmp/chronoos-visible-bios-20260613-184819-boot.png` | verified-qemu | Visible framebuffer shell, top bar, prompt, and serial `[CHRONO] boot complete`. |
| `/private/tmp/chronoos-visible-bios-20260613-184819-start.png` | verified-qemu | `start` first-run welcome screen. |
| `/private/tmp/chronoos-visible-bios-20260613-184819-guide-quick.png` | verified-qemu | `guide quick` short guide screen. |
| `/private/tmp/chronoos-visible-bios-20260613-184819-safe-on.png` | verified-qemu | `mode status` output and `safe on` warning-only safe-mode state. |
| `/private/tmp/chronoos-visible-bios-20260613-184819-doctor.png` | verified-qemu | `doctor` conservative subsystem report. |
| `/private/tmp/chronoos-visible-bios-20260613-185808-apps-list.png` | verified-qemu | `apps list` static app registry output. |
| `/private/tmp/chronoos-visible-bios-20260613-184819-mode-status.png` | planned | Captured too early/inconclusive; use the later `safe-on` frame as `mode status` evidence. |
| `/private/tmp/chronoos-visible-bios-20260613-184819-current-after-poster-attempt.png` | planned | Diagnostic frame after garbled `poster system` input; not evidence that `poster system` worked. |
| `/private/tmp/chronoos-visible-bios-20260613-185808-stalled-boot.png` | verified-qemu | Diagnostic second-run boot frame; serial later reached `[CHRONO] boot complete`. |

The matching serial logs are
`/private/tmp/chronoos-visible-bios-20260613-184819.serial.log` and
`/private/tmp/chronoos-visible-bios-20260613-185808.serial.log`. They include
exact command lines for `start`, `guide quick`, `mode status`, `safe on`,
`doctor`, and `apps list`. They do not include exact `cmd: poster system` or
`cmd: capsule current`; those commands remain manual-verification targets.

PPM originals with the same basenames were captured before PNG conversion. GIF
capture was not attempted in this pass.

## 2026-06-13 ChronoFS Captured Evidence

These files were captured during the disposable ChronoFS QEMU verification pass
and are recorded in `docs/AI_PROGRESS_LOG.md`.

| File | Status | Evidence |
| --- | --- | --- |
| `/private/tmp/chronoos-chronofs-20260613-191106-boot.png` | verified-qemu | Fresh disposable image formatted/mounted and visible shell reached boot complete. |
| `/private/tmp/chronoos-chronofs-20260613-191106-fs-status.png` | verified-qemu | `fs status` output with persistent ATA disk, file/slot counts, and journal summary. |
| `/private/tmp/chronoos-chronofs-20260613-191106-fs-info-ls.png` | verified-qemu | `fs info` layout output and clean initial `ls` listing. |
| `/private/tmp/chronoos-chronofs-20260613-191106-current-before-rm-retry.png` | verified-qemu | `write`, `cat`, `fs check`, `fs journal`, `fsck`, and `journal` output before deletion. |
| `/private/tmp/chronoos-chronofs-20260613-191106-post-delete-ls.png` | verified-qemu | `rm verify.txt` and post-delete `ls` showing `verify.txt` absent. |
| `/private/tmp/chronoos-chronofs-20260613-191106-reboot-persistence.png` | verified-qemu | Reboot with the same disposable image; `cat verify.txt` reported file not found, `fs status` and `journal` remained clean. |

The matching serial logs are
`/private/tmp/chronoos-chronofs-20260613-191106.serial.log` and
`/private/tmp/chronoos-chronofs-20260613-191106-reboot.serial.log`. They include
exact command lines for `fs status`, `fs info`, `ls`, `write verify.txt chrono
verification test`, `cat verify.txt`, `fs check`, `fs journal`, `fsck`,
`journal`, and `rm verify.txt`. They also include stray `lls` and `ffs check`
input artifacts from QEMU monitor key injection; those artifacts are not counted
as filesystem behavior.

This pass verifies delete persistence after reboot. It does not verify
independent write persistence before deletion, `fsck repair`, crash recovery,
corrupt journal handling, heap fallback, hardware, or GIF capture.

## 2026-06-13 Window/Input Captured Evidence

These files were captured during the visible single-core BIOS window/input QEMU
verification pass and are recorded in `docs/AI_PROGRESS_LOG.md`.

| File | Status | Evidence |
| --- | --- | --- |
| `/private/tmp/chronoos-window-input-20260613-193131-boot.png` | verified-qemu | Fresh disposable-image boot reached `[CHRONO] boot complete`. |
| `/private/tmp/chronoos-window-input-20260613-193131-windows-status.png` | verified-qemu | `windows status` output. |
| `/private/tmp/chronoos-window-input-20260613-193131-open-notes.png` | verified-qemu | `open notes` created a visible notes window. |
| `/private/tmp/chronoos-window-input-20260613-193131-current-after-sysinfo-attempt.png` | verified-qemu | `open sysinfo` created visible sysinfo windows; duplicate window was an input artifact. |
| `/private/tmp/chronoos-window-input-20260613-193131-windows-list-exact.png` | verified-qemu | Exact `windows` alias listed notes/sysinfo window IDs and task IDs. |
| `/private/tmp/chronoos-window-input-20260613-193131-mouse-click-notes-attempt.png` | verified-qemu | `windows focus 1` brought notes to front; serial later recorded mouse click at `70,65`. |
| `/private/tmp/chronoos-window-input-20260613-193131-mouse-final-attempt.png` | planned | Diagnostic frame after HMP mouse commands; visible movement was not clear enough for a cursor-movement claim. |
| `/private/tmp/chronoos-window-input-20260613-193131-windows-close-2.png` | planned | Diagnostic frame after serial-backed `windows close 2`; not visual proof of final window/task state. |

The matching serial log is
`/private/tmp/chronoos-window-input-20260613-193131.serial.log`. It includes
exact command lines for `windows status`, `open notes`, `open sysinfo`,
`windows`, and `windows focus 1`, plus `mouse: click at 70,65`. It does not
include exact `cmd: windows close 2`, `sched: killed task 2`, and
`wm: close sysinfo`, but no follow-up `windows`/`tasks` output and no
`kill <id>` command were run after that.

This pass does not verify manual typing, Backspace, Shift, visible cursor
movement, mouse drag, mouse close, visual close confirmation, `tasks`, `kill`,
hardware, or GIF capture.

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
