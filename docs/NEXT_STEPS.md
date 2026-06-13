# Next Steps

## Current Priority Queue

1. Use `docs/POST_VERIFICATION_SUMMARY.md` as the current product/engineering decision point after the visible BIOS, ChronoFS, window/input, userspace, and UEFI verification sequence.
2. Use `docs/VERIFICATION_MATRIX.md` as the compact evidence matrix and `docs/PROGRESS_AUDIT.md` as the broader progress audit.
3. Preserve build sanity: `cargo check -p kernel --offline --locked` and the host-target `chronosapien` check now pass with no warnings.
4. Preserve runtime tooling: QEMU 11.0.1 and PowerShell 7.6.2 are installed and available on PATH.
5. Treat the BIOS product path as the safest current demo base: single-core BIOS boot, serial logs, framebuffer shell, screenshots, `start`, `guide quick`, `mode status`, `safe on`, `doctor`, and `apps list` have QEMU evidence.
6. Make input/window lifecycle stabilization the primary next track before broader UI polish: `windows close <id>`, `tasks`, `kill <observed-task-id>`, manual typing, Backspace, Shift, visible mouse movement, drag, and mouse close still need clean proof.
7. Use shell workspace polish as the secondary product track once the input/window lifecycle gap is reduced.
8. Keep ChronoFS repair/recovery, userspace syscall/ELF follow-up, and UEFI `Out of Resources` as separate narrow technical tracks.
9. Record any successful QEMU or hardware evidence in `docs/AI_PROGRESS_LOG.md` before upgrading status labels from "needs runtime verification".
10. Keep source/docs aligned as small systems are verified; do not start large new feature tracks yet.

## Progress Audit Note

`docs/POST_VERIFICATION_SUMMARY.md` is the newest consolidation after five
verification passes. It recommends input/window lifecycle stabilization as the
primary track and shell workspace polish as the secondary product track.
ChronoFS repair/recovery, userspace syscall/ELF execution, and UEFI loader
failure work remain focused technical tracks. Networking expansion, USB,
dynamic linking, package management, full compositing, and preemptive
scheduling are intentionally not the immediate next track.

## Repository Workflow

- Active integration branch: `main`.
- Public/product name: ChronoOS.
- Repo/package/image names may remain `chronosapien` until a dedicated rename task.
- Push future completed work to `origin/main` unless a task explicitly asks for a separate branch or pull request.

## Technical Gaps

- Interrupt-driven keyboard: implemented in code; IRQ1 buffering exists and polling fallback remains, but it needs runtime verification.
- Reusable heap allocator: implemented in code; the allocator is a simple free list, but reuse behavior needs build/runtime validation.
- ChronoFS repair/journaling: implemented in code; read-only `fs` inspection, `fsck`, `fsck repair`, and a tiny one-record journal exist, but shell workflows and recovery scenarios need runtime verification.
- Graphics shell: partially implemented; framebuffer console, mouse cursor, and small draggable windows exist, but there is no full desktop/compositor.
- Process model: partially implemented; ring 3, syscalls, static ELF execution, and read-only `userspace` inspection exist, but there is no dynamic linker, argv/env, or general multiprocess model.
- Scheduler: partially implemented; cooperative task scheduling exists, while preemption and production-grade scheduling are roadmap/design-only.
- Networking: partially implemented; ARP/UDP over RTL8139 plus read-only
  observability commands exist in code, while DHCP, DNS, TCP, sockets, packet
  capture, and broad hardware support are roadmap/design-only.
- Real hardware/USB: partially implemented through the UEFI boot path; USB HID, USB storage, USB serial, and broad hardware support are roadmap/design-only.

## Phase 2 Verification Order

1. Visual BIOS QEMU pass: verify framebuffer text, shell prompt, PIT/timer, and keyboard input.
2. SMP/AP follow-up: investigate why the two-core BIOS serial smoke exits before `[CHRONO] boot complete`.
3. Storage smoke pass: `fs status`, `fs info`, `ls`, `write`, `cat`, `rm`, `fs check`, `fs journal`, `fsck`, controlled `fsck repair`, and `journal`.
4. Product/app pass: `demo`, `tour`, `capsule`, `doctor`, `poster`, `travel <year>`, `apps`, `notes`, `calc`, and `sysinfo`.
5. Window/input pass: PS/2 mouse, cursor, `open notes`, `open sysinfo`, drag, focus, close, `tasks`, and `kill`.
6. Risky systems last: custom BIOS path, UEFI path, SMP/AP startup, `userspace status`, ring 3, syscalls, static ELF execution, and ARP/UDP networking via `net status`, `net config`, `net arp`, `net log`, and `net send`.

## Product / Indie Features

- Implemented in code, needs runtime verification: `demo`, `tour`, `capsule`, `doctor`, `poster`, `travel <year>`, and `apps`.
- Implemented in code, needs runtime verification: expanded museum pages for disk, filesystem, userspace, syscalls, ELF, networking, SMP, and scheduler.
- Partially implemented: app launcher, notes flow, small windows, quest/status/inventory screens.
- Roadmap/design-only: theme studio, crash lab, tiny paint, file explorer window mode, boot chime selector, network demo mode, stronger app launcher, and richer era-specific behavior.
- Partially implemented but not polished showcase features: user-space showcase through `ring3`/`syshello`/`exec` and text-only visual boot timeline surfaces through `capsule` / `poster boot`.

## Do Not Build Yet

Features that are too big or risky right now:

- Full TCP/IP stack
- DHCP, DNS, or TCP
- Full desktop compositor
- Dynamic linker
- Package manager
- Complex GUI toolkit
- Full USB stack
- Browser
- Production-grade preemptive scheduler
