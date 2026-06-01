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
