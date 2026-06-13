# ChronoOS Progress Audit

Date: 2026-06-13

This audit summarizes the current ChronoOS working tree after the v0.1 release
candidate docs, verification matrix, ChronoFS hardening, user-space foundation,
app/window polish, networking observability, safe mode, learning paths, portfolio
kit, and v0.2 roadmap passes.

This is an evidence audit, not runtime proof. Do not upgrade a feature to
`verified in QEMU` or `verified on hardware` unless `docs/AI_PROGRESS_LOG.md`,
`docs/VERIFICATION_MATRIX.md`, or a later audit records the actual command,
log, screenshot, or hardware evidence.

## 1. Executive Summary

ChronoOS is a Rust `no_std` x86_64 educational hobby OS with a strong
terminal-first product shell: eras, museum pages, quests, learning paths, small
apps, ChronoFS, a tiny window layer, userspace teaching paths, and conservative
verification/status surfaces.

The real verified core is narrower than the codebase:

- Verified in QEMU: normal single-core BIOS boot, boot-time serial logging,
  visible framebuffer prompt/top bar, narrow shell input and command output,
  `start`, `guide quick`, `mode status`, `safe on`, `doctor`, `apps`,
  `apps list`, notes home, `calc 6 - 7`, `open notes`, `open sysinfo`,
  `windows status`, the `windows` list alias, `windows focus 1`, one mouse
  click packet, RTL8139 device init/MAC discovery, PNG screendump capture, and
  a disposable-image ChronoFS flow covering `fs status`, `fs info`, `ls`,
  `write`, `cat`, `rm`, `fs check`, `fs journal`, `fsck`, `journal`, and
  delete persistence after reboot; a userspace-boundary pass covering
  `userspace status`, `userspace syscalls`, and the fixed `ring3`
  transition/privilege-fault teaching demo.
- Partially verified in QEMU: keyboard input, app/window behavior, mouse/window
  path, UEFI loader start under OVMF, SMP/BSP-only smoke, and networking device
  initialization.
- Implemented in code but still unverified: `fsck repair`, journal recovery,
  independent write persistence before deletion, heap fallback reporting, heap
  reuse, successful `windows close <id>`, `tasks`, `kill`, broader scheduler
  lifecycle, `poster system`, `capsule current`, most product/status commands,
  learning paths, app registry subcommands beyond `apps list`, `userspace elf`,
  `syshello`, static ELF execution, and ARP/UDP behavior.
- Blocked or broken: UEFI kernel handoff after loader start, custom BIOS build
  dependency, AP startup evidence, reliable QEMU shell input/manual GUI typing,
  the visually inconclusive `windows close 2` follow-up, GIF tooling, and all
  real hardware claims.
- Roadmap/design-only: TCP, DHCP, DNS, USB, dynamic linker, package manager,
  full compositor, preemptive scheduler, theme studio, crash lab, tiny paint,
  windowed file explorer, boot chime selector, and packet-demo app.

## 2. Feature Status Table

| Area | Feature | Status | Verification | Notes |
| --- | --- | --- | --- | --- |
| Identity | ChronoOS naming/public identity | implemented in code | not runtime-facing | Public docs use ChronoOS; repo/package/image names may remain `chronosapien`. |
| Docs | README/docs alignment | partially implemented | needs runtime verification | README links current docs but still contains compact and older status sections that can lag the matrix. |
| Docs | manual testing docs | implemented in code | documented only | `docs/manual-testing.md` gives staged checks; not itself proof. |
| Docs | verification matrix | implemented in code | documented only | `docs/VERIFICATION_MATRIX.md` is the compact evidence source. |
| Docs | release docs | implemented in code | documented only | v0.1 RC docs are honest but must be read beside current evidence. |
| Docs | portfolio docs | implemented in code | documented only | Portfolio kit/showcase/resume posts exist; runtime claims still need evidence checks. |
| Product shell | guided onboarding | implemented in code | partially verified in QEMU | `start` and `guide quick` were observed on 2026-06-13; `welcome` and broader guide topics remain unverified. |
| Product shell | command/UX polish | implemented in code | partially verified in QEMU | `help`, `help start`, and `about` were observed; most topic pages remain unverified. |
| Product shell | safe mode | implemented in code | partially verified in QEMU | `mode status` and `safe on` were observed on 2026-06-13; other mode/safe flows remain unverified. |
| Product shell | learning paths | implemented in code | needs runtime verification | `learn` namespace exists and is read-only; QEMU walkthrough still needed. |
| Apps | app registry | implemented in code | partially verified in QEMU | `apps list` was observed on 2026-06-13; `apps info`, `apps launch`, `apps verified`, and `apps roadmap` remain unverified. |
| Apps/UI | window/app lifecycle | partially implemented | partially verified in QEMU | `open notes`, `open sysinfo`, `windows status`, `windows` list alias, `windows focus 1`, and serial-backed `windows close 2` observed on 2026-06-13; visual close confirmation, `tasks`, `kill`, drag, and mouse close remain unverified. |
| Storage | ChronoFS hardening | implemented in code | partially verified in QEMU | 2026-06-13 disposable-image pass observed `fs status`, `fs info`, `ls`, `write`, `cat`, `rm`, and delete persistence after reboot. |
| Storage | fsck | implemented in code | partially verified in QEMU | 2026-06-13 pass observed clean read-only `fs check` and `fsck`; `fsck repair` remains unverified. |
| Storage | journal | implemented in code | partially verified in QEMU | 2026-06-13 pass observed clean `fs journal`/`journal`; rollback, roll-forward, corrupt-record refusal, and crash recovery remain unverified. |
| Userspace | user-space model | partially implemented | partially verified in QEMU | `userspace status` and the fixed `ring3` transition/fault demo were observed on 2026-06-13; this is not a general process model. |
| Userspace | syscall table | partially implemented | partially verified in QEMU | `userspace syscalls` listed write/read/exit/uptime on 2026-06-13; `syshello` runtime syscall behavior remains unverified. |
| Userspace | static ELF execution | partially implemented | blocked by tooling | `exec <name>` exists, but `ld.lld` was missing and no safe `hello.elf` was installed during the 2026-06-13 pass. |
| Networking | networking observability | partially implemented | partially verified in QEMU | RTL8139 init/MAC observed; counters/commands and ARP/UDP behavior unverified. |
| Product apps | theme studio | roadmap/design-only | roadmap/design-only | `apps theme` is a text preview only. |
| Product apps | crash lab | roadmap/design-only | roadmap/design-only | No crash lab command/app exists. |
| Product apps | tiny paint | roadmap/design-only | roadmap/design-only | `open paint` reports future status; no canvas exists. |
| Product apps | file explorer | roadmap/design-only | roadmap/design-only | `apps files` is a shell-command card, not a windowed explorer. |
| Product apps | boot chime | roadmap/design-only | roadmap/design-only | Era tones exist, but no user-facing selector. |
| Networking | network demo | roadmap/design-only | roadmap/design-only | `net demo` is a read-only guide, not a verified packet-demo app. |
| Product shell | visual boot timeline | partially implemented | needs runtime verification | `capsule` and `poster boot` are text surfaces; 2026-06-13 `capsule current` input garbled as `ccapsule current`. |
| Product shell | era-specific help/about | partially implemented | needs runtime verification | Era profiles, `era`, `travel`, and product text exist; deeper era-specific help is future polish. |
| Apps/UI | mini desktop launcher | partially implemented | partially verified in QEMU | Text launcher and tiny window layer exist; `open notes`/`open sysinfo`/list/focus have narrow QEMU evidence; not a desktop/compositor. |
| Media | screenshots/GIF checklist | partially implemented | partially verified in QEMU | QEMU PNG screendumps worked again on 2026-06-13 for boot, onboarding, safe/status, doctor, app registry, ChronoFS, and window/input screens; GIF capture remains unverified/tool-blocked. |
| Planning | v0.2 roadmap | implemented in code | documented only | Recommends Reliability and Verification primary, ChronoFS Hardening secondary. |
| Boot | BIOS boot | implemented in code | verified in QEMU | Single-core BIOS runs reached `[CHRONO] boot complete`; multi-core/AP path separate. |
| Boot | UEFI boot | implemented in code | partially verified in QEMU UEFI | 2026-06-13 fixed the UEFI build/image path and OVMF started the ChronoOS loader, but the loader failed with `Out of Resources` before kernel handoff. |
| Boot | custom BIOS | partially implemented | blocked | `nasm` missing during preflight; custom scripts not run. |
| Hardware | hardware | documented only | needs runtime verification | No hardware image write, boot, or serial log is recorded. |
| SMP | SMP/AP | partially implemented | partially verified in QEMU | Two-core smoke reached BSP startup only; no AP-online or two-core-ready evidence. |
| Roadmap | DHCP/DNS/TCP roadmap | roadmap/design-only | roadmap/design-only | Current network boundary is static IPv4 ARP/UDP only. |
| Roadmap | USB roadmap | roadmap/design-only | roadmap/design-only | No USB HID/storage/serial stack. |
| Roadmap | preemptive scheduler roadmap | roadmap/design-only | roadmap/design-only | Current scheduler is cooperative. |
| Roadmap | dynamic app loading/package manager roadmap | roadmap/design-only | roadmap/design-only | Static registry only; no dynamic loading. |
| Roadmap | GUI/compositor roadmap | roadmap/design-only | roadmap/design-only | Current UI is framebuffer shell plus small fixed-capacity windows. |

## 3. Commands Actually Present

This inventory is based on `kernel/src/shell.rs`, `kernel/src/apps/mod.rs`,
`kernel/src/net.rs`, `kernel/src/museum.rs`, `kernel/src/quest.rs`, and
`docs/shell-commands.md`.

### Getting started

- `help`
- `help start`, `help guide`
- `start`
- `welcome`
- `guide`
- `guide quick`, `guide full`, `guide eras`, `guide apps`, `guide systems`,
  `guide status`, `guide next`
- `demo`
- `tour`
- `tour boot`, `tour memory`, `tour files`, `tour apps`, `tour userspace`,
  `tour future`

### Eras/themes

- `era`
- `era 1984`, `era 1995`, `era 2007`, `era 2040`
- `travel <year>`
- `poster eras`
- `apps theme`

### Apps

- `apps`, `apps list`
- `apps info <name>`
- `apps launch <name>`
- `apps verified`
- `apps roadmap`
- `apps notes`, `apps calc`, `apps sysinfo`, `apps files`, `apps clock`,
  `apps museum`, `apps theme`, `apps tasks`
- `notes`, `notes read`, `notes write <text>`, `notes clear`, `notes save`,
  `notes open`
- `calc <int> +|-|*|/ <int>`
- `sysinfo`

### Filesystem

- `fs`, `fs status`
- `fs info`
- `fs check`
- `fs journal`
- `fs help`
- `fs repair`, `fs check repair` (refuse and point to `fsck repair`)
- `ls`
- `cat <name>`
- `write <name> <content>`
- `rm <name>`
- `fsck`
- `fsck repair`
- `journal`

### Museum/quests/learning

- `learn`
- `learn boot`, `learn memory`, `learn interrupts`, `learn filesystem`,
  `learn gui`, `learn userspace`, `learn networking`, `learn scheduler`,
  `learn eras`, `learn roadmap`, `learn next`
- `museum boot`, `museum kernel`, `museum memory`, `museum interrupts`,
  `museum keyboard`, `museum serial`, `museum era`
- `museum disk`, `museum filesystem`, `museum userspace`, `museum syscalls`,
  `museum elf`, `museum networking`, `museum smp`, `museum scheduler`
- `quest list`
- `quest status`
- `stats`
- `inventory`

### Status/verification

- `doctor`
- `capsule`
- `capsule milestones`, `capsule current`, `capsule next`
- `poster`
- `poster boot`, `poster system`, `poster roadmap`, `poster eras`
- `mem`
- `cores`
- `uptime`
- `clock`
- `help system`, `help status`, `help verify`

There is no separate top-level `status` or `verify` command; the shell routes
users toward `doctor`, `help system`, `capsule current`, and poster/status
surfaces.

### Userspace

- `userspace`, `userspace status`
- `userspace syscalls`
- `userspace elf`
- `userspace roadmap`
- `userspace help`
- `ring3`
- `syshello`
- `exec <name>`

### Networking

- `net`, `net status`
- `net config`
- `net arp`
- `net udp`
- `net send`
- `net send <ip> <port> <text>`
- `net log`
- `net demo`
- `net roadmap`
- `net help`

### Window/UI

- `windows`, `windows list`
- `windows status`
- `windows focus <id>`
- `windows close <id>`
- `windows help`
- `open notes`
- `open sysinfo`
- `open paint` (reports roadmap/design-only)
- `tasks`
- `kill <id>`

### Debug/lab

- `beep <hz>`
- `reboot`
- `fsck repair`
- `ring3`
- `syshello`
- `exec <name>`
- `net arp`
- `net send`

### Safe mode

- `mode`, `mode status`
- `mode safe`
- `mode demo`
- `mode experimental`
- `mode help`
- `safe`, `safe status`
- `safe on`
- `safe off`
- `safe help`

### Roadmap/future

- `help roadmap`
- `capsule next`
- `poster roadmap`
- `tour future`
- `userspace roadmap`
- `net roadmap`
- `apps roadmap`

## 4. Docs That Are Current

These docs appear current enough to use as source-truth or active references:

- `README.md`: good entrypoint and link hub, but see stale/duplicative notes.
- `AGENTS.md`: current project rules for ChronoOS identity and verification.
- `docs/AI_PROGRESS_LOG.md`: append-only evidence/progress log.
- `docs/CURRENT_STATUS.md`: broad source-truth status handoff.
- `docs/VERIFICATION_MATRIX.md`: compact evidence-level matrix.
- `docs/manual-testing.md`: active staged verification checklist.
- `docs/release-checklist.md`: release gates and recorded evidence.
- `docs/RELEASE_v0.1.md`: v0.1 RC release note.
- `docs/ROADMAP_v0.2.md`: current v0.2 product roadmap.
- `docs/NEXT_STEPS.md`: active priority queue, updated by this audit.
- `docs/shell-commands.md`: current command inventory.
- `docs/screenshots.md`: screenshot/GIF capture checklist.
- `docs/demo-script.md`: demo paths with verification caveats.
- `docs/storage.md` and `docs/chronofs-hardening.md`: current ChronoFS docs.
- `docs/userspace-model.md`, `docs/ring3.md`, `docs/syscalls.md`,
  `docs/elf.md`: current user-space boundary docs.
- `docs/apps.md` and `docs/windowing.md`: current app/window boundaries.
- `docs/networking.md`: current static IPv4/ARP/UDP boundary docs.
- `docs/learning-paths.md` and `docs/safe-mode.md`: current product shell docs.
- `docs/portfolio-kit.md`, `docs/build-in-public-posts.md`,
  `docs/resume-bullets.md`: useful public-facing material when checked against
  the verification matrix.

## 5. Docs That Are Stale or Duplicative

- `docs/status-audit.md`: Phase 2 historical snapshot; useful for history, not
  the newest source of truth.
- `docs/roadmap.md`: older broad roadmap; mostly superseded by
  `docs/ROADMAP_v0.2.md` and this audit.
- `docs/ROADMAP_AFTER_v0.1.md`: still useful context, but overlaps with v0.2
  planning.
- `docs/dev-log.md`: historical scaffold notes with Chronosapian naming.
- Older entries in `docs/AI_PROGRESS_LOG.md`: intentionally historical and may
  mention Time Capsule OS or Chronosapian.
- README compact/current-state sections: useful as an overview, but the exact
  status should be checked against `docs/VERIFICATION_MATRIX.md` and this audit.
- `docs/showcase.md`: useful portfolio narrative, but some screenshot/GIF
  targets are still planned or need evidence refresh before publication.

## 6. Features That Are Code-Present But Unverified

- ChronoFS remaining gaps: independent write persistence before deletion, heap
  fallback reporting, non-disposable image behavior, and disk-error handling.
- `fsck repair`, controlled bitmap repair, stale-slot repair, and repair refusal
  scenarios.
- Journal rollback, roll-forward, corrupt-record refusal, and recovery after
  interrupted write/remove operations.
- Heap allocator reuse and corruption resistance under app/window/filesystem
  churn.
- Scheduler lifecycle: `tasks`, `kill`, task cleanup, task/window owner cleanup,
  repeated spawn/close behavior.
- Safe mode and reliability mode flows beyond the observed `mode status` and
  `safe on` commands.
- Learning paths, guided onboarding, demo/tour/capsule/doctor/poster surfaces
  beyond the observed `start`, `guide quick`, and `doctor` paths.
- App registry subcommands beyond the observed `apps list`: `apps info`,
  `apps launch`, `apps verified`, `apps roadmap`.
- `sysinfo`, notes read/write/clear/persistence, and broader calculator cases.
- Window lifecycle gaps after the 2026-06-13 pass: exact `windows list`
  command input, visual close confirmation, `tasks`, `kill`, drag, mouse close,
  visible cursor movement, and repeated spawn/close cleanup.
- Remaining userspace gaps: exact `userspace elf` input, `syshello`, static ELF
  execution, invalid/missing ELF behavior, and broader syscall behavior.
- ARP request/reply, UDP send/receive, `net log` counter behavior, malformed RX
  counters, and host-to-guest networking.
- Keyboard Backspace, shifted input, manual typing reliability, and polling
  fallback.
- Timer/status command behavior from the interactive shell beyond incidental
  boot observations.

## 7. Features That Are Documented But Not Implemented

- Theme studio/editor workflow.
- Crash lab product surface.
- Tiny paint canvas/app.
- Windowed file explorer.
- Boot chime selector.
- Verified packet-demo/network demo app.
- DHCP, DNS, TCP, sockets, and packet capture.
- USB HID, USB storage, USB serial, and broad real-hardware USB support.
- Dynamic linker.
- Dynamic app loading and package manager.
- Full compositor, GUI toolkit, production desktop environment.
- Production-grade preemptive scheduler.
- General Unix-like userland with fork/exec semantics, argv/env, libc, or broad
  file descriptors.

## 8. Features That Are Broken or Risky

- UEFI image build now succeeds, but the 2026-06-13 single-core OVMF attempt
  fails after loader start with `Out of Resources` before framebuffer handoff.
- Custom BIOS build/run is blocked by missing `nasm`.
- SMP/AP startup is high-risk; recorded two-core smoke reached BSP startup only.
- Networking ARP/UDP verification is blocked by unreliable QEMU monitor command
  input and a host UDP forwarding/listener conflict in the recorded pass.
- GIF capture is blocked by missing GIF/video tooling in the recorded pass.
- Hardware is unverified; there is no hardware boot, image write, or serial log.
- QEMU monitor key injection can garble longer commands, so future verification
  needs slower one-command-at-a-time input or a better interactive path. The
  2026-06-13 pass still garbled `poster system` and `capsule current` as
  `poster s`/`possteerr  ssyyssttem`/`pposter system` and `ccapsule current`.
- A 2026-06-13 window/input pass partially verified `windows close 2` through
  serial (`cmd: windows close 2`, `sched: killed task 2`, `wm: close sysinfo`),
  but the framebuffer snapshot was a breakpoint-like black screen and no
  follow-up `windows`/`tasks` output was captured. Treat visual close
  confirmation, `tasks`, and `kill` as unverified until a fresh pass proves
  them.
- ChronoFS repair and journal recovery mutate metadata and still need
  disposable disk images plus before/after evidence.
- Heap allocator lacks recorded stress evidence; older audit notes identify
  corruption/double-free guards as limited.
- Ring 3/syscall/ELF paths are low-level and should still be tested one at a
  time; only the fixed `ring3` teaching demo has runtime proof.
- README and older roadmap/status docs can become stale if copied without the
  verification matrix.

## 9. Verification Gaps

- Follow-up product command input check for `poster system` and
  `capsule current` using manual typing or a better input harness; the rest of
  the 2026-06-13 scoped product-safe path has narrow QEMU evidence.
- GIF/tooling status; 2026-06-13 captured PNG screenshots but did not attempt a
  GIF.
- ChronoFS follow-up pass for independent write persistence before deletion,
  controlled repair, journal rollback/roll-forward, corrupt journal refusal, and
  heap fallback on throwaway images.
- User-space follow-up for exact `userspace elf`, exact `syshello`, and
  `exec <name>` after `ld.lld` or an equivalent safe linker path is available
  and a known static ELF is installed into a disposable image.
- Window/input follow-up for manual typing, Backspace, Shift, exact
  `windows list`, successful `windows close <id>`, `tasks`, `kill
  <observed-task-id>`, pointer movement, drag, mouse close, task/window
  cleanup, and heap reuse.
- Networking QEMU pass for `net status`, `net config`, `net arp`, `net send`,
  `net log`, ARP reply learning, UDP transmit/receive, and host-to-guest setup.
- UEFI loader follow-up for the `Out of Resources` failure before framebuffer
  handoff.
- NASM availability and custom BIOS build-only pass before custom BIOS boot.
- SMP/AP follow-up to find why the two-core smoke did not show AP-online or
  boot-complete evidence.
- Real hardware plan only after QEMU BIOS/UEFI/storage/input paths are cleaner.

## 10. Recommended Next Product Direction

Recommended v0.3 direction:

- Primary track: Reliability and verification.
- Secondary track: ChronoFS hardening.

ChronoOS has more than enough code-present surface area. The best next product
move is to convert existing systems into evidence-backed claims, then deepen one
serious subsystem with controlled storage tests. Networking expansion, USB,
dynamic linking, package management, a full compositor, and preemptive
scheduling should stay deferred until the current shell, storage, UI, and
userspace boundaries have better proof.

Recommended next 5 Codex prompts after the 2026-06-13 userspace-boundary pass:

1. Run a focused window lifecycle cleanup pass for `windows close <id>`,
   `tasks`, and `kill <observed-task-id>` using a fresh disposable QEMU image
   and real observed IDs. Do not run `kill 0`; do not claim success unless
   serial and screenshots show removal.
2. Run a focused product-command input cleanup pass for `poster system` and
   `capsule current` using manual typing or a better input harness. Do not
   claim success unless the serial log shows exact command lines and screenshots
   show the output.
3. Run a controlled ChronoFS repair/recovery QEMU pass on throwaway images for
   independent write persistence, `fsck repair` on safe synthetic damage,
   repair refusal cases, and journal rollback/roll-forward/corrupt-record
   handling. Do not touch the repo data image.
4. Run a manual keyboard and mouse interaction pass for manual typing,
   Backspace, Shift, visible cursor movement, title-bar drag, and mouse close.
5. Run a userspace follow-up pass for exact `userspace elf`, exact `syshello`,
   and controlled `exec hello.elf` only after `ld.lld` or an equivalent safe
   linker path is available and the known test ELF is installed into a
   disposable image.
