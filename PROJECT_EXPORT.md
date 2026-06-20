# ChronoOS Project Export

## 1. Project Name And Purpose

Public/product identity: **ChronoOS**. Repo package name in `Cargo.toml` is
still `chronosapien`, and the README title is `Chronosapian`. Per
`AGENTS.md`, this naming split is intentional and should not be renamed
without an explicit task.

ChronoOS is a beginner-friendly Rust `no_std` x86_64 hobby/educational
operating system. v0.3 theme: "ChronoOS as a tiny educational OS workspace."
It builds for QEMU with a BIOS image path (primary, demo-safe) and a UEFI
loader path (build-fixed, boot not yet verified), and exposes kernel concepts
through a framebuffer shell with eras, museum pages, quests, learning paths,
small built-in apps, a custom filesystem (ChronoFS), early ARP/UDP
networking, a tiny window manager, a cooperative scheduler, and a Ring
3/syscall teaching path.

This file describes current repository state and is informed by
`docs/POST_VERIFICATION_SUMMARY.md`, `docs/VERIFICATION_MATRIX.md`,
`docs/PROGRESS_AUDIT.md`, and `docs/ROADMAP_v0.3.md` (dated 2026-06-13).
Features are marked code-present unless QEMU/hardware evidence exists; do not
upgrade a status without recorded evidence in those docs.

## 2. What We Are Building

- Bootable kernel via the `bootloader` crate BIOS image path (the verified,
  demo-safe default).
- Optional custom BIOS bootloader path (Stage 1/Stage 2), currently blocked
  locally because `nasm` is not on PATH.
- Rust UEFI loader (`uefi-loader`) that builds a GPT/FAT32 ESP image; the
  build succeeds and OVMF starts the loader, but it fails with
  `Out of Resources` before kernel framebuffer handoff.
- Framebuffer console with era-specific colors/text/sound, top bar, cursor,
  window drawing, and mouse cursor rendering.
- A terminal-first product shell: guided onboarding (`start`, `guide`),
  workspace/status/verification surfaces (`workspace`, `status`, `verify`,
  `doctor`, `poster`, `capsule`), a static app registry with discovery
  metadata (`apps featured/recent/category/info/demo/launch`), a learning
  progress map (`learn`, `explain`, `museum`, `quest`), and a ChronoFS
  usability layer (`files list/info/search/sample/demo/copy`) layered over the
  base filesystem commands.
- Early kernel subsystems: GDT/TSS, IDT, PIC, PIT, serial, PS/2 keyboard
  (IRQ1 + polling fallback), PS/2 mouse IRQ handling, ATA PIO, PCI scanning,
  RTL8139 ARP/UDP networking, paging, a reusable free-list heap, SMP
  discovery/AP startup, and a cooperative scheduler.
- A tiny window manager (`windows`, `open notes`/`open sysinfo`, `tasks`,
  `kill <id>`) layered on the scheduler.
- A teaching-only userspace path: `userspace status`/`syscalls`, fixed
  `ring3` privilege-boundary demo, and one foreground static ELF execution
  path (`exec`); `docs/USERSPACE_NEXT.md` stages further work as read-only
  ELF process metadata, not a general process model.

## 3. Current Tech Stack

- Language: Rust 2021, nightly, `no_std` kernel.
- Targets in `rust-toolchain.toml`: `x86_64-unknown-none`, `x86_64-unknown-uefi`.
- Assembly: NASM-style `.asm` files for the optional custom BIOS bootloader
  (build currently blocked locally: `nasm` not found on PATH).
- User demo: freestanding C (`user/hello.c`) intended for `clang` + `ld.lld`;
  `ld.lld` is not currently on PATH, so `exec hello.elf` stays blocked.
- Build/package tools: Cargo workspace (`kernel`, `uefi-loader` members), root
  `build.rs`, standalone Rust host tools, PowerShell scripts.
- Emulation target: QEMU `qemu-system-x86_64`.
- Local tool availability as of this export: `cargo`, `rustc`,
  `qemu-system-x86_64`, and `pwsh` are present on PATH; `nasm` and `ld.lld` are
  not.
- Main Rust dependencies: root build dependency `bootloader = "0.11.15"`;
  kernel deps `bootloader_api = "0.11.15"` and `x86_64 = "0.14"`; UEFI loader
  dep `uefi = "0.37.0"`.
- `Cargo.lock` is now consistent with the manifests (lists `bootloader`,
  `bootloader_api`, `x86_64`, and the `uefi-loader` workspace member); the
  previously recorded stale-lock warning no longer applies.

## 4. Current Folder/File Structure

Tracked files from `git ls-files` (main branch):

```text
.cargo/config.toml
.gitignore
AGENTS.md
Cargo.lock
Cargo.toml
PROJECT_EXPORT.md
README.md
boot/stage1/stage1.asm
boot/stage2/stage2_long.asm
boot/stage2/stage2_pm.rs
boot/stage2/stage2_real.asm
build.rs
docs/AI_PROGRESS_LOG.md
docs/CURRENT_STATUS.md
docs/KNOWN_LIMITATIONS.md
docs/NEXT_STEPS.md
docs/POST_VERIFICATION_SUMMARY.md
docs/PROGRESS_AUDIT.md
docs/RELEASE_v0.1.md
docs/ROADMAP_AFTER_v0.1.md
docs/ROADMAP_v0.2.md
docs/ROADMAP_v0.3.md
docs/USERSPACE_NEXT.md
docs/VERIFICATION_MATRIX.md
docs/apps.md
docs/architecture.md
docs/boot-flow.md
docs/build-in-public-posts.md
docs/chronofs-hardening.md
docs/custom-bootloader.md
docs/demo-script.md
docs/dev-log.md
docs/elf.md
docs/hardware-testing.md
docs/learning-paths.md
docs/manual-testing.md
docs/networking.md
docs/portfolio-kit.md
docs/release-checklist.md
docs/resume-bullets.md
docs/ring3.md
docs/roadmap.md
docs/safe-mode.md
docs/screenshots.md
docs/shell-commands.md
docs/showcase.md
docs/status-audit.md
docs/storage.md
docs/syscalls.md
docs/uefi.md
docs/userspace-model.md
docs/windowing.md
kernel/Cargo.toml
kernel/src/apps/calc.rs
kernel/src/apps/mod.rs
kernel/src/apps/notes.rs
kernel/src/apps/sysinfo.rs
kernel/src/ata.rs
kernel/src/boot.rs
kernel/src/console.rs
kernel/src/elf.rs
kernel/src/framebuffer/font.rs
kernel/src/framebuffer/mod.rs
kernel/src/fs.rs
kernel/src/gdt.rs
kernel/src/interrupts.rs
kernel/src/io.rs
kernel/src/keyboard.rs
kernel/src/main.rs
kernel/src/memory.rs
kernel/src/mouse.rs
kernel/src/museum.rs
kernel/src/net.rs
kernel/src/panic.rs
kernel/src/pci.rs
kernel/src/pic.rs
kernel/src/process.rs
kernel/src/quest.rs
kernel/src/ring3.rs
kernel/src/sched.rs
kernel/src/serial.rs
kernel/src/shell.rs
kernel/src/smp.rs
kernel/src/sound.rs
kernel/src/spinlock.rs
kernel/src/syscall.rs
kernel/src/theme.rs
kernel/src/timer.rs
kernel/src/wm.rs
rust-toolchain.toml
scripts/build-custom.ps1
scripts/build-uefi.ps1
scripts/build-user.ps1
scripts/build.ps1
scripts/debug-serial.ps1
scripts/run-custom.ps1
scripts/run-uefi.ps1
scripts/run.ps1
scripts/write-usb.ps1
src/main.rs
tools/chronofs_put.rs
tools/custom_image_builder.rs
tools/uefi_image_builder.rs
uefi-loader/Cargo.toml
uefi-loader/src/main.rs
user/hello.c
user/user.ld
```

The docs tree has grown substantially since the original export (roughly a
dozen new docs covering v0.1 release/checklist, v0.2/v0.3 roadmaps, the
verification matrix/progress audit/post-verification summary, safe mode,
learning paths, ChronoFS hardening, userspace staging, hardware/manual
testing, and portfolio/showcase material). `docs/PROGRESS_AUDIT.md` §4-5
tracks which docs are current vs. stale/duplicative; `docs/status-audit.md`,
`docs/roadmap.md`, `docs/ROADMAP_AFTER_v0.1.md`, and `docs/dev-log.md` are
historical/superseded rather than source-of-truth.

## 5. Important Files

- `Cargo.toml` / `build.rs`: workspace setup and BIOS image build via the
  `bootloader` 0.11 crate.
- `kernel/src/shell.rs`: command loop/dispatch; the single largest surface for
  product behavior (workspace, status, apps, files, museum/quest, userspace,
  windows, networking, safe-mode commands all route through here). Cross-ref
  `docs/shell-commands.md` for the current full command inventory.
- `kernel/src/museum.rs`, `quest.rs`: educational shell surfaces and
  RPG-style capability tracker (`learn`, `explain`, `museum`, `quest`).
- `kernel/src/apps/`: built-in apps (`notes`, `calc`, `sysinfo`) plus the
  static app-registry metadata consumed by `apps featured/category/...`.
- `kernel/src/fs.rs`, `ata.rs`: ChronoFS facade (superblock, file table,
  bitmap, contiguous extents, heap fallback) and ATA PIO disk I/O; the
  `files` usability namespace (list/info/search/sample/demo/copy) wraps this.
- `kernel/src/wm.rs`, `sched.rs`: fixed-capacity window manager and the
  cooperative task scheduler behind `windows`/`tasks`/`kill`.
- `kernel/src/ring3.rs`, `syscall.rs`, `elf.rs`, `process.rs`: Ring 3 demo,
  SYSCALL/SYSRET ABI, ELF64 parser, and the one foreground ELF process path
  behind the `userspace`/`ring3`/`syshello`/`exec` commands.
- `kernel/src/net.rs`, `pci.rs`: RTL8139 ARP/UDP stack and PCI scanning behind
  `net status/config/arp/send/log`.
- `uefi-loader/src/main.rs`: Rust UEFI app; build is fixed and OVMF starts it,
  but it currently fails with `Out of Resources` before kernel handoff — see
  `docs/uefi.md` and the UEFI rows in `docs/VERIFICATION_MATRIX.md`.
- `tools/chronofs_put.rs`, `custom_image_builder.rs`, `uefi_image_builder.rs`:
  host-side image builders for ChronoFS data, custom BIOS, and UEFI ESP images.
- `docs/POST_VERIFICATION_SUMMARY.md`: current product/engineering decision
  point — read this first for "what's actually safe to build on."
- `docs/VERIFICATION_MATRIX.md`: compact per-feature evidence lookup.
- `docs/PROGRESS_AUDIT.md`: broader evidence audit plus the current command
  inventory (§3) and doc-currency notes (§4-5).
- `docs/ROADMAP_v0.3.md`: current release-planning source and track
  evaluation table.
- `docs/NEXT_STEPS.md`: active priority queue; points back at the four docs
  above.
- `docs/USERSPACE_NEXT.md`: staged design doc limiting next userspace work to
  read-only foreground ELF metadata.

## 6. Features Already Implemented In Code

Verified in QEMU (has recorded serial/screenshot evidence, per
`docs/VERIFICATION_MATRIX.md`):

- Single-core BIOS boot, serial logging, framebuffer shell (top bar, prompt,
  scrollable text), and PNG screendump capture.
- Onboarding: `start`, `guide quick`.
- Safe/status: `mode status`, `safe on`, `doctor`.
- App launcher: `apps`, `apps list`, notes home, `calc 6 - 7`, `open notes`,
  `open sysinfo`.
- ChronoFS on a disposable image: `fs status`, `fs info`, `ls`, `write`,
  `cat`, `rm`, `fs check`, `fs journal`, `fsck`, `journal`, and delete
  persistence across reboot.
- Window lifecycle (narrow): `windows status`, `windows`/list alias, `windows
  focus 1`, serial-backed `windows close 2`; one mouse click packet.
- Userspace boundary: `userspace status`, `userspace syscalls` (read-only
  table), and the fixed `ring3` demo (kernel entry, privilege transition,
  expected GP-fault catch on a privileged instruction).
- RTL8139 device init and MAC discovery.

Implemented in code, not yet runtime-verified:

- Shell workspace polish: `workspace`, `shortcuts`, `whereami`, `recent`,
  `status`, `verify`, `theme`, `help search <term>`, typo suggestions.
- App platform polish: `apps featured`, `apps recent`, `apps category`,
  `apps help`, `apps demo`, `apps launch`, `apps verified`, `apps roadmap`,
  richer manifest metadata, verification badges, risk labels.
- ChronoFS usability layer: `files`, `files list/info/search/sample/demo`,
  non-overwriting `files copy`, refusing `files rename`.
- Learning progress map: `learn map/progress/beginner/advanced/next`,
  `explain <term>`, `museum index`, `quest dependencies`, `quest badges`.
- UEFI build path: `cargo build -p uefi-loader --target x86_64-unknown-uefi`
  and `scripts/build-uefi.ps1` pass; OVMF starts the loader (see §7 for the
  boot-time failure).
- Reusable free-list heap allocator (split/free/reinsert/coalesce).
- Cooperative scheduler task spawn/kill plumbing beyond the one observed
  `sched: killed task 2` event.
- Static ELF64 parser and foreground ELF execution path (blocked from running
  end-to-end only by missing `ld.lld`/no installed test ELF, not by missing
  code).

## 7. Features Partially Implemented Or Limited

- **Window/input lifecycle** — the single biggest product-polish risk per
  `docs/POST_VERIFICATION_SUMMARY.md`: visual close confirmation after
  `windows close <id>`, `tasks`, `kill <observed-id>`, manual keyboard typing,
  Backspace, Shift, visible mouse movement, drag, and mouse close all still
  need clean proof. QEMU monitor key injection has repeatedly garbled longer
  commands (`poster system`, `capsule current`, `windows list`, ChronoFS
  command attempts).
- **UEFI boot** — build/image path is fixed and OVMF starts the ChronoOS
  loader, but it fails with `Out of Resources` before kernel framebuffer
  handoff. Treated as a separate technical track, not a v0.3 blocker.
- **Userspace/process model** — `userspace elf`, exact `syshello`, and
  controlled `exec hello.elf` remain unverified/blocked because `ld.lld` is
  not available and input garbling has prevented exact command capture. No
  dynamic linker, libc, argv/env, or general multi-process model exists by
  design (see `docs/USERSPACE_NEXT.md`).
- **Custom BIOS boot** — blocked locally because `nasm` is not on PATH;
  `scripts/build-custom.ps1`/`run-custom.ps1` have not been run.
- **SMP/AP** — two-core smoke reaches BSP-online only; no AP-online or
  two-core-ready evidence recorded. Treated as high-risk.
- **Networking** — ARP/UDP over RTL8139 plus new `net status/config/log/demo`
  observability commands exist in code; ARP/UDP runtime behavior is
  unverified (input garbling, host UDP forwarding conflicts in the recorded
  pass). No DHCP/DNS/TCP/sockets.
- **ChronoFS repair/recovery** — `fsck repair`, controlled bitmap/stale-slot
  repair, journal rollback/roll-forward, corrupt-journal refusal, and crash
  recovery are implemented but unverified; independent write persistence
  before deletion is also unverified (only delete-then-reboot persistence has
  evidence).
- Docs can lag source/evidence; always check `docs/VERIFICATION_MATRIX.md`
  before trusting a status claim elsewhere.

## 8. Planned But Not Started

Intentionally deferred for v0.3 (`docs/ROADMAP_v0.3.md`):

- TCP, DHCP, DNS, sockets, and broader networking.
- USB HID, USB storage, USB serial, and broad real-hardware support.
- Dynamic linker, package manager, dynamic app loading, app store behavior.
- Full compositor, GUI toolkit, windowed file explorer, tiny paint canvas,
  theme studio, crash lab, boot chime selector.
- Preemptive scheduler, production process scheduling, and SMP/AP expansion.
- Full userspace process model: fork/exec semantics, argv/env, libc, broad
  file descriptors, permissions.

## 9. Current Build, Run, And Test Commands

```powershell
cargo build -p kernel
.\scripts\build.ps1
.\scripts\run.ps1
.\scripts\debug-serial.ps1
.\scripts\build-custom.ps1   # blocked locally: nasm not on PATH
.\scripts\run-custom.ps1
.\scripts\build-uefi.ps1
.\scripts\run-uefi.ps1
.\scripts\build-user.ps1     # blocked locally: ld.lld not on PATH
```

Direct build-check commands confirmed passing per `docs/NEXT_STEPS.md`:

```bash
cargo check -p kernel --offline --locked
cargo build -p uefi-loader --target x86_64-unknown-uefi --offline --locked
```

QEMU smoke pattern used for the recorded verification passes (single-core,
serial-logged, screenshot-captured):

```bash
qemu-system-x86_64 -smp 1 -drive format=raw,file=<bios-image>,if=ide,index=0,media=disk \
  -drive format=raw,file=<chronofs-data-image>,if=ide,index=1,media=disk \
  -serial file:<path>.serial.log
```

There are no Rust unit tests in tracked source (`#[test]`/`mod tests`).
Verification is QEMU/manual/screenshot-evidence based; see
`docs/manual-testing.md` and `docs/release-checklist.md`.

## 10. Current Known Errors, Warnings, Or Broken Areas

- UEFI loader: `Out of Resources` failure before kernel framebuffer handoff
  (build/image path itself is fixed).
- Custom BIOS path: `nasm` not found on PATH locally, so this path is
  untested in this environment.
- `ld.lld` not found on PATH locally, so static ELF exec (`exec hello.elf`)
  cannot be exercised end-to-end here.
- QEMU monitor key injection garbles longer/typo-adjacent commands during
  automated verification passes (e.g. `poster system` → `possteerr ssyysttem`,
  `capsule current` → `ccapsule current`); manual GUI typing has not been
  available in this environment to confirm Backspace/Shift/typing reliability.
- `windows close 2` has only a serial-backed confirmation
  (`sched: killed task 2`, `wm: close sysinfo`); the matching screenshot was an
  inconclusive black/breakpoint-like framebuffer with no follow-up
  `windows`/`tasks` capture.
- No hardware boot, image write, or hardware serial log exists; all hardware
  claims remain unverified by design.
- `Cargo.lock` is now consistent with the manifests (previously recorded as
  stale — that issue is resolved).

## 11. Important Architecture Decisions Already Made

- Keep early boot approachable via the `bootloader` crate as the default BIOS
  image path; keep custom BIOS Stage 1/Stage 2 and the Rust UEFI loader as
  optional, separately-tracked paths.
- Use a shared kernel-side `BootContext` abstraction across bootloader-crate
  and custom/UEFI handoffs.
- Use framebuffer-provided graphics rather than a GPU/VESA driver.
- Use COM1 serial as the reliable debug/evidence path; require exact `cmd:`
  serial lines before counting a shell command as verified.
- Keep legacy PC-compatible devices as the QEMU baseline: PS/2 keyboard/mouse,
  PIC, PIT, ATA PIO IDE, RTL8139.
- Keep ChronoFS intentionally simple: fixed metadata layout, 512-byte
  sectors, contiguous extents, no journal beyond the current
  clean/empty-state journal status.
- Keep networking intentionally small: static IPv4, ARP, UDP, polling receive.
- Use Ring 3 demos and a fixed syscall ABI to teach privilege separation
  before building any general process model; the next safe step is read-only
  foreground ELF metadata, not a process table (`docs/USERSPACE_NEXT.md`).
- Treat "implemented in code" and "verified" as distinct status labels
  everywhere (`AGENTS.md`); never upgrade a status without recorded evidence.
- v0.3 release sequencing: reliability/verification is the primary track,
  window/input stabilization is secondary, userspace/process work is an
  optional stretch track — no new subsystems until existing surfaces have
  evidence (`docs/ROADMAP_v0.3.md`).

## 12. What Changed Recently

Recent commits on `main`:

```text
13f2ab6 docs: record userspace and v0.3 planning
61aecbf docs: update next steps for v0.3
694ae7a docs: add v0.3 roadmap
a5a6d79 docs: align userspace model with next steps
ad316f0 docs: add userspace process next steps
c592290 docs: refresh status trackers for product polish
19ab266 docs: update demo and manual verification paths
46040dd docs: document learning progress map
bbbe0bf docs: document ChronoFS usability layer
b551079 docs: document app platform polish
44c280e docs: update shell command reference
1a20f39 shell: add workspace app files and learning surfaces
bba27c1 learn: add museum and quest progress views
6af8709 apps: enrich static app manifests
9ba16d1 fs: add ChronoFS inspection helpers
a54c7ea record verification consolidation progress
c966224 prioritize input window verification
a985ee8 point next steps at verification summary
9a83650 add post verification summary
c402296 document UEFI loader boot result
```

This is documentation/planning-heavy: the underlying v0.1 verification pass
(BIOS, ChronoFS, window/input, userspace, UEFI loader) already landed, and
recent commits layer ChronoFS usability, app platform polish, learning
progress map, and v0.3 roadmap/next-steps planning on top of it.

## 13. What Should Be Built Next

Per `docs/ROADMAP_v0.3.md` and `docs/NEXT_STEPS.md`:

1. **Primary**: v0.3 educational-workspace verification pass — run the
   exact-next-prompt QEMU smoke test (see §14) covering `workspace`,
   `shortcuts`, `whereami`, `status`, `verify`, `help search`, the app
   platform polish commands, the learning progress map commands, and the
   ChronoFS usability commands, on visible single-core BIOS QEMU.
2. **Secondary**: window/input lifecycle stabilization — `windows close
   <id>`, `tasks`, `kill <observed-id>`, manual typing, Backspace, Shift,
   visible mouse movement, drag, and mouse close.
3. **Stretch, optional**: userspace/process foundation limited to read-only
   foreground ELF metadata per `docs/USERSPACE_NEXT.md` — do not build a
   process table.
4. Keep ChronoFS repair/recovery, userspace syscall/ELF execution, and the
   UEFI `Out of Resources` investigation as separate narrow technical tracks,
   not blockers for the primary/secondary v0.3 work.
5. Do not start new feature tracks (networking expansion, USB, dynamic
   linking, package management, full compositor, preemptive scheduling) until
   the above are evidence-backed.

## 14. Recommended Next 5 Codex Prompts

From `docs/PROGRESS_AUDIT.md` §10 (current as of the 2026-06-13
userspace-boundary pass):

1. "Run a focused window lifecycle cleanup pass for `windows close <id>`,
   `tasks`, and `kill <observed-task-id>` using a fresh disposable QEMU image
   and real observed IDs. Do not run `kill 0`; do not claim success unless
   serial and screenshots show removal."
2. "Run a focused product-command input cleanup pass for `poster system` and
   `capsule current` using manual typing or a better input harness. Do not
   claim success unless the serial log shows exact command lines and
   screenshots show the output."
3. "Run a controlled ChronoFS repair/recovery QEMU pass on throwaway images
   for independent write persistence, `fsck repair` on safe synthetic damage,
   repair refusal cases, and journal rollback/roll-forward/corrupt-record
   handling. Do not touch the repo data image."
4. "Run a manual keyboard and mouse interaction pass for manual typing,
   Backspace, Shift, visible cursor movement, title-bar drag, and mouse
   close."
5. "Run a userspace follow-up pass for exact `userspace elf`, exact
   `syshello`, and controlled `exec hello.elf` only after `ld.lld` or an
   equivalent safe linker path is available and the known test ELF is
   installed into a disposable image."

Exact next prompt from `docs/ROADMAP_v0.3.md` (the gating v0.3 smoke pass,
takes priority over the list above per `docs/NEXT_STEPS.md`):

```text
Run a v0.3 educational workspace verification pass in visible single-core BIOS QEMU. Verify workspace, shortcuts, whereami, status, verify, help search, apps featured/category/info/demo, learn map/progress/beginner/advanced, museum index, quest badges/dependencies, files list/info/search/sample/demo, and capture serial logs plus screenshots. Do not add features or upgrade labels without evidence.
```

## 15. Risks, Confusing Areas, And Cleanup Needs

- Naming is inconsistent: `Chronosapian`, `chronosapien`, and `ChronoOS` all
  appear as project/OS identifiers; intentionally left alone per `AGENTS.md`.
- Docs outnumber verified evidence: roughly 40 docs exist, but the verified
  core is narrow (single-core BIOS boot, a few shell commands, a basic
  ChronoFS CRUD flow, narrow window open/list/focus, and the fixed `ring3`
  demo). Always check `docs/VERIFICATION_MATRIX.md` before trusting a claim
  elsewhere, including this file.
- Several docs are explicitly historical/superseded:
  `docs/status-audit.md`, `docs/roadmap.md`, `docs/ROADMAP_AFTER_v0.1.md`,
  `docs/dev-log.md`, and older `docs/AI_PROGRESS_LOG.md` entries (these may
  reference an earlier "Time Capsule OS" or pure-Chronosapian naming).
- QEMU monitor key injection is an unreliable input harness for longer
  commands; future verification passes need slower one-command-at-a-time
  input or a better interactive path before claiming manual-input success.
- The `windows close 2` evidence is mixed: serial confirms the close, but the
  paired screenshot is inconclusive (black/breakpoint-like) with no
  follow-up `windows`/`tasks` capture — do not treat visual close as proven.
- SMP plus global `UnsafeCell` state needs careful review; only BSP-online
  has been observed in the recorded two-core smoke test, not AP-online.
- ChronoFS repair and journal recovery mutate metadata; any future repair
  testing must use disposable/throwaway images only, never the repo data
  image.
- `nasm` and `ld.lld` are both missing from the local PATH, which blocks
  custom BIOS build/run and static ELF exec verification until resolved.
- No automated tests exist; all verification is QEMU/manual/screenshot based,
  so evidence paths (serial logs, PNGs) are the actual source of truth, not
  prose claims in docs.
