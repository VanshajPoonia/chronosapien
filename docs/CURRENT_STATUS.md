# ChronoOS Current Status

Date: 2026-06-02

This is the compact post-Phase-4 source-of-truth handoff for ChronoOS. It
summarizes what is implemented in code, what is partial, what is only roadmap or
design, and what has actual verification evidence.

## Project Identity

- Public/product name: ChronoOS.
- Repo/package/image names: `chronosapien`, `chronosapien-bios.img`, and
  `chronosapien-uefi.img` may remain until a dedicated rename task.
- Legacy/internal names: older progress-log entries and some internal context
  may mention Chronosapian or Time Capsule OS as historical names.
- One-line concept: ChronoOS is a beginner-friendly Rust `no_std` x86_64
  educational hobby OS with era themes, a terminal-first product shell, museum
  pages, quests, small apps, ChronoFS, and carefully labeled teaching systems.

## Status Labels

- implemented in code: source paths or shell commands exist.
- partially implemented: useful teaching version with known limits.
- needs runtime verification: QEMU or hardware evidence is still required.
- verified in QEMU: actual QEMU evidence is recorded in this repo.
- verified on hardware: actual hardware evidence is recorded in this repo.
- roadmap/design-only: documented direction only; do not present as built.

## Verification Evidence

- Build sanity: `cargo check -p kernel --offline --locked` passed with no
  warnings on 2026-06-01.
- Host package sanity: `cargo check -p chronosapien --target
  aarch64-apple-darwin --offline --locked` passed with no warnings on
  2026-06-01.
- High-risk verification preflight: `cargo check -p kernel --offline --locked`
  and `cargo check -p chronosapien --target aarch64-apple-darwin --offline
  --locked` passed again on 2026-06-02 before UEFI/custom BIOS/SMP/networking
  checks.
- Runtime tooling: QEMU 11.0.1 and PowerShell 7.6.2 were installed and verified
  locally on 2026-06-01.
- High-risk tooling proof: PowerShell, QEMU, Rustup, OVMF
  `/opt/homebrew/share/qemu/edk2-x86_64-code.fd`, and `nc` were available on
  2026-06-02. `nasm` was not available, so the custom BIOS build/run path was
  blocked by a missing build dependency.
- Runtime proof: single-core BIOS serial-only QEMU reached `[CHRONO] boot
  complete` on 2026-06-01.
- Runtime proof: a visible single-core BIOS QEMU run on 2026-06-02 reached
  `[CHRONO] boot complete`, captured serial output at
  `/private/tmp/chronoos-qemu-20260602-013807.serial.log`, and captured
  framebuffer screenshots through QEMU `screendump`.
- Visible UI proof: QEMU screendumps on 2026-06-02 showed the ChronoOS top bar,
  boot text, and `CHRONO>` shell prompt. Additional screendumps showed the
  grouped `help` output and `help start` output.
- Keyboard/input proof: QEMU monitor `sendkey` input submitted `help` and
  `help start`. This verifies a narrow shell input path in QEMU, but not
  broader manual typing, Backspace, shifted symbols, or polling fallback.
- UI/app/input proof: a second visible single-core BIOS QEMU pass on
  2026-06-02 captured serial output at
  `/private/tmp/chronoos-ui-input-20260602-150049.serial.log` and screenshots
  for boot, `apps`, `notes`, `calc 6 - 7`, `open notes`, mouse movement
  attempt, and drag attempt under `/private/tmp/chronoos-ui-input-20260602-150049-*.png`.
  Observed serial lines include `cmd: apps`, `cmd: notes`, `cmd: calc 6 - 7`,
  `app: calc launched`, `cmd: open notes`, `wm: open notes`, and
  `mouse: click at 740,410`.
- Important limit: the 2026-06-02 input run became unreliable when a longer
  command batch was injected. Backspace produced `abouut`, `sysinfo` was
  submitted as `ssysinfo`, and an early `open notes` attempt was submitted as
  `oopen notes`. Treat this as partial keyboard evidence, not broad manual
  input verification.
- Important UI/input limit: `apps`, the `notes` home screen, and `calc 6 - 7`
  were observed; `open notes` partially opened a window/task path; and one
  mouse click packet was logged. `sysinfo`, `open sysinfo`, notes read/write,
  cursor movement, drag, close, Backspace, shifted input, polling fallback,
  GIF capture, and hardware remain unverified.
- Important UEFI limit: `pwsh -NoLogo -NoProfile -File scripts/build-uefi.ps1`
  downloaded the UEFI loader dependencies after escalation but failed to compile
  `uefi-loader` because `uefi::boot::MemoryMap` no longer exists in the current
  `uefi` crate API. No UEFI QEMU boot was attempted.
- Important SMP limit: a 2026-06-02 two-core BIOS serial-only QEMU smoke at
  `/private/tmp/chronoos-smp-20260602-162000.serial.log` reached
  `[CHRONO] smp: BSP online (core 0)` and `[CHRONO] active era: 1984`, but did
  not show AP startup, `smp: 2 cores ready`, or `[CHRONO] boot complete` before
  the timeboxed run was stopped.
- Important networking limit: a 2026-06-02 single-core QEMU run with RTL8139 at
  `/private/tmp/chronoos-net-20260602-162000.serial.log` reached boot complete
  and logged `net: rtl8139 found` plus MAC `52:54:00:12:34:56`. QEMU
  `hostfwd=udp::9000-:9000` conflicted with the host UDP listener, and later
  shell injection attempts submitted `n7et` and `neett`, so ARP/UDP behavior was
  not verified. Host UDP log
  `/private/tmp/chronoos-net-20260602-162000.host-udp.log` remained 0 bytes.
- Hardware proof: no hardware verification is recorded in this repo.

## Compact Status Table

| Feature | Status | Verification | Notes |
| --- | --- | --- | --- |
| Public ChronoOS identity | implemented in code | not runtime-facing | Public docs use ChronoOS; repo/package names may remain `chronosapien`. |
| BIOS boot path | implemented in code | verified in QEMU, needs broader runtime verification | Single-core BIOS boot reached `[CHRONO] boot complete` in serial-only and visible QEMU runs; multi-core still needs verification. |
| Custom BIOS bootloader path | partially implemented | blocked: build dependency missing | Stage/handoff code and scripts exist, but `nasm` was not on PATH during the 2026-06-02 preflight. |
| UEFI loader path | implemented in code | blocked: build failure | UEFI docs/scripts exist, OVMF is available, but `uefi-loader` failed to compile against the current `uefi` crate API. |
| Framebuffer console/UI | implemented in code | verified in QEMU, needs broader runtime verification | QEMU screendump showed the top bar, boot text, and `CHRONO>` prompt; broader rendering and interaction still need checks. |
| Serial logging | implemented in code | verified in QEMU, needs broader runtime verification | Boot-time serial logging reached boot complete in single-core QEMU; shell-command serial output is only partially observed. |
| Shell command surface | implemented in code | verified in QEMU, needs broader runtime verification | `help`, `help start`, and `about` were observed through visible QEMU; most commands still need staged checks. |
| Command and UX polish | implemented in code | verified in QEMU, needs broader runtime verification | Grouped `help` and `help start` were observed; other topic pages, unknown hints, and risky warnings still need checks. |
| Apps | implemented in code | partially verified in QEMU, needs broader runtime verification | Static app registry and `apps` launcher exist; `apps`, `notes`, and `calc 6 - 7` were observed; registry subcommands, `sysinfo`, notes read/write, and persistence still need checks. |
| Guided onboarding | implemented in code | needs runtime verification | `start`, `welcome`, and `guide` topic pages route first-run users toward existing safe commands. |
| Museum/quest/product layer | implemented in code | needs runtime verification | `demo`, `tour`, `capsule`, `doctor`, `poster`, `travel`, museum pages, quests, stats, and inventory exist. |
| Theme studio | roadmap/design-only | needs runtime verification | Current `apps theme` is a text preview, not a studio. |
| Crash lab | roadmap/design-only | needs runtime verification | Do not build before core runtime evidence is stronger. |
| Tiny paint | roadmap/design-only | needs runtime verification | No paint app is implemented. |
| File explorer | roadmap/design-only | needs runtime verification | Current `apps files` points to shell file commands; no windowed explorer exists. |
| Boot chime selector | roadmap/design-only | needs runtime verification | Era tones exist; no user-facing selector exists. |
| Network demo mode | roadmap/design-only | needs runtime verification | Current networking surface is static IPv4 ARP/UDP with read-only observability commands such as `net status`, `net config`, `net log`, `net demo`, and `net roadmap`; no verified packet-demo app exists. |
| User-space showcase | partially implemented | needs runtime verification | `ring3`, `syshello`, `exec`, and museum/tour pages exist; no polished showcase app exists. |
| Visual boot timeline | partially implemented | needs runtime verification | `capsule` and `poster boot` are text surfaces; no visual timeline is implemented. |
| Era-specific help/about | partially implemented | needs runtime verification | `about`, `era`, `travel`, product text, and category help exist; deeper era-specific help remains future polish. |
| Mini desktop/app launcher | partially implemented | needs runtime verification | Text launcher plus small notes/sysinfo windows exist; not a full desktop. |
| ChronoFS | implemented in code | needs runtime verification | ATA-backed named files and shell commands exist; shell workflows are not verified. |
| fsck and journal | implemented in code | needs runtime verification | Conservative `fsck`, `fsck repair`, one-record journal, and mount recovery exist; crash states need controlled tests. |
| Keyboard | implemented in code | verified in QEMU, needs broader runtime verification | QEMU monitor `sendkey` submitted shell commands; manual typing, Backspace, Shift, and polling fallback still need checks. |
| Mouse | implemented in code | partially verified in QEMU, needs broader runtime verification | Serial log showed PS/2 initialization and one click at `740,410`; cursor movement, drag, and close are not verified. |
| Window manager | partially implemented | partially verified in QEMU, needs broader runtime verification | `open notes` spawned a task and logged `wm: open notes`; shell lifecycle commands exist; `open sysinfo`, `windows` commands, drag, close, and focus behavior are not verified. |
| Screenshot/GIF evidence | partially implemented | partially verified in QEMU, needs manual verification | QEMU `screendump` PNG capture works; animated GIF capture still needs manual tooling/evidence. |
| Heap allocator | implemented in code | needs runtime verification | Free-list allocator with splitting, reinsertion, and coalescing exists; reuse/corruption behavior needs testing. |
| Cooperative scheduler | implemented in code | needs runtime verification | Fixed task slots and spawn/kill/yield paths exist. |
| SMP/AP startup | partially implemented | partially verified in QEMU, high-risk, needs runtime verification | Two-core serial smoke reached BSP startup only; no AP-online or two-core-ready evidence. |
| Ring 3/syscalls/ELF | partially implemented | needs runtime verification | Teaching paths and read-only `userspace` inspection exist; no general userland, dynamic linker, argv/env, libc, or package model. |
| Networking | partially implemented | partially verified in QEMU | RTL8139 discovery and MAC were observed; ARP/UDP commands were not verified because QEMU input garbled `net` commands and host UDP saw 0 bytes. No DHCP, DNS, TCP, or socket stack. |
| Long-term systems 96-110 | roadmap/design-only | needs runtime verification | See the long-term section below; current repo should not imply these are built. |

## Boot Paths

- BIOS boot through the `bootloader` crate is implemented in code and has
  QEMU evidence: single-core serial-only and visible-display runs reached
  `[CHRONO] boot complete`.
- Custom BIOS boot is partially implemented. Keep it separate from the normal
  BIOS path because it has its own stage/handoff risk. The 2026-06-02 custom
  BIOS preflight was blocked because `nasm` was missing.
- UEFI boot is implemented in code, but the 2026-06-02 UEFI image build failed
  in `uefi-loader` before QEMU boot could be attempted.
- Framebuffer and shell startup have limited visible QEMU evidence from the
  2026-06-02 screendumps. UEFI, custom BIOS, and SMP/AP boot still need
  separate verification.

## Framebuffer, Serial, And UI

- Serial boot logging is verified in QEMU only for the single-core BIOS
  boot path.
- Framebuffer text rendering, top bar drawing, cursor behavior, and visual shell
  prompt are verified in QEMU at boot/shell-prompt level by 2026-06-02
  screendumps.
- Broader UI behavior, redraw edge cases, app/window interaction beyond the
  partial `open notes` path, and GIF capture remain needs runtime verification.
- The UI should stay terminal-first until broader visual shell and input
  evidence exists.

## Shell Command Surface

- Core shell commands are implemented in code: `help`, `clear`, `about`,
  `reboot`, `era`, `uptime`, `clock`, `mem`, `cores`, and `beep <hz>`.
- Category help is implemented in code: `help start`, `help apps`, `help fs`,
  `help system`, `help network`, `help userspace`, `help labs`, and
  `help roadmap`, with beginner-friendly topic aliases.
- Product and guide commands are implemented in code: `demo`, `tour`,
  `capsule`, `doctor`, `poster`, `travel <year>`, and `apps`.
- Guided onboarding commands are implemented in code: `start`, `welcome`,
  `guide`, `guide quick`, `guide full`, `guide eras`, `guide apps`,
  `guide systems`, `guide status`, and `guide next`.
- Filesystem commands are implemented in code: `ls`, `cat`, `write`, `rm`,
  `fsck`, `fsck repair`, and `journal`.
- Userspace commands are implemented in code: `ring3`, `syshello`, and
  `exec <name>`.
- Networking commands are implemented in code: `net status`, `net config`,
  `net arp`, `net udp`, `net send [ip port text]`, `net log`, `net demo`,
  `net roadmap`, and `net help`.
- Museum and quest commands are implemented in code; see
  `docs/shell-commands.md` for the command reference.
- Runtime evidence is still narrow but growing: `help`, `help start`, `about`,
  `apps`, `notes`, and `calc 6 - 7` were observed in visible QEMU. Longer
  command batches via QEMU monitor input were unreliable, so staged manual or
  scripted shell verification is still needed for the rest of the command
  surface.

## Apps

- `notes`, `calc`, and `sysinfo` are implemented in code.
- Notes storage is standardized around the ChronoFS file named `notes`.
- `apps` is a text-first launcher that routes to existing shell/app surfaces.
- `apps`, the `notes` home screen, and `calc 6 - 7` were observed in QEMU on
  2026-06-02.
- `sysinfo`, notes read/write, notes persistence, and broader app output still
  need runtime verification.

## Museum, Quest, And Product Layer

- Museum pages, quest list/status, stats, inventory, demo, tour, capsule,
  doctor, poster, travel, and launcher text are implemented in code.
- These surfaces should remain honest about runtime status and should not claim
  QEMU/hardware verification unless progress-log evidence exists.

## Theme Studio

- Status: roadmap/design-only.
- Current source has era profiles and an `apps theme` preview, but no editor or
  theme studio workflow.

## Crash Lab

- Status: roadmap/design-only.
- Keep this deferred until panic/fault display, serial evidence, and controlled
  recovery docs are stronger.

## Tiny Paint

- Status: roadmap/design-only.
- No paint canvas, drawing tools, or file format are implemented.

## File Explorer

- Status: roadmap/design-only for window mode.
- Current `apps files` is a text card that points to `ls`, `cat`, `write`,
  `rm`, `fsck`, and `journal`.

## Boot Chime Selector

- Status: roadmap/design-only.
- Era-specific tones exist in code, but there is no selector or persisted user
  choice.

## Network Demo Mode

- Status: roadmap/design-only.
- Current networking is static IPv4 ARP/UDP teaching code and shell commands,
  not a guided demo mode.

## User-Space Showcase

- Status: partially implemented.
- `ring3`, `syshello`, `exec`, and userspace museum/tour pages exist.
- This is not a general process platform and not a polished showcase app yet.

## Visual Boot Timeline

- Status: partially implemented.
- `capsule` and `poster boot` provide text timeline/status surfaces.
- A graphical boot timeline remains roadmap/design-only.

## Era-Specific Help And About

- Status: partially implemented.
- Era profiles, `era`, `travel`, `about`, and many product texts use the active
  era.
- `help` now has category and topic pages, but deeper era-specific phrasing can
  still be made richer later.

## Mini Desktop And App Launcher

- Status: partially implemented.
- A text app launcher, cooperative tasks, and small notes/sysinfo windows exist.
- This is not a full desktop, compositor, or GUI toolkit.

## ChronoFS

- Status: implemented in code.
- ChronoFS supports a simple educational layout with superblock, file table,
  allocation bitmap, and contiguous file data.
- Runtime verification still needs `ls`, `write`, `cat`, `rm`, persistence, and
  failure-case checks.

## fsck And Journal

- Status: implemented in code, risky.
- `fsck`, conservative `fsck repair`, a one-record journal, mount recovery, and
  bitmap rebuilding paths exist.
- Recovery is intentionally limited and must be tested against controlled clean,
  intent, committed, corrupt, and ambiguous disk states.

## Keyboard And Mouse

- Keyboard status: implemented in code, verified in QEMU for narrow command
  submission, needs broader runtime verification.
- Keyboard notes: IRQ1 buffering and polling fallback both exist. QEMU monitor
  `sendkey` successfully submitted `help`, `help start`, `apps`, `notes`, and
  `calc 6 - 7`, but manual typing, Backspace, shifted keys, and polling fallback
  still need verification. The Backspace attempt was not verified because the
  observed command became `abouut`.
- Mouse status: implemented in code, partially verified in QEMU, needs broader
  runtime verification.
- Mouse notes: PS/2 initialization appeared in the serial boot log, and a QEMU
  monitor click generated `[CHRONO] mouse: click at 740,410`. Visual cursor
  movement, drag, close, and reliable window interaction are not verified.

## Window Manager

- Status: partially implemented.
- Fixed-capacity windows, focus, drag, close, notes/sysinfo window paths, and
  shell lifecycle commands exist.
- `open notes` was partially verified in QEMU: it spawned a notes task, logged
  `wm: open notes`, and produced a visible window boundary. `open sysinfo`,
  `windows` commands, dragging, closing, and focus behavior are not verified.
- Treat this as a teaching window layer, not a compositor.

## Heap Allocator

- Status: implemented in code, risky.
- The heap uses a free-list allocator with block splitting, freeing, sorted
  reinsertion, and coalescing.
- It lacks production-grade corruption/double-free protection and needs reuse
  testing across shell, app, task, and filesystem workflows.

## Scheduler And SMP

- Cooperative scheduler status: implemented in code, needs runtime
  verification.
- SMP/AP startup status: partially implemented, risky, needs runtime
  verification.
- The 2026-06-02 two-core BIOS serial-only smoke reached BSP startup but did not
  show AP startup, `smp: 2 cores ready`, or boot complete, so do not expand SMP
  until this is isolated.
- Preemptive scheduling is roadmap/design-only.

## Ring 3, Syscalls, And ELF

- Status: partially implemented, risky.
- Ring 3 demo, tiny syscall ABI, static ELF execution paths, and read-only
  `userspace status|syscalls|elf|roadmap` inspection commands exist.
- This is not general userland: no dynamic linker, package model, argv/env,
  libc, permissions model, or robust multi-process lifecycle is implemented.

## Networking

- Status: partially implemented.
- RTL8139 discovery, static IPv4, ARP, UDP send, and polling receive paths exist.
- Serial boot showed RTL8139 discovery and the expected MAC address during a
  2026-06-02 single-core QEMU networking pass.
- ARP/UDP behavior itself remains needs runtime verification because
  `hostfwd=udp::9000-:9000` conflicted with the local UDP listener and QEMU
  monitor input submitted garbled commands (`n7et`, `neett`) instead of clean
  `net` commands.
- DHCP, DNS, TCP, sockets, and broad hardware support are roadmap/design-only.

## Long-Term Systems 96-110

Treat these as roadmap/design-only unless a future source audit finds matching
code and a progress-log entry records the change:

- DHCP.
- DNS.
- TCP.
- Socket-like API.
- USB HID.
- USB storage.
- USB serial.
- Real-hardware boot hardening beyond the current BIOS/UEFI teaching paths.
- Stronger process model and multi-process user mode.
- Preemptive scheduler.
- Dynamic app loading beyond current static ELF teaching paths.
- Package/app manager.
- Richer GUI shell.
- Full desktop compositor.
- More complete networking stack.

## Risky Areas

- SMP/AP startup: currently high-risk because the two-core serial-only smoke did
  not show AP startup or boot complete.
- Ring 3/syscalls/static ELF: low-level privilege transitions and loading paths
  are inherently fragile and still unverified.
- ChronoFS repair/recovery: writes metadata during repair/recovery and must be
  tested only with controlled disk images.
- Heap allocator: reusable allocator exists but needs stress/reuse checks.
- Custom BIOS and UEFI boot: code-present paths are currently blocked by build
  issues or missing dependencies before runtime boot evidence.
- Networking: keep limited to static IPv4 ARP/UDP; only RTL8139 init/MAC is
  partially verified so far.

## Recommended Next Goal

Fix the UEFI loader API mismatch or install NASM only as a targeted build-path
task, then rerun one high-risk verification path at a time. For networking, use
a more reliable shell input path before attempting `net arp` or `net send`
again.
