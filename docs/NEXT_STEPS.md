# Next Steps

## Current Priority Queue

1. Use `docs/POST_VERIFICATION_SUMMARY.md` as the current product/engineering decision point after the visible BIOS, ChronoFS, window/input, userspace, and UEFI verification sequence.
2. Use `docs/VERIFICATION_MATRIX.md` as the compact evidence matrix and `docs/PROGRESS_AUDIT.md` as the broader progress audit.
3. Preserve build sanity: `cargo check -p kernel --offline --locked` and the host-target `chronosapien` check now pass with no warnings.
4. Preserve runtime tooling: QEMU 11.0.1 and PowerShell 7.6.2 are installed and available on PATH.
5. Treat the BIOS product path as the safest current demo base: single-core BIOS boot, serial logs, framebuffer shell, screenshots, `start`, `guide quick`, `mode status`, `safe on`, `doctor`, and `apps list` have QEMU evidence.
6. Make input/window lifecycle stabilization the primary product-risk track before broader UI polish: `windows close <id>`, `tasks`, `kill <observed-task-id>`, manual typing, Backspace, Shift, visible mouse movement, drag, and mouse close still need clean proof.
7. Shell workspace polish is implemented in code through `workspace`, `shortcuts`, `whereami`, `recent`, `status`, `verify`, `theme`, and `help search <term>`; the next safest verification step is a focused BIOS QEMU smoke pass for those commands.
8. ChronoFS usability polish is implemented in code through `files`, `files list`, `files info`, `files search`, `files sample`, `files demo`, non-overwriting `files copy`, and refusing `files rename`; verify it only on a disposable data image.
9. App platform polish is implemented in code through `apps featured`, `apps recent`, `apps category`, `apps help`, `apps demo`, richer metadata, verification badges, and risk labels; verify it with the same shell-first BIOS path before broader app polish.
10. Learning progress map polish is implemented in code through `learn map`, `learn progress`, `learn beginner`, `learn advanced`, updated `learn next`, `explain <term>`, `museum index`, `quest dependencies`, and `quest badges`; verify it with a focused shell-first BIOS path before demo claims.
11. Keep ChronoFS repair/recovery, userspace syscall/ELF follow-up, and UEFI `Out of Resources` as separate narrow technical tracks.
12. Record any successful QEMU or hardware evidence in `docs/AI_PROGRESS_LOG.md` before upgrading status labels from "needs runtime verification".
13. Keep source/docs aligned as small systems are verified; do not start large new feature tracks yet.

## Progress Audit Note

`docs/POST_VERIFICATION_SUMMARY.md` is the newest consolidation after five
verification passes. Shell workspace polish, ChronoFS usability polish, app
platform polish, and learning progress map polish are now code-present and need
focused QEMU smoke passes before runtime claims.
Input/window lifecycle stabilization remains the primary product-risk track.
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
- ChronoFS repair/journaling: implemented in code; read-only `fs` inspection, clean `fsck`, clean journal status, read/write/read/delete, and delete persistence have narrow disposable-image QEMU evidence, but the new `files` usability namespace, `fsck repair`, recovery scenarios, independent write persistence, heap fallback, and disk-error behavior need runtime verification.
- Graphics shell: partially implemented; framebuffer console, mouse cursor, and small draggable windows exist, but there is no full desktop/compositor. Narrow QEMU evidence now covers window open/list/focus and serial-backed `windows close 2`, while visual close confirmation, manual drag, mouse close, `tasks`, and `kill` still need proof.
- Process model: partially implemented; `userspace status`, `userspace syscalls`, and the fixed `ring3` teaching demo have narrow QEMU evidence, while `userspace elf`, `syshello`, static ELF execution, and broader syscall behavior remain unverified. There is no dynamic linker, argv/env, or general multiprocess model.
- Scheduler: partially implemented; cooperative task scheduling exists, while preemption and production-grade scheduling are roadmap/design-only.
- UEFI boot: build/image path is fixed and single-core OVMF starts the ChronoOS loader, but the loader fails with `Out of Resources` before kernel framebuffer handoff or shell prompt.
- Networking: partially implemented; ARP/UDP over RTL8139 plus read-only
  observability commands exist in code, while DHCP, DNS, TCP, sockets, packet
  capture, and broad hardware support are roadmap/design-only.
- Real hardware/USB: partially implemented through the UEFI boot path; USB HID, USB storage, USB serial, and broad hardware support are roadmap/design-only.

## Recommended Verification Order

1. Shell workspace smoke pass: verify `workspace`, `shortcuts`, `whereami`, `recent`, `status`, `verify`, `theme`, `help search fs`, `help search app`, and typo suggestions for `hlep`, `apss`, `verfy`, and `lern` in visible single-core BIOS QEMU.
2. App platform smoke pass: verify `apps featured`, `apps recent`, `apps category Core`, `apps category Files`, `apps category Learning`, `apps info notes`, `apps help notes`, `apps demo notes`, `apps launch calc`, `apps verified`, and `apps roadmap`.
3. Learning progress map smoke pass: verify `learn map`, `learn progress`, `learn beginner`, `learn advanced`, `learn next`, `explain kernel`, `explain filesystem`, `explain syscall`, `explain ARP`, `museum index`, `quest dependencies`, and `quest badges`.
4. ChronoFS usability smoke pass: on a fresh disposable image, verify `files`, `files sample`, `files list`, `write demo.txt ChronoOS file demo`, `files info demo.txt`, `files search demo`, `files search ChronoOS`, `files copy demo.txt demo-copy.txt`, `cat demo-copy.txt`, refusing `files rename demo.txt renamed.txt`, `fs check`, and `journal`.
5. Input/window lifecycle pass: retest `windows close <id>`, `tasks`, `kill <observed-task-id>`, manual typing, Backspace, Shift, visible pointer movement, drag, and mouse close on a fresh disposable QEMU image; do not run `kill 0`.
6. Product-command input cleanup pass: retest only `poster system` and `capsule current` with reliable manual typing or a better input harness; do not upgrade unless serial shows exact command lines and screenshots show output.
7. ChronoFS repair/recovery pass: use only throwaway images for independent write persistence, `fsck repair` on safe synthetic damage, repair refusal cases, journal rollback/roll-forward, corrupt journal refusal, and heap fallback.
8. Userspace follow-up pass: exact `userspace elf`, exact `syshello`, and controlled `exec hello.elf` only after `ld.lld` or an equivalent safe linker path is available and the known test ELF can be installed into a disposable image.
9. UEFI follow-up pass: investigate the loader `Out of Resources` failure before framebuffer handoff, then rerun single-core UEFI QEMU with serial/framebuffer evidence.
10. Risky systems last: custom BIOS path after NASM is available, SMP/AP startup, ARP/UDP networking via `net status`, `net config`, `net arp`, `net log`, and `net send`, and real hardware only after QEMU paths are cleaner.

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
