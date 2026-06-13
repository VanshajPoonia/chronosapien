# ChronoOS Demo Script

Use this script for portfolio walkthroughs, build-in-public clips, and live
demos. It lists good paths through ChronoOS without implying that every command
has already been runtime-verified.

For the v0.1 release candidate, treat these as planned demo paths until visible
QEMU or hardware evidence records the framebuffer shell, keyboard input, and the
commands being shown. See `docs/RELEASE_v0.1.md` for the release story and
`docs/KNOWN_LIMITATIONS.md` for boundaries.

Current verification boundary: single-core BIOS QEMU has evidence for boot,
serial output, framebuffer shell, `start`, `guide quick`, `mode status`,
`safe on`, `doctor`, `apps list`, disposable-image ChronoFS basics, and narrow
userspace/window paths. New shell workspace polish commands should be treated as
implemented in code until a later QEMU pass records them.

## Demo Rules

- Prefer the normal BIOS image for portfolio demos until UEFI/custom BIOS are
  separately verified.
- Record the exact date, image, QEMU command, screenshots, and serial logs.
- Keep explanations beginner-friendly: this is an educational OS, not a Linux
  replacement.
- If a command fails during a live demo, call it a verification result and log
  it rather than smoothing over it.

## 2-Minute Demo Path

Goal: show identity, product feel, and a conservative status surface.

1. Start from a visible BIOS QEMU boot and first shell prompt.
2. Set the demo safety frame:
   - `workspace`
   - `shortcuts`
   - `whereami`
   - `mode status`
   - `mode safe`
3. Show identity:
   - `start`
   - `guide quick`
   - `learn`
   - `help start`
   - `about`
   - `help`
4. Show era/product flavor:
   - `era`
   - `travel 1998`
5. Show the guided product layer:
   - `learn boot`
   - `demo`
   - `poster`
6. Close with status honesty:
   - `status`
   - `verify`
   - `help system`
   - `doctor`
   - `safe off`

Suggested narration:

- "ChronoOS is a Rust `no_std` x86_64 teaching OS."
- "The shell is also the product surface: eras, museum pages, quests, apps, and
  filesystem tools."
- "The docs separate implemented-in-code from verified-in-QEMU so the project
  stays honest."

## 5-Minute Demo Path

Goal: show ChronoOS as both a systems project and an indie educational product.

1. Identity and shell:
   - `workspace`
   - `shortcuts`
   - `welcome`
   - `guide full`
   - `learn next`
   - `about`
   - `help`
   - `help search app`
   - `help apps`
   - `uptime`
   - `mem`
2. Era switching:
   - `era 1984`
   - `travel 2004`
   - `poster eras`
3. Museum and quest layer:
   - `learn boot`
   - `learn filesystem`
   - `museum boot`
   - `museum filesystem`
   - `quest list`
   - `stats`
   - `inventory`
4. Apps:
   - `apps`
   - `apps featured`
   - `apps category Core`
   - `apps notes`
   - `apps demo notes`
   - `notes write hello from ChronoOS`
   - `notes read`
   - `calc 6 * 7`
   - `sysinfo`
5. Filesystem:
   - `files`
   - `files sample`
   - `files list`
   - `help fs`
   - `ls`
   - `write demo.txt ChronoOS has a tiny filesystem`
   - `cat demo.txt`
   - `files info demo.txt`
   - `files search tiny`
   - `fsck`
   - `journal`
6. Conservative close:
   - `whereami`
   - `verify`
   - `capsule current`
   - `doctor`

## 10-Minute Demo Path

Goal: show the full current surface without expanding into risky systems.

1. Boot and identity:
   - `workspace`
   - `shortcuts`
   - `whereami`
   - `start`
   - `guide`
   - `guide full`
   - `about`
   - `help`
   - `help search fs`
   - `help start`
   - `clock`
   - `uptime`
   - `mem`
2. Era experience:
   - `theme`
   - `era 1984`
   - `era 1995`
   - `era 2007`
   - `era 2040`
   - `travel 1987`
   - `travel 2049`
3. Guided product surfaces:
   - `learn gui`
   - `demo`
   - `tour`
   - `tour boot`
   - `tour files`
   - `capsule`
   - `poster system`
4. Museum and quest:
   - `museum kernel`
   - `museum interrupts`
   - `museum filesystem`
   - `museum scheduler`
   - `quest status`
   - `stats`
   - `inventory`
5. Apps and launcher:
   - `apps`
   - `apps featured`
   - `apps category Learning`
   - `apps calc`
   - `apps info calc`
   - `apps demo calc`
   - `calc 21 + 21`
   - `apps sysinfo`
   - `sysinfo`
   - `apps notes`
   - `notes write demo note`
   - `notes read`
6. ChronoFS workflow:
   - `ls`
   - `write demo.txt ChronoOS demo file`
   - `cat demo.txt`
   - `rm demo.txt`
   - `fsck`
   - `journal`
7. Optional windows if visual input is being verified:
   - `windows status`
   - `open notes`
   - `windows list`
   - `open sysinfo`
   - `windows focus <id>`
   - `windows close <id>`
   - `tasks`
   - `kill <id>`
8. Optional userspace if intentionally verifying risky paths:
   - `help userspace`
   - `ring3`
   - `syshello`
   - `exec hello.elf`
9. Optional networking if intentionally verifying ARP/UDP:
   - `help network`
   - `net status`
   - `net config`
   - `net log`
   - `net demo`
   - `net arp`
   - `net udp`
   - `net send`
10. Wrap:
   - `poster roadmap`
   - `doctor`

## Commands To Show ChronoOS Identity

- `about`
- `help`
- `start`
- `welcome`
- `guide quick`
- `demo`
- `capsule current`
- `poster system`
- `doctor`

## Commands To Show Era Switching

- `era`
- `era 1984`
- `era 1995`
- `era 2007`
- `era 2040`
- `travel 1987`
- `travel 1998`
- `travel 2004`
- `travel 2049`
- `poster eras`

## Commands To Show Museum And Quest Layer

- `learn map`
- `learn progress`
- `learn beginner`
- `learn advanced`
- `learn next`
- `explain kernel`
- `explain filesystem`
- `explain syscall`
- `museum boot`
- `museum index`
- `museum kernel`
- `museum memory`
- `museum interrupts`
- `museum filesystem`
- `museum userspace`
- `museum syscalls`
- `museum elf`
- `museum networking`
- `museum smp`
- `museum scheduler`
- `quest list`
- `quest status`
- `quest dependencies`
- `quest badges`
- `stats`
- `inventory`

## Commands To Show Apps

- `apps`
- `apps list`
- `apps featured`
- `apps recent`
- `apps category Core`
- `apps category Files`
- `apps category Learning`
- `apps info notes`
- `apps help notes`
- `apps demo notes`
- `apps launch calc`
- `apps verified`
- `apps roadmap`
- `apps notes`
- `notes write <text>`
- `notes read`
- `notes clear`
- `notes open`
- `apps calc`
- `calc 6 * 7`
- `apps sysinfo`
- `sysinfo`
- `apps files`
- `apps clock`
- `apps museum`
- `apps theme`
- `apps tasks`

## Commands To Show Filesystem

- `files`
- `files sample`
- `files list`
- `ls`
- `write <name> <content>`
- `cat <name>`
- `files info <name>`
- `files search <term>`
- `files copy <src> <dst>` on a disposable image only
- `files rename <old> <new>` to show the current conservative refusal
- `rm <name>`

Use throwaway demo filenames such as `demo.txt` or `tour.txt`. Avoid deleting
files that were not created during the current demo.

## Commands To Show fsck And Journal

- `fsck`
- `journal`

Use `fsck repair` only during intentional filesystem verification with a
known-safe disk image. It writes metadata and should not be part of a casual
portfolio demo.

## Commands To Show Userspace, Syscalls, And ELF

These are implemented-in-code teaching paths but remain risky until runtime
evidence is recorded.

- `ring3`
- `syshello`
- `exec <name>`

Only run these during intentional verification, not as the default live demo
path.

## Commands To Show Networking

Networking is currently static IPv4 ARP/UDP only. There is no DHCP, DNS, TCP, or
socket stack.

- `net status`
- `net config`
- `net log`
- `net demo`
- `net arp`
- `net udp`
- `net send`
- `net send <ip> <port> <text>`
- `net roadmap`

Use `net status`, `net config`, `net log`, `net demo`, `net udp`, and
`net roadmap` for read-only explanation. Only run `net arp` and `net send`
during intentional ARP/UDP verification with a known QEMU network setup.

## Commands And Paths To Avoid Unless Testing Risk

Run `mode status` before the risky section, and use `mode experimental` only for
intentional verification. Safe mode is warning-only; it does not block commands.

- `ring3`, `syshello`, and `exec <name>` unless verifying userspace paths.
- `net arp`, `net send`, and custom UDP sends unless verifying ARP/UDP.
- `fsck repair` unless using a controlled disk image.
- `open notes`, `open sysinfo`, `windows focus <id>`, `windows close <id>`,
  mouse drag, and close behavior unless verifying the visual window/input layer.
- Multi-core QEMU / SMP testing unless investigating AP startup.
- Custom BIOS and UEFI boot paths unless testing those boot flows explicitly.
- Crash/fault demos unless a future crash lab is implemented and documented.
- Any claim of TCP, DHCP, DNS, USB, dynamic linking, package management, full
  compositor behavior, or preemptive scheduling.
