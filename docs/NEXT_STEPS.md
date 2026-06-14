# Next Steps

## Current Priority Queue

1. Use `docs/POST_VERIFICATION_SUMMARY.md` as the current product/engineering decision point after the visible BIOS, ChronoFS, window/input, userspace, and UEFI verification sequence.
2. Use `docs/VERIFICATION_MATRIX.md` as the compact evidence matrix and `docs/PROGRESS_AUDIT.md` as the broader progress audit.
3. Use `docs/ROADMAP_v0.3.md` as the current release planning source. v0.3 theme: "ChronoOS as a tiny educational OS workspace."
4. Preserve build sanity: `cargo check -p kernel --offline --locked` and the host-target `chronosapien` check now pass with no warnings.
5. Preserve runtime tooling: QEMU 11.0.1 and PowerShell 7.6.2 are installed and available on PATH.
6. Treat the BIOS product path as the safest current demo base: single-core BIOS boot, serial logs, framebuffer shell, screenshots, `start`, `guide quick`, `mode status`, `safe on`, `doctor`, and `apps list` have QEMU evidence.
7. Make v0.3 primary work reliability and verification for the educational workspace: verify code-present workspace, app, learning, ChronoFS usability, and status commands before adding more features.
8. Make window/input lifecycle stabilization the v0.3 secondary track: `windows close <id>`, `tasks`, `kill <observed-task-id>`, manual typing, Backspace, Shift, visible mouse movement, drag, and mouse close still need clean proof.
9. Keep userspace/process foundation as an optional stretch track limited to read-only foreground ELF metadata and verification planning through `docs/USERSPACE_NEXT.md`.
10. Keep ChronoFS repair/recovery, userspace syscall/ELF follow-up, and UEFI `Out of Resources` as separate narrow technical tracks.
11. Record any successful QEMU or hardware evidence in `docs/AI_PROGRESS_LOG.md` before upgrading status labels from "needs runtime verification".
12. Keep source/docs aligned as small systems are verified; do not start large new feature tracks yet.

## Progress Audit Note

`docs/POST_VERIFICATION_SUMMARY.md` is the newest consolidation after five
verification passes. Shell workspace polish, ChronoFS usability polish, app
platform polish, and learning progress map polish are now code-present and need
focused QEMU smoke passes before runtime claims.
`docs/ROADMAP_v0.3.md` recommends reliability and verification as the primary
v0.3 track, window/input stabilization as the secondary track, and
userspace/process metadata as an optional stretch track. ChronoFS
repair/recovery, userspace syscall/ELF execution, and UEFI loader failure work
remain focused technical tracks. `docs/USERSPACE_NEXT.md` defines the
conservative userspace/process staging path. Networking expansion, USB, dynamic
linking, package management, full compositing, and preemptive scheduling are
intentionally not the immediate next track.

## Repository Workflow

- Active integration branch: `main`.
- Public/product name: ChronoOS.
- Repo/package/image names may remain `chronosapien` until a dedicated rename task.
- Push future completed work to `origin/main` unless a task explicitly asks for a separate branch or pull request.

## Technical Gaps

- Interrupt-driven keyboard: implemented in code; IRQ1 buffering exists and polling fallback remains, but it needs runtime verification.
- Reusable heap allocator: implemented in code; the allocator is a simple free list, but reuse behavior needs build/runtime validation.
- ChronoFS repair/journaling: implemented in code; read-only `fs` inspection, clean `fsck`, clean journal status, read/write/read/delete, and delete persistence have narrow disposable-image QEMU evidence, but the new `files` usability namespace, `fsck repair`, recovery scenarios, independent write persistence, heap fallback, and disk-error behavior need runtime verification.
- Graphics shell: partially implemented; framebuffer console, mouse cursor, and small draggable windows exist, but there is no full desktop/compositor. Narrow QEMU evidence now covers window open/list/focus and serial-backed `windows close 2`, while visual close confirmation, manual drag, mouse close, `tasks`, and `kill` still need proof.
- Process model: partially implemented; `userspace status`, `userspace syscalls`, and the fixed `ring3` teaching demo have narrow QEMU evidence, while `userspace elf`, `syshello`, static ELF execution, and broader syscall behavior remain unverified. There is no dynamic linker, argv/env, or general multiprocess model. The next safe implementation step is read-only foreground ELF process metadata, not a process table.
- Scheduler: partially implemented; cooperative task scheduling exists, while preemption and production-grade scheduling are roadmap/design-only.
- UEFI boot: build/image path is fixed and single-core OVMF starts the ChronoOS loader, but the loader fails with `Out of Resources` before kernel framebuffer handoff or shell prompt.
- Networking: partially implemented; ARP/UDP over RTL8139 plus read-only
  observability commands exist in code, while DHCP, DNS, TCP, sockets, packet
  capture, and broad hardware support are roadmap/design-only.
- Real hardware/USB: partially implemented through the UEFI boot path; USB HID, USB storage, USB serial, and broad hardware support are roadmap/design-only.

## Recommended Verification Order

1. v0.3 educational workspace verification pass: verify `workspace`, `shortcuts`, `whereami`, `status`, `verify`, `help search`, `apps featured/category/info/demo`, `learn map/progress/beginner/advanced`, `museum index`, `quest badges/dependencies`, and `files list/info/search/sample/demo` in visible single-core BIOS QEMU.
2. Shell workspace smoke pass: verify `recent`, `theme`, `help search fs`, `help search app`, and typo suggestions for `hlep`, `apss`, `verfy`, and `lern` if they are not covered by the broader v0.3 workspace pass.
3. App platform smoke pass: verify `apps recent`, `apps launch calc`, `apps verified`, and `apps roadmap` if they are not covered by the broader v0.3 workspace pass.
4. ChronoFS usability smoke pass: on a fresh disposable image, verify `write demo.txt ChronoOS file demo`, `files copy demo.txt demo-copy.txt`, `cat demo-copy.txt`, refusing `files rename demo.txt renamed.txt`, `fs check`, and `journal`.
5. Input/window lifecycle pass: retest `windows close <id>`, `tasks`, `kill <observed-task-id>`, manual typing, Backspace, Shift, visible pointer movement, drag, and mouse close on a fresh disposable QEMU image; do not run `kill 0`.
6. Product-command input cleanup pass: retest only `poster system` and `capsule current` with reliable manual typing or a better input harness; do not upgrade unless serial shows exact command lines and screenshots show output.
7. ChronoFS repair/recovery pass: use only throwaway images for independent write persistence, `fsck repair` on safe synthetic damage, repair refusal cases, journal rollback/roll-forward, corrupt journal refusal, and heap fallback.
8. Userspace design follow-up: use `docs/USERSPACE_NEXT.md` to keep the next implementation limited to read-only foreground ELF process metadata.
9. Userspace verification follow-up: exact `userspace elf`, exact `syshello`, and controlled `exec hello.elf` only after `ld.lld` or an equivalent safe linker path is available and the known test ELF can be installed into a disposable image.
10. UEFI follow-up pass: investigate the loader `Out of Resources` failure before framebuffer handoff, then rerun single-core UEFI QEMU with serial/framebuffer evidence.
11. Risky systems last: custom BIOS path after NASM is available, SMP/AP startup, ARP/UDP networking via `net status`, `net config`, `net arp`, `net log`, and `net send`, and real hardware only after QEMU paths are cleaner.

## Product / Indie Features

- Implemented in code, needs runtime verification: `workspace`, `shortcuts`, `whereami`, `recent`, `status`, `verify`, `files`, `files list`, `files info`, `files search`, `files sample`, `files demo`, `files copy`, refusing `files rename`, `theme`, `demo`, `tour`, `capsule`, `doctor`, `poster`, `travel <year>`, `apps`, `apps featured`, `apps recent`, `apps category`, `apps help`, `apps demo`, `learn map`, `learn progress`, `learn beginner`, `learn advanced`, `explain <term>`, `museum index`, `quest dependencies`, and `quest badges`.
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
