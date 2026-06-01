# ChronoOS Current Status

Date: 2026-06-01

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
- Runtime tooling: QEMU 11.0.1 and PowerShell 7.6.2 were installed and verified
  locally on 2026-06-01.
- Runtime proof: single-core BIOS serial-only QEMU reached `[CHRONO] boot
  complete` on 2026-06-01.
- Important limit: that QEMU smoke used `-display none`, so framebuffer output,
  visible shell prompt, keyboard input, apps, storage commands, windows, and
  graphics were not verified by that run.
- Important SMP limit: a two-core BIOS serial-only QEMU smoke exited before
  `[CHRONO] boot complete`, after `[CHRONO] active era: 1984`.
- Hardware proof: no hardware verification is recorded in this repo.

## Compact Status Table

| Feature | Status | Verification | Notes |
| --- | --- | --- | --- |
| Public ChronoOS identity | implemented in code | not runtime-facing | Public docs use ChronoOS; repo/package names may remain `chronosapien`. |
| BIOS boot path | implemented in code | verified in QEMU, needs broader runtime verification | Single-core serial-only BIOS boot reached `[CHRONO] boot complete`; framebuffer, shell, and multi-core still need verification. |
| Custom BIOS bootloader path | partially implemented | needs runtime verification | Stage/handoff code and scripts exist; no boot proof is recorded. |
| UEFI loader path | implemented in code | needs runtime verification | Loader, docs, and scripts exist; no UEFI QEMU/hardware proof is recorded. |
| Framebuffer console/UI | implemented in code | needs runtime verification | Text rendering/top bar paths exist; not verified because the recorded smoke was serial-only. |
| Serial logging | implemented in code | verified in QEMU, needs broader runtime verification | Boot-time serial logging reached boot complete in single-core QEMU; shell-command serial output still needs checks. |
| Shell command surface | implemented in code | needs runtime verification | Dispatch exists for core, product, app, filesystem, userspace, networking, museum, and quest commands. |
| Apps | implemented in code | needs runtime verification | `notes`, `calc`, `sysinfo`, and `apps` launcher paths exist. |
| Museum/quest/product layer | implemented in code | needs runtime verification | `demo`, `tour`, `capsule`, `doctor`, `poster`, `travel`, museum pages, quests, stats, and inventory exist. |
| Theme studio | roadmap/design-only | needs runtime verification | Current `apps theme` is a text preview, not a studio. |
| Crash lab | roadmap/design-only | needs runtime verification | Do not build before core runtime evidence is stronger. |
| Tiny paint | roadmap/design-only | needs runtime verification | No paint app is implemented. |
| File explorer | roadmap/design-only | needs runtime verification | Current `apps files` points to shell file commands; no windowed explorer exists. |
| Boot chime selector | roadmap/design-only | needs runtime verification | Era tones exist; no user-facing selector exists. |
| Network demo mode | roadmap/design-only | needs runtime verification | Current networking surface is `net`, `net arp`, and `net send`. |
| User-space showcase | partially implemented | needs runtime verification | `ring3`, `syshello`, `exec`, and museum/tour pages exist; no polished showcase app exists. |
| Visual boot timeline | partially implemented | needs runtime verification | `capsule` and `poster boot` are text surfaces; no visual timeline is implemented. |
| Era-specific help/about | partially implemented | needs runtime verification | `about`, `era`, `travel`, and product text use era context; `help` is still mostly generic. |
| Mini desktop/app launcher | partially implemented | needs runtime verification | Text launcher plus small notes/sysinfo windows exist; not a full desktop. |
| ChronoFS | implemented in code | needs runtime verification | ATA-backed named files and shell commands exist; shell workflows are not verified. |
| fsck and journal | implemented in code | needs runtime verification | Conservative `fsck`, `fsck repair`, one-record journal, and mount recovery exist; crash states need controlled tests. |
| Keyboard | implemented in code | needs runtime verification | IRQ1 buffering plus polling fallback exist. |
| Mouse | implemented in code | needs runtime verification | PS/2 mouse path initialized in code; movement/window interaction not verified. |
| Window manager | partially implemented | needs runtime verification | Fixed-capacity windows, focus, drag, close, notes/sysinfo windows, and task wiring exist. |
| Heap allocator | implemented in code | needs runtime verification | Free-list allocator with splitting, reinsertion, and coalescing exists; reuse/corruption behavior needs testing. |
| Cooperative scheduler | implemented in code | needs runtime verification | Fixed task slots and spawn/kill/yield paths exist. |
| SMP/AP startup | partially implemented | needs runtime verification | High-risk; two-core serial-only smoke exited before boot complete. |
| Ring 3/syscalls/ELF | partially implemented | needs runtime verification | Teaching paths exist; no general userland, dynamic linker, argv/env, libc, or package model. |
| Networking | partially implemented | needs runtime verification | RTL8139, static IPv4, ARP, UDP send/poll exist; no DHCP, DNS, TCP, or socket stack. |
| Long-term systems 96-110 | roadmap/design-only | needs runtime verification | See the long-term section below; current repo should not imply these are built. |

## Boot Paths

- BIOS boot through the `bootloader` crate is implemented in code and has
  limited QEMU evidence: single-core serial-only boot reached `[CHRONO] boot
  complete`.
- Custom BIOS boot is partially implemented. Keep it separate from the normal
  BIOS path because it has its own stage/handoff risk.
- UEFI boot is implemented in code, but no UEFI QEMU or hardware proof is
  recorded.
- Framebuffer and shell startup are not verified by the serial-only boot smoke.

## Framebuffer, Serial, And UI

- Serial boot logging is verified in QEMU only for the single-core BIOS
  serial-only boot path.
- Framebuffer text rendering, top bar drawing, cursor behavior, and visual shell
  prompt remain needs runtime verification.
- The UI should stay terminal-first until visual shell and input evidence exists.

## Shell Command Surface

- Core shell commands are implemented in code: `help`, `clear`, `about`,
  `reboot`, `era`, `uptime`, `clock`, `mem`, `cores`, and `beep <hz>`.
- Product and guide commands are implemented in code: `demo`, `tour`,
  `capsule`, `doctor`, `poster`, `travel <year>`, and `apps`.
- Filesystem commands are implemented in code: `ls`, `cat`, `write`, `rm`,
  `fsck`, `fsck repair`, and `journal`.
- Userspace commands are implemented in code: `ring3`, `syshello`, and
  `exec <name>`.
- Networking commands are implemented in code: `net`, `net arp`, and
  `net send [ip port text]`.
- Museum and quest commands are implemented in code; see
  `docs/shell-commands.md` for the command reference.

## Apps

- `notes`, `calc`, and `sysinfo` are implemented in code.
- Notes storage is standardized around the ChronoFS file named `notes`.
- `apps` is a text-first launcher that routes to existing shell/app surfaces.
- App launch, notes persistence, and app output still need runtime verification.

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
- `help` remains mostly generic and can be made more era-aware later.

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

- Keyboard status: implemented in code, needs runtime verification.
- Keyboard notes: IRQ1 buffering and polling fallback both exist.
- Mouse status: implemented in code, needs runtime verification.
- Mouse notes: PS/2 packet decoding and window interaction hooks exist, but
  visual movement and drag/close behavior are not verified.

## Window Manager

- Status: partially implemented.
- Fixed-capacity windows, focus, drag, close, and notes/sysinfo window paths
  exist.
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
- The recorded two-core BIOS serial-only smoke did not reach boot complete, so
  do not expand SMP until this is isolated.
- Preemptive scheduling is roadmap/design-only.

## Ring 3, Syscalls, And ELF

- Status: partially implemented, risky.
- Ring 3 demo, tiny syscall ABI, and static ELF execution paths exist.
- This is not general userland: no dynamic linker, package model, argv/env,
  libc, permissions model, or robust multi-process lifecycle is implemented.

## Networking

- Status: partially implemented.
- RTL8139 discovery, static IPv4, ARP, UDP send, and polling receive paths exist.
- Serial boot showed NIC discovery during the limited BIOS smoke, but ARP/UDP
  behavior itself remains needs runtime verification.
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
  not reach boot complete.
- Ring 3/syscalls/static ELF: low-level privilege transitions and loading paths
  are inherently fragile and still unverified.
- ChronoFS repair/recovery: writes metadata during repair/recovery and must be
  tested only with controlled disk images.
- Heap allocator: reusable allocator exists but needs stress/reuse checks.
- Custom BIOS and UEFI boot: code-present paths require separate boot evidence.
- Networking: keep limited to static IPv4 ARP/UDP until runtime evidence exists.

## Recommended Next Goal

Run a visible BIOS QEMU verification pass: framebuffer text, shell prompt,
keyboard input, `help`, `about`, `uptime`, `mem`, `ls`, `write`, `cat`, `rm`,
`fsck`, `journal`, `apps`, `demo`, `tour`, `capsule`, `doctor`, `poster`, and
one notes/calc/sysinfo flow. Record screenshots or serial logs in
`docs/AI_PROGRESS_LOG.md` before upgrading any status labels.
