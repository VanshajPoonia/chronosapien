# AI Progress Log

## Purpose

This file tracks every major Codex/AI-assisted change to ChronoOS.

## Current Source of Truth

- Public OS identity: ChronoOS.
- Repo/package name: `chronosapien`; older internal/source text may still mention Chronosapian.
- Current development phase: post-Phase-4 stabilization, documentation, and verification planning before new feature work.
- Current source-truth handoff: `docs/CURRENT_STATUS.md`.
- Verified in QEMU with limits: single-core BIOS boot reaches `[CHRONO] boot complete`; boot-time serial logging is observed; visible QEMU screendumps show the framebuffer shell prompt; QEMU monitor input submitted `help` and `help start`; `about`, `apps`, the `notes` home screen, and `calc 6 - 7` output were observed in the framebuffer; `open notes` partially reached the window/task path; one mouse click packet was logged; RTL8139 initialization and MAC discovery were observed.
- Implemented in code but still needs broader or first runtime verification: optional custom BIOS bootloader, UEFI loader, broader framebuffer redraw behavior, manual keyboard typing, Backspace, shifted input, keyboard polling fallback, cursor movement, window drag/close, `open sysinfo`, timer behavior from shell commands, GDT/IDT/PIC behavior beyond boot, reusable heap allocator reuse, ChronoFS shell workflows, ATA storage workflows, ChronoFS `fsck`, ChronoFS journal/recovery, ARP/UDP behavior, ring 3 demo, syscalls, ELF execution, SMP/AP startup beyond BSP, scheduler task lifecycle, museum, quests, most product commands, GIF capture, and hardware.
- Partial/missing systems: richer graphics shell, fuller process model, preemptive scheduler, broader networking, real hardware/USB support, and runtime verification of current implemented-in-code systems.
- Current priority: preserve context, stabilize the repo, verify existing systems, and avoid new OS features until build/runtime status is known.

## Log Entries

Each future entry should use this format:

### YYYY-MM-DD — Short task title
- Prompt/task:
- Files changed:
- What changed:
- What was intentionally avoided:
- Runtime verified: yes/no
- If not verified, what still needs verification:
- New risks introduced:
- Next recommended step:

### 2026-06-03 — Add networking observability before protocol expansion
- Prompt/task: Improve observability around the existing RTL8139/static IPv4/ARP/UDP stack before adding DHCP, DNS, TCP, sockets, or broader networking features.
- Files changed: `kernel/src/net.rs`, `kernel/src/shell.rs`, `README.md`, `docs/networking.md`, `docs/shell-commands.md`, `docs/manual-testing.md`, `docs/demo-script.md`, `docs/VERIFICATION_MATRIX.md`, `docs/CURRENT_STATUS.md`, `docs/KNOWN_LIMITATIONS.md`, `docs/ROADMAP_AFTER_v0.1.md`, `docs/roadmap.md`, `docs/NEXT_STEPS.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added real no-alloc networking counters for ARP requests, ARP replies, UDP sent, UDP received, malformed RX packets, last event, and last error; expanded `net` into `net status`, `net config`, `net arp`, `net udp`, `net send`, `net log`, `net demo`, `net roadmap`, and `net help`; updated `help network`; and documented that `net log` is a counter view, not packet capture.
- What was intentionally avoided: No DHCP, DNS, TCP, sockets, packet capture, package model, dynamic networking API, networking architecture rewrite, QEMU run, hardware test, or runtime verification upgrade.
- Runtime verified: no. This pass used source/build/documentation checks only unless separately recorded below.
- If not verified, what still needs verification: `net status`, `net config`, `net arp`, `net udp`, `net send`, `net log`, `net demo`, `net roadmap`, ARP reply learning, UDP guest-to-host send, UDP host-to-guest receive, malformed packet counters, serial logs, screenshots, and hardware.
- New risks introduced: Low; counters are tied to existing code paths, but they still need runtime checks to confirm the observed values match QEMU packet behavior.
- Next recommended step: Run a controlled single-core BIOS QEMU networking pass using reliable shell input, a non-conflicting UDP listener setup, `net status`, `net arp`, `net log`, `net send`, and recorded serial/host UDP evidence.

### 2026-06-03 — Polish window and app lifecycle surface
- Prompt/task: Add small shell-visible window lifecycle commands and document the tiny educational window layer without building a full compositor or GUI toolkit.
- Files changed: `kernel/src/wm.rs`, `kernel/src/shell.rs`, `README.md`, `docs/windowing.md`, `docs/shell-commands.md`, `docs/manual-testing.md`, `docs/demo-script.md`, `docs/VERIFICATION_MATRIX.md`, `docs/CURRENT_STATUS.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added stable shell-facing window IDs; added read-only window inspection helpers plus focus/close-by-ID helpers; added `windows`, `windows list`, `windows status`, `windows focus <id>`, `windows close <id>`, and `windows help`; improved `open notes`/`open sysinfo` failure messages; made `open paint` report roadmap/design-only; and documented window lifecycle behavior and limits.
- What was intentionally avoided: No full compositor, GUI toolkit, animations, GPU acceleration, framebuffer renderer rewrite, complex event-loop rewrite, risky multitasking expansion, QEMU run, hardware test, or runtime verification upgrade.
- Runtime verified: no. This pass used source/build/documentation checks only unless separately recorded below.
- If not verified, what still needs verification: `windows list`, `windows status`, `windows focus <id>`, `windows close <id>`, `open sysinfo`, `open paint` messaging, mouse cursor movement, drag, close button behavior, focus order, task/window cleanup, screenshots, and hardware.
- New risks introduced: Low to moderate; `windows close <id>` now exposes existing close behavior from the shell and may terminate the owning cooperative task, matching the close-button behavior.
- Next recommended step: Run a controlled visible QEMU window pass for `open notes`, `windows list`, `open sysinfo`, `windows focus <id>`, `windows close <id>`, `tasks`, and mouse drag/close.

### 2026-06-03 — Add static app registry foundation
- Prompt/task: Create a lightweight app manifest and app registry foundation for ChronoOS without adding a package manager, dynamic linker, or dynamic app loading.
- Files changed: `kernel/src/apps/mod.rs`, `kernel/src/shell.rs`, `README.md`, `docs/apps.md`, `docs/shell-commands.md`, `docs/demo-script.md`, `docs/manual-testing.md`, `docs/VERIFICATION_MATRIX.md`, `docs/CURRENT_STATUS.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added static app metadata for notes, calc, sysinfo, files, museum, theme, tasks, paint, network, userspace, timeline, crashlab, and doctor; added `apps list`, `apps info <name>`, `apps launch <name>`, `apps verified`, and `apps roadmap`; preserved existing direct app aliases; and documented app status, verification, risk, and future app-loading boundaries.
- What was intentionally avoided: No package manager, dynamic linker, dynamic app loading, app architecture rewrite, new runtime apps, new GUI system, QEMU run, hardware test, or runtime verification upgrade.
- Runtime verified: no. This pass used source/build/documentation checks only unless separately recorded below.
- If not verified, what still needs verification: `apps list`, `apps info notes`, `apps launch calc`, `apps verified`, `apps roadmap`, direct app aliases, roadmap app refusal behavior, `sysinfo`, notes read/write persistence, app/window launch paths, and screenshots.
- New risks introduced: Low; the registry is static metadata and `apps launch` delegates only to existing shell commands for implemented or partial entries.
- Next recommended step: Run a controlled single-core BIOS QEMU app-registry pass for `apps`, `apps list`, `apps info notes`, `apps launch calc`, `apps verified`, `apps roadmap`, and legacy aliases.

### 2026-06-03 — Clarify user-space and process foundation
- Prompt/task: Strengthen the user-space/process foundation by clarifying Ring 3, syscalls, static ELF execution, and future process-model boundaries without implementing a full process model.
- Files changed: `kernel/src/shell.rs`, `README.md`, `docs/userspace-model.md`, `docs/ring3.md`, `docs/syscalls.md`, `docs/elf.md`, `docs/manual-testing.md`, `docs/VERIFICATION_MATRIX.md`, `docs/shell-commands.md`, `docs/CURRENT_STATUS.md`, `docs/NEXT_STEPS.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added a read-only `userspace` namespace with `userspace status`, `userspace syscalls`, `userspace elf`, `userspace roadmap`, and `userspace help`; kept `ring3`, `syshello`, and `exec <name>` behavior unchanged while making their warning point to the current boundary; added `docs/userspace-model.md`; and updated syscall/ELF/Ring 3 docs with the current teaching-path limits.
- What was intentionally avoided: No full process model, preemptive scheduler, dynamic linker, package manager, argv/env, fork/exec semantics, file descriptor table, scheduler rewrite, QEMU run, hardware test, or runtime verification upgrade.
- Runtime verified: no. This pass used source/build/documentation checks only unless separately recorded below.
- If not verified, what still needs verification: `userspace status`, `userspace syscalls`, `userspace elf`, `userspace roadmap`, `ring3`, `syshello`, `exec hello.elf`, invalid ELF handling, syscall return/error behavior, foreground ELF return-to-shell behavior, and scheduler interactions during user-space tests.
- New risks introduced: Low; the new `userspace` commands are read-only status/documentation surfaces and do not alter process, syscall, ELF, or scheduler behavior.
- Next recommended step: Run a controlled single-core BIOS QEMU user-space verification pass after installing `hello.elf` with `scripts/build-user.ps1`.

### 2026-06-03 — Harden ChronoFS inspection and diagnostics
- Prompt/task: Add conservative ChronoFS hardening focused on inspection commands, clearer fsck/journal reporting, safer repair wording, and updated verification docs.
- Files changed: `kernel/src/fs.rs`, `kernel/src/shell.rs`, `README.md`, `docs/chronofs-hardening.md`, `docs/storage.md`, `docs/manual-testing.md`, `docs/shell-commands.md`, `docs/VERIFICATION_MATRIX.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added a read-only `fs` namespace with `fs status`, `fs info`, `fs check`, `fs journal`, and `fs help`; added an `FsStatus` source summary; improved fsck output so checked, suspicious, repaired, and not-repaired information is clearer; strengthened `fsck repair` warning text; clarified journal status wording; and documented current ChronoFS design, risks, repair boundaries, and verification path.
- What was intentionally avoided: No disk-format change, directories, permissions, large-file support, complex journaling, POSIX behavior, filesystem rewrite, QEMU run, hardware test, or runtime verification upgrade.
- Runtime verified: no. This pass used source/build/documentation checks only unless separately recorded below.
- If not verified, what still needs verification: ChronoFS shell workflows (`fs status`, `fs info`, `ls`, `write`, `cat`, `rm`, `fs check`, `fs journal`, `fsck`, `fsck repair`, `journal`), persistence after reboot, controlled repair behavior, journal recovery states, heap fallback behavior, and cache/disk behavior after disk errors.
- New risks introduced: Low; the new `fs` commands are read-only. The new status output exposes more internal counts but does not mutate filesystem metadata.
- Next recommended step: Run a controlled single-core BIOS QEMU filesystem verification pass with a disposable data disk and record serial/screenshot evidence before upgrading ChronoFS rows.

### 2026-06-02 — Add final verification evidence matrix
- Prompt/task: Create one clear source of truth showing what actually works, what was tested, what is code-present only, and what remains unverified.
- Files changed: `docs/VERIFICATION_MATRIX.md`, `README.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added a compact evidence matrix covering boot paths, framebuffer/serial/input/UI, apps, storage, heap, scheduler/SMP, userspace, networking, product surfaces, screenshots/GIFs, roadmap-only ideas, and hardware; linked the matrix from the README; and recorded this documentation-only consolidation pass.
- What was intentionally avoided: No new features, QEMU run, hardware test, screenshots, GIF capture, kernel/source-code edits, runtime status upgrade, TCP, DHCP, DNS, USB, dynamic linker, package manager, full compositor, or preemptive scheduler.
- Runtime verified: no new runtime verification. The matrix only consolidates evidence already recorded in `docs/CURRENT_STATUS.md`, `docs/release-checklist.md`, `docs/screenshots.md`, and prior progress-log entries.
- If not verified, what still needs verification: UEFI boot, custom BIOS boot, ChronoFS workflows, fsck/journal behavior, heap reuse, scheduler lifecycle, AP startup, Ring 3, syscalls, static ELF execution, ARP/UDP behavior, full onboarding/status/product command flows, GIF capture, and hardware.
- New risks introduced: Low; documentation-only evidence consolidation.
- Next recommended step: Fix the UEFI loader API mismatch or install NASM for a custom BIOS build pass, then verify one risky path at a time.

### 2026-06-02 — Slice ChronoOS progress into 10 commits
- Prompt/task: Slice the current ChronoOS progress into exactly 10 intentional commits and push to GitHub.
- Files changed: `docs/AI_PROGRESS_LOG.md`.
- What changed: Recorded the commit-slicing and publish pass after the source, release documentation, current-status, testing, screenshot, release-checklist, and verification-log changes were grouped into logical commits.
- Commands run: `git status -sb --untracked-files=all`; `git diff --stat`; `cargo check -p kernel --offline --locked`; `cargo check -p chronosapien --target aarch64-apple-darwin --offline --locked`; `git diff --check`; explicit `git add` commands per slice; `git commit` for each slice.
- What was intentionally avoided: No source edits beyond the already-staged ChronoOS progress, no force push, no rebase, no reset, no PR creation, no runtime status upgrade, and no new QEMU/hardware verification.
- Runtime verified: no new runtime verification. This pass used build and diff checks only.
- If not verified, what still needs verification: UEFI boot, custom BIOS boot, AP startup, ARP/UDP behavior beyond RTL8139 init, hardware boot, and the remaining manual UI/input/filesystem/userspace flows listed in `docs/CURRENT_STATUS.md`.
- New risks introduced: Low; this pass is git organization and publishing of existing work.
- Next recommended step: After push, continue with a narrow UEFI loader build-compatibility fix or a NASM-enabled custom BIOS build pass, not feature expansion.

### 2026-06-02 — Run high-risk UEFI custom BIOS SMP networking audit
- Prompt/task: Carefully verify or block the higher-risk UEFI, custom BIOS, SMP/AP, networking, and hardware areas without adding features or rewriting low-level architecture.
- Files changed: `docs/CURRENT_STATUS.md`, `docs/manual-testing.md`, `docs/hardware-testing.md`, `docs/networking.md`, `docs/boot-flow.md`, `docs/release-checklist.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Recorded the high-risk preflight and runtime outcomes; added a hardware testing guide; marked UEFI as blocked by a loader build failure; marked custom BIOS as blocked by missing `nasm`; kept SMP/AP high-risk after a two-core serial smoke showed BSP only; marked networking partially verified only for RTL8139 init/MAC; and kept hardware as manual verification only.
- Commands run: `command -v pwsh`; `command -v qemu-system-x86_64`; `command -v rustup`; `test -f /opt/homebrew/share/qemu/edk2-x86_64-code.fd`; `command -v nasm`; `command -v nc`; `cargo check -p kernel --offline --locked`; `cargo check -p chronosapien --target aarch64-apple-darwin --offline --locked`; `pwsh -NoLogo -NoProfile -File scripts/build-uefi.ps1`; two-core serial-only `qemu-system-x86_64 -smp 2`; single-core RTL8139 QEMU networking smoke; `nc -ul 9000`.
- Evidence captured: OVMF exists at `/opt/homebrew/share/qemu/edk2-x86_64-code.fd`; both cargo checks passed; UEFI build first failed under sandboxed network, then after escalation downloaded dependencies but failed compiling `uefi-loader` because `uefi::boot::MemoryMap` was unresolved; `nasm` was not on PATH; SMP serial log `/private/tmp/chronoos-smp-20260602-162000.serial.log` reached `smp: BSP online (core 0)` and `active era: 1984` only; networking serial log `/private/tmp/chronoos-net-20260602-162000.serial.log` reached boot complete and logged RTL8139 discovery plus MAC `52:54:00:12:34:56`; host UDP log `/private/tmp/chronoos-net-20260602-162000.host-udp.log` was 0 bytes.
- What was intentionally avoided: No source changes, UEFI loader patch, NASM install, custom BIOS build/run without NASM, bootloader rewrite, SMP rewrite, networking protocol expansion, TCP, DHCP, DNS, USB, dynamic linker, package manager, full compositor, preemptive scheduler, packet-success claim, hardware image write, or hardware boot.
- Runtime verified: yes, limited. Partially verified in QEMU: two-core BIOS smoke reached BSP startup only; RTL8139 device init and MAC read worked in single-core QEMU. Not verified: UEFI boot, custom BIOS boot, AP startup, ARP, UDP send/receive, host-to-guest forwarding, and hardware.
- If not verified, what still needs verification: UEFI loader build compatibility, UEFI QEMU boot, custom BIOS build after NASM is available, custom BIOS QEMU boot, AP startup evidence, clean `net` shell command input, ARP reply, UDP transmit/receive, hardware boot, serial logging on hardware, and any USB-related behavior.
- New risks introduced: None expected; this pass changed verification docs only. The run exposed real blockers: UEFI loader API drift, missing NASM, unreliable QEMU monitor shell input for networking commands, and UDP port conflict between `hostfwd=udp::9000-:9000` and a local listener.
- Next recommended step: Fix the UEFI loader API mismatch as a narrow build-compatibility task, or install NASM for a custom BIOS build-only pass; keep SMP/AP and ARP/UDP runtime verification separate and one-command-at-a-time.

### 2026-06-02 — Run UI input app and mouse QEMU verification pass
- Prompt/task: Verify the still-unverified UI/input areas without adding features: visible framebuffer shell, keyboard behavior, mouse/window behavior, app UI paths, and screenshot/GIF evidence.
- Files changed: `docs/CURRENT_STATUS.md`, `docs/release-checklist.md`, `docs/screenshots.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Recorded a second visible single-core BIOS QEMU pass focused on apps, partial window behavior, mouse click evidence, and screenshot capture; upgraded only the observed surfaces; and kept Backspace, shifted input, polling fallback, `sysinfo`, `open sysinfo`, drag, close, GIF capture, and hardware unverified.
- Commands run: `command -v pwsh`; `command -v qemu-system-x86_64`; `command -v nc`; `command -v sips`; `command -v ffmpeg`; `command -v magick`; `command -v gifsicle`; `cargo check -p kernel --offline --locked`; `cargo check -p chronosapien --target aarch64-apple-darwin --offline --locked`; `pwsh -NoLogo -NoProfile -File scripts/build.ps1`; direct `qemu-system-x86_64 -smp 1` with the BIOS and ChronoFS data images, RTL8139, visible display, serial log, and QEMU monitor socket.
- Evidence captured: serial log `/private/tmp/chronoos-ui-input-20260602-150049.serial.log`; screenshots `/private/tmp/chronoos-ui-input-20260602-150049-boot.png`, `/private/tmp/chronoos-ui-input-20260602-150049-apps.png`, `/private/tmp/chronoos-ui-input-20260602-150049-notes-attempt.png`, `/private/tmp/chronoos-ui-input-20260602-150049-calc.png`, `/private/tmp/chronoos-ui-input-20260602-150049-open-notes-window.png`, `/private/tmp/chronoos-ui-input-20260602-150049-mouse-move.png`, and `/private/tmp/chronoos-ui-input-20260602-150049-drag-attempt.png`. Serial evidence includes `[CHRONO] boot complete`, `cmd: apps`, `cmd: notes`, `cmd: calc 6 - 7`, `app: calc launched`, `cmd: open notes`, `wm: open notes`, and `mouse: click at 740,410`.
- What was intentionally avoided: No source features, TCP, DHCP, DNS, USB, dynamic linker, package manager, full compositor, preemptive scheduler, window-manager rewrite, input-stack rewrite, networking packet send, UEFI test, custom BIOS test, SMP test, hardware test, or broad runtime claim.
- Runtime verified: yes, limited. Verified in QEMU: framebuffer shell remained visible, QEMU `sendkey` submitted several shell commands, `apps` launcher was visible, `notes` home screen was visible, `calc 6 - 7` returned `-1`, QEMU `screendump` produced PNG screenshots, and a mouse click packet was logged. Partially verified in QEMU: `open notes` spawned a task, logged `wm: open notes`, and showed a visible window boundary.
- If not verified, what still needs verification: Backspace, shifted symbols, polling fallback, reliable manual typing, `sysinfo`, notes read/write, `open sysinfo`, cursor movement, window focus, drag, close, app window workflows beyond `open notes`, GIF capture, screenshots for broader demo flows, and hardware.
- New risks introduced: None expected; this pass changed verification docs only. The run reinforced that QEMU monitor key injection can duplicate or garble first characters, as seen with `abouut`, `ssysinfo`, and an early `oopen notes` attempt.
- Next recommended step: Use a more reliable interactive input method or slower one-command-at-a-time QEMU pass for Backspace/Shift, `sysinfo`, `open sysinfo`, notes read/write, ChronoFS basics, and window drag/close before upgrading more labels.

### 2026-06-02 — Run core single-core BIOS QEMU verification pass
- Prompt/task: Verify the safest ChronoOS runtime path first: normal BIOS image, single-core QEMU, visible framebuffer, serial logging, shell startup, keyboard input, basic commands, and conservative evidence recording.
- Files changed: `docs/CURRENT_STATUS.md`, `docs/release-checklist.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Recorded actual QEMU evidence from the 2026-06-02 visible BIOS pass; upgraded only the observed systems in the current-status handoff; added a dated release-checklist evidence table; and kept unobserved apps, ChronoFS workflows, userspace, networking behavior, mouse/window interaction, UEFI, custom BIOS, SMP/AP, GIFs, and hardware unverified.
- Commands run: `command -v pwsh`; `pwsh -NoLogo -NoProfile -Command '$PSVersionTable.PSVersion.ToString()'`; `command -v qemu-system-x86_64`; `qemu-system-x86_64 --version`; `cargo check -p kernel --offline --locked`; `cargo check -p chronosapien --target aarch64-apple-darwin --offline --locked`; `pwsh -NoLogo -NoProfile -File scripts/build.ps1`; direct `qemu-system-x86_64 -smp 1` with the BIOS and ChronoFS data images, RTL8139, serial log, visible display, and QEMU monitor socket.
- Evidence captured: PowerShell 7.6.2; QEMU 11.0.1; both cargo checks passed; BIOS image build passed; serial log `/private/tmp/chronoos-qemu-20260602-013807.serial.log` includes `[CHRONO] boot start`, framebuffer initialization, filesystem mount, timer, mouse, RTL8139 initialization, keyboard initialization, and `[CHRONO] boot complete`; QEMU screendumps `/private/tmp/chronoos-qemu-20260602-013807-screendump.png`, `/private/tmp/chronoos-qemu-20260602-013807-help-after.png`, `/private/tmp/chronoos-qemu-20260602-013807-basic-product.png`, and `/private/tmp/chronoos-qemu-20260602-013807-about.png` showed the ChronoOS top bar, `CHRONO>` prompt, grouped `help`, `help start`, and `about` output.
- What was intentionally avoided: No new features, source-code changes, TCP, DHCP, DNS, USB, dynamic linker, package manager, full compositor, preemptive scheduler, UEFI test, custom BIOS test, SMP retry, filesystem mutation beyond existing boot disk mount, userspace execution, packet sending, hardware test, or broad runtime claim.
- Runtime verified: yes, limited. Verified in QEMU: normal single-core BIOS boot, boot-time serial output, visible framebuffer shell prompt, QEMU screendump capture, and narrow keyboard/shell input for `help` and `help start`; `about` output was observed on the framebuffer.
- If not verified, what still needs verification: Apps, ChronoFS shell workflows (`ls`, `write`, `cat`, `rm`, `fsck`, `fsck repair`, `journal`), product command flows (`demo`, `tour`, `doctor`, `capsule`, `poster`), top-level `status`/`verify`/`timeline` command expectations, manual keyboard typing, Backspace, shifted keys, polling fallback, mouse/window interaction, userspace/syscalls/ELF (`ring3`, `syshello`, `exec`), ARP/UDP networking behavior, UEFI boot, custom BIOS boot, SMP/AP startup, GIF capture, and hardware.
- New risks introduced: None expected; this pass changed verification docs only. The run exposed that QEMU monitor batch key injection can garble longer command sequences, so future shell verification should use slower one-command-at-a-time input or a more reliable interactive method.
- Next recommended step: Run a second visible BIOS QEMU pass focused only on one-command-at-a-time shell workflows: `uptime`, `mem`, ChronoFS basics, `fsck`, `journal`, `apps`, `notes`, `calc`, `sysinfo`, and product/status commands.

### 2026-06-02 — Prepare v0.1 release candidate documentation package
- Prompt/task: Create a clean, honest, portfolio-ready ChronoOS v0.1 release candidate package without adding new kernel features or upgrading runtime verification claims.
- Files changed: `docs/RELEASE_v0.1.md`, `docs/KNOWN_LIMITATIONS.md`, `docs/ROADMAP_AFTER_v0.1.md`, `README.md`, `docs/showcase.md`, `docs/demo-script.md`, `docs/screenshots.md`, `docs/release-checklist.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added the v0.1 RC release note for "ChronoOS v0.1 RC - Time-Museum Shell"; added standalone known limitations; added a post-v0.1 roadmap grouped by verification, filesystem, userspace, networking, UI, real hardware/USB, and long-term systems; updated README/showcase/demo/screenshots/release checklist links and v0.1 capture guidance.
- What was intentionally avoided: No new kernel features, TCP, DHCP, DNS, USB, dynamic linker, package manager, full compositor, preemptive scheduler, QEMU run, hardware test, runtime status upgrade, or project/package rename.
- Runtime verified: no new runtime verification. This pass is documentation and release packaging only.
- If not verified, what still needs verification: Visible framebuffer output, shell prompt, keyboard input, `help`, onboarding, product commands, apps, filesystem workflows, userspace/syscalls/ELF, networking, mouse/windows, UEFI, custom BIOS, SMP/AP, screenshots/GIFs, and hardware.
- New risks introduced: Low; documentation-only release packaging that makes current limits more explicit.
- Next recommended step: Run the visible BIOS QEMU verification pass for framebuffer, shell prompt, keyboard input, `help`, onboarding, ChronoFS basics, apps, product commands, and screenshots before tagging or publishing v0.1 media.

### 2026-06-02 — Polish shell command help and UX
- Prompt/task: Make ChronoOS commands easier to navigate by grouping help output, adding topic-specific help pages, clarifying overlapping product concepts, improving unknown-command guidance, and warning before risky/unverified command paths.
- Files changed: `kernel/src/shell.rs`, `kernel/src/net.rs`, `README.md`, `docs/shell-commands.md`, `docs/demo-script.md`, `docs/manual-testing.md`, `docs/CURRENT_STATUS.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Replaced the flat help list with category-based help; added `help start`, `help apps`, `help fs`, `help system`, `help network`, `help userspace`, `help labs`, and `help roadmap` plus beginner-friendly help-topic aliases; added helpful unknown-command hints; added warnings before userspace demo commands, `fsck repair`, and ARP/UDP send paths; updated docs and testing checklists to match.
- What was intentionally avoided: No risky kernel features, TCP, DHCP, DNS, USB, dynamic linker, package manager, full compositor, preemptive scheduler, new status/verify command alias, subsystem rewrite, QEMU run, hardware test, or runtime verification claim.
- Runtime verified: no. This pass only updates code/docs and should be treated as needing runtime verification until visible shell evidence is recorded.
- If not verified, what still needs verification: Categorized `help`, every `help <topic>` page, unknown-command hints, risky-command warning output, visible framebuffer output, shell interaction, apps, filesystem workflows, userspace/syscalls/ELF, networking, mouse/windows, UEFI, custom BIOS, SMP/AP, and hardware.
- New risks introduced: Low; behavior changes are shell text routing and warning lines before existing commands.
- Next recommended step: Run a visible BIOS QEMU shell pass for `help`, all help topics, `start`, `guide quick`, `doctor`, `fsck`, `journal`, and warning output before publishing a demo recording.

### 2026-06-02 — Add guided first-run shell experience
- Prompt/task: Add a shell-first guided onboarding flow that makes ChronoOS welcoming and demo-ready without adding risky kernel features or duplicating the existing demo/tour/capsule/status surfaces.
- Files changed: `kernel/src/shell.rs`, `README.md`, `docs/shell-commands.md`, `docs/demo-script.md`, `docs/manual-testing.md`, `docs/CURRENT_STATUS.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added `start` and `welcome` as first-run welcome aliases; added `guide`, `guide quick`, `guide full`, `guide eras`, `guide apps`, `guide systems`, `guide status`, and `guide next`; updated help and docs so the new flow routes users toward existing commands such as `demo`, `tour`, `capsule`, `doctor`, `poster`, `apps`, museum pages, quests, `fsck`, and `journal`.
- What was intentionally avoided: No TCP, DHCP, DNS, USB, dynamic linker, package manager, full compositor, preemptive scheduler, persistence flag, automatic first-run state, QEMU run, hardware test, or runtime verification claim.
- Runtime verified: no. This was build-checked only unless a later entry records QEMU or hardware evidence.
- If not verified, what still needs verification: `start`, `welcome`, every `guide` topic, visible framebuffer output, shell interaction, apps, filesystem workflows, userspace/syscalls/ELF, networking, mouse/windows, UEFI, custom BIOS, SMP/AP, and hardware.
- New risks introduced: Low; the new commands are read-only text surfaces inside the existing shell command model.
- Next recommended step: Run a visible BIOS QEMU pass for `start`, `guide quick`, `guide full`, `guide status`, and the 2-minute demo path before publishing screenshots.

### 2026-06-01 — Add demo and release documentation package
- Prompt/task: Create a ChronoOS demo/release package for portfolio and build-in-public use without adding risky kernel features or upgrading verification claims.
- Files changed: `docs/demo-script.md`, `docs/screenshots.md`, `docs/release-checklist.md`, `README.md`, `docs/showcase.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added 2-minute, 5-minute, and 10-minute demo paths; documented commands for identity, eras, museum/quest, apps, filesystem, fsck/journal, userspace/syscalls/ELF, and networking; added screenshot/GIF capture checklists with naming conventions and status tags; added a release checklist covering build, QEMU boot, serial, keyboard, mouse/window, filesystem, apps, demo commands, docs, README, and known limitations; improved the README portfolio narrative and known-limitations section; and linked the showcase to the new demo/release package docs.
- What was intentionally avoided: No kernel behavior changes, risky OS features, QEMU run, hardware test, TCP, DHCP, DNS, USB, dynamic linker, package manager, full compositor, preemptive scheduler, broad runtime claim, or project/package rename.
- Runtime verified: no new runtime verification. Existing evidence remains limited to single-core BIOS serial-only QEMU reaching `[CHRONO] boot complete` and boot-time serial logging through that point.
- If not verified, what still needs verification: visual framebuffer, shell interaction, apps, filesystem workflows, userspace/syscalls/ELF, networking, mouse/windows, UEFI, custom BIOS, SMP/AP, and hardware.
- New risks introduced: Low; this is documentation-only and intentionally conservative.
- Next recommended step: Run the visible BIOS QEMU demo path, capture screenshots/logs with `docs/screenshots.md`, and update `docs/AI_PROGRESS_LOG.md` before publishing portfolio claims.

### 2026-06-01 — Add post-Phase-4 current status audit
- Prompt/task: Create a post-Phase-4 reality audit before adding new features, align high-level docs to source truth, and keep runtime claims limited to existing evidence.
- Files changed: `docs/CURRENT_STATUS.md`, `README.md`, `docs/roadmap.md`, `docs/NEXT_STEPS.md`, `docs/showcase.md`, `docs/manual-testing.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added `docs/CURRENT_STATUS.md` as the compact current-state handoff; documented project identity, boot paths, framebuffer/serial/UI, shell, apps, museum/quest/product layer, Phase 3 product ideas, ChronoFS, input/windows, heap, scheduler/SMP, Ring 3/syscalls/ELF, networking, and long-term systems 96-110; linked README/showcase/NEXT_STEPS to the new handoff; restored a clear Phase 4 roadmap section with networking, USB/real hardware, scheduler/process/user mode, app loading/package model, and GUI/compositor tracks; and expanded manual-testing boundary checks for Phase 3 and Phase 4.
- What was intentionally avoided: No source behavior changes, new features, DHCP, DNS, TCP, USB, dynamic linker, package manager, full compositor, preemptive scheduler, QEMU run, hardware test, or project/package rename.
- Runtime verified: no new runtime verification in this audit. Existing evidence remains limited to build checks with no warnings and single-core BIOS serial-only QEMU reaching `[CHRONO] boot complete`.
- If not verified, what still needs verification: Visible framebuffer output, shell prompt, keyboard input, PIT/timer shell behavior, filesystem shell commands, product/demo commands, apps, mouse/window interaction, heap reuse, ChronoFS fsck/repair/journal recovery, UEFI boot, custom BIOS boot, ring 3, syscalls, static ELF execution, ARP/UDP networking, SMP/AP startup, and hardware boot.
- New risks introduced: Low; changes are documentation-only and intentionally conservative.
- Next recommended step: Run a visible BIOS QEMU pass and record framebuffer, shell, keyboard, ChronoFS, app, and product-command evidence before adding features or upgrading status labels.

### 2026-06-01 — Install runtime tooling and clean warning baseline
- Prompt/task: Implement the runtime setup, commit prep, and warning-cleanup plan while preserving honest runtime verification labels.
- Files changed: `kernel/src/framebuffer/mod.rs`, `kernel/src/fs.rs`, `kernel/src/gdt.rs`, `kernel/src/interrupts.rs`, `kernel/src/memory.rs`, `kernel/src/net.rs`, `kernel/src/pci.rs`, `kernel/src/sched.rs`, `kernel/src/shell.rs`, `kernel/src/smp.rs`, `kernel/src/syscall.rs`, `docs/status-audit.md`, `docs/manual-testing.md`, `docs/NEXT_STEPS.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Installed QEMU and PowerShell with Homebrew; verified QEMU 11.0.1 and PowerShell 7.6.2; reduced the kernel and host-target build warning baseline to zero warnings using mechanical cleanup and narrow `#[allow(dead_code)]` annotations for intentional teaching scaffolding; built the BIOS image with `pwsh -NoProfile -File scripts/build.ps1`; created the expected 16 MiB ChronoFS data image; and ran serial-only BIOS QEMU smoke checks.
- What was intentionally avoided: No DHCP, DNS, TCP, USB, dynamic linker, package manager, full compositor, preemptive scheduler, broad SMP/scheduler/userspace rewrite, visual framebuffer claim, shell-interaction claim, app/filesystem-command claim, or hardware claim.
- Runtime verified: yes, limited. Single-core BIOS serial-only QEMU reached `[CHRONO] boot complete`, and boot-time serial output was observed. A two-core BIOS serial-only QEMU run exited before `[CHRONO] boot complete`, after `[CHRONO] active era: 1984`.
- If not verified, what still needs verification: Visible framebuffer output, shell prompt, keyboard input, PIT/timer behavior from shell commands, filesystem shell commands, apps/product commands, mouse/window interaction, heap reuse, ChronoFS fsck/repair/journal recovery, UEFI boot, custom BIOS boot, ring 3, syscalls, static ELF execution, ARP/UDP networking, SMP/AP startup, and hardware boot remain unverified.
- New risks introduced: Low; warning cleanup was mechanical and kept educational scaffolding. Runtime evidence exposed an existing SMP/AP startup risk in the two-core BIOS smoke.
- Next recommended step: Run a visual BIOS QEMU pass for framebuffer, shell prompt, keyboard, and ChronoFS basics, then investigate the two-core SMP/AP boot exit separately.

### 2026-06-01 — Restore build sanity and audit local runtime tooling
- Prompt/task: Implement the combined build-sanity, runtime-audit, and docs-polish plan without adding large features or claiming runtime verification.
- Files changed: `Cargo.lock`, `kernel/src/gdt.rs`, `kernel/src/process.rs`, `kernel/src/ring3.rs`, `kernel/src/syscall.rs`, `kernel/src/sched.rs`, `kernel/src/boot.rs`, `kernel/src/framebuffer/mod.rs`, `kernel/src/smp.rs`, `docs/status-audit.md`, `docs/manual-testing.md`, `docs/NEXT_STEPS.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Refreshed the lockfile for the current workspace dependency set; fixed current compiler blockers by updating GDT entry insertion, naked function attributes, `iretq` frame assembly, bootloader-region borrowing, framebuffer clear borrowing, and AP trampoline assembler expressions/far jumps; recorded local runtime tooling availability; and updated verification docs to separate build sanity from runtime proof.
- What was intentionally avoided: No DHCP, DNS, TCP, USB, dynamic linker, package manager, full compositor, preemptive scheduler, broad scheduler/SMP redesign, QEMU run, hardware test, or runtime verification claim.
- Runtime verified: no.
- If not verified, what still needs verification: `cargo check -p kernel --offline --locked` passed with warnings. `cargo check -p chronosapien --target aarch64-apple-darwin --offline --locked` passed with warnings. `qemu-system-x86_64` and `pwsh` were not available on PATH in this environment, so BIOS boot, UEFI boot, custom BIOS boot, framebuffer/serial output, shell behavior, input, storage, apps, userspace, networking, scheduler, and SMP behavior still need runtime verification.
- New risks introduced: Low to moderate; changes touch low-level boot/userspace/SMP assembly syntax to satisfy the current compiler and assembler, but intentionally avoid changing subsystem scope or adding behavior.
- Next recommended step: Install or expose PowerShell and QEMU locally, then start the staged BIOS QEMU pass from `docs/manual-testing.md` and record actual evidence before changing runtime status labels.

### 2026-06-01 — Add Phase 2 status audit and verification checklist
- Prompt/task: Implement the Phase 2 verification/stabilization plan by documenting implemented-in-code, partially implemented, risky, stale, and unverified systems without adding large features or claiming runtime success.
- Files changed: `README.md`, `docs/status-audit.md`, `docs/manual-testing.md`, `docs/NEXT_STEPS.md`, `docs/networking.md`, `docs/ring3.md`, `docs/syscalls.md`, `docs/elf.md`, `docs/custom-bootloader.md`, `docs/uefi.md`, `docs/shell-commands.md`, `docs/AI_PROGRESS_LOG.md`, `kernel/src/shell.rs`.
- What changed: Added a Phase 2 status audit with labels for boot paths, framebuffer/serial/timer/shell, window/app platform, scheduler/SMP, ring 3/syscalls/static ELF, ChronoFS journal/recovery, networking, notes, museum/quest, mouse/window interaction, heap, and IRQ keyboard behavior; expanded manual testing with build sanity, scheduler, SMP, window/app, recovery, userspace, networking, and product-command boundary checks; linked the audit from README/NEXT_STEPS; added conservative status headers to networking/userspace/boot docs; documented `notes` as the active notes backing file; and aligned doctor/poster mouse/network wording with implemented-in-code or partially implemented but unverified status.
- What was intentionally avoided: No DHCP, DNS, TCP, USB, dynamic linker, package manager, full compositor, preemptive scheduler, broad refactor, QEMU run, hardware test, feature expansion, or package/image rename.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity, BIOS boot, custom BIOS boot, UEFI boot, framebuffer output, serial output, PIT/timer behavior, shell startup, app launch, filesystem commands, product/demo commands, IRQ keyboard with polling fallback, PS/2 mouse/window interactions, heap reuse, ChronoFS read/write/delete, fsck/repair, journal recovery, ring 3 demo, syscalls, static ELF execution, ARP/UDP networking, cooperative scheduler, and SMP/AP startup.
- New risks introduced: Low; changes are documentation/status alignment plus shell wording only. No runtime behavior was intentionally changed.
- Next recommended step: Fix the current build-only errors narrowly, then start staged QEMU evidence from BIOS boot, framebuffer/serial, shell startup, keyboard, and ChronoFS basics before touching high-risk SMP/userspace/networking paths.

### 2026-06-01 — Align docs and source truth to ChronoOS
- Prompt/task: Rename the public/product identity from Time Capsule OS to ChronoOS in public docs, remove stale source-truth claims, add manual verification and shell command reference docs, fix the notes filename mismatch, and update quest wording for IRQ keyboard/reusable heap status.
- Files changed: `AGENTS.md`, `README.md`, `docs/showcase.md`, `docs/NEXT_STEPS.md`, `docs/storage.md`, `docs/architecture.md`, `docs/boot-flow.md`, `docs/roadmap.md`, `docs/manual-testing.md`, `docs/shell-commands.md`, `docs/AI_PROGRESS_LOG.md`, `kernel/src/quest.rs`, `kernel/src/apps/notes.rs`, `kernel/src/apps/sysinfo.rs`, `kernel/src/wm.rs`, `kernel/src/shell.rs`, `kernel/src/theme.rs`, `kernel/src/framebuffer/mod.rs`.
- What changed: Made ChronoOS the public/product name while preserving `chronosapien` repo/package/image naming; replaced stale bump-only/no-journal/no-mouse claims with implemented-in-code and needs-runtime-verification labels; created staged manual testing checklists; added a shell command reference; standardized notes storage on the `notes` ChronoFS file; and changed quest status text so IRQ keyboard and reusable heap are implemented in code but unverified instead of locked future work.
- What was intentionally avoided: No TCP, DHCP, DNS, USB, dynamic linker, package manager, full compositor, preemptive scheduler, large feature work, QEMU run, hardware test, image build, lockfile change, or project/package rename.
- Runtime verified: no.
- If not verified, what still needs verification: `cargo check -p kernel` was attempted and failed on existing low-level compile errors in naked asm attributes, asm `noreturn` outputs, GDT API usage, boot/framebuffer borrows, and related kernel paths. Build sanity, BIOS boot, custom BIOS boot, UEFI boot, framebuffer/serial output, shell commands, IRQ keyboard and polling fallback, mouse/window interactions, heap reuse, ChronoFS read/write/delete, fsck/repair, journal recovery, apps, product commands, ring 3, syscalls, static ELF execution, ARP/UDP networking, SMP, and scheduler behavior still need verification.
- New risks introduced: Low; most changes are documentation/status alignment. Source changes are limited to user-visible strings, quest status labels, and the notes storage filename used by the small app/window path.
- Next recommended step: Fix the current build-only errors narrowly, then follow `docs/manual-testing.md` in staged QEMU passes and record actual evidence before claiming runtime verification.

### 2026-05-26 — Add persistent AI progress tracking
- Prompt/task: Create project-level AI/Codex instructions, an append-only AI progress log, and a living next-steps file for Chronosapian / Time Capsule OS.
- Files changed: `AGENTS.md`, `docs/AI_PROGRESS_LOG.md`, `docs/NEXT_STEPS.md`.
- What changed: Added standing instructions for future AI work, documented the current source of truth, added the first progress entry, and created a staged priority/roadmap tracking file.
- What was intentionally avoided: No terminal commands, build commands, QEMU runs, scripts, lockfile regeneration, OS source changes, new OS features, README changes, or project renaming.
- Runtime verified: no.
- If not verified, what still needs verification: Future runs should verify build sanity, BIOS boot, framebuffer/serial/shell behavior, filesystem persistence, mouse/window behavior, timer/interrupts, ring 3/syscall/ELF behavior, networking, SMP, and UEFI/custom boot paths.
- New risks introduced: None expected; this is documentation/process setup only.
- Next recommended step: Run build sanity checks later in a dedicated verification task before changing OS features.

### 2026-05-26 — Add IRQ1 keyboard buffering
- Prompt/task: Implement interrupt-driven PS/2 keyboard input while preserving polling input as a fallback.
- Files changed: `kernel/src/keyboard.rs`, `kernel/src/pic.rs`, `kernel/src/interrupts.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added a fixed-size no-heap keyboard event ring buffer, shared scancode decoding between IRQ and polling paths, wired IRQ1 into the IDT/PIC setup, and kept `keyboard::read_key()` as the shell-facing API.
- What was intentionally avoided: No terminal commands, build commands, QEMU runs, scripts, lockfile regeneration, unrelated refactors, new OS features, or project renaming.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity, BIOS boot, IRQ1 delivery, shell typing, shifted characters, Backspace, Enter, polling fallback behavior, timer IRQ0 regression, and mouse IRQ12 regression.
- New risks introduced: IRQ keyboard buffering adds another interrupt/shared-state path; runtime testing must confirm the PIC mask and EOI behavior are correct.
- Next recommended step: Run a dedicated build-and-boot verification pass later, starting with compile checks before QEMU testing.

### 2026-05-26 — Add reusable heap allocator
- Prompt/task: Replace the bump-only heap allocator with a small reusable free-list allocator.
- Files changed: `kernel/src/memory.rs`, `kernel/src/shell.rs`, `kernel/src/apps/sysinfo.rs`, `docs/AI_PROGRESS_LOG.md`, `docs/NEXT_STEPS.md`.
- What changed: Added a no-heap free-list allocator with allocation headers, first-fit block selection, splitting, deallocation, address-sorted reinsertion, coalescing, and expanded heap stats for used/free/largest-free reporting.
- What was intentionally avoided: No terminal commands, build commands, QEMU runs, scripts, lockfile regeneration, paging redesign, unrelated refactors, new OS features, dependencies, or project renaming.
- Runtime verified: no.
- If not verified, what still needs verification: Kernel compile sanity, boot to shell, `mem`/`sysinfo` stats, allocation reuse after repeated file/window/task operations, and regressions in ChronoFS, ELF loading, scheduler/window close, and apps.
- New risks introduced: The allocator now has metadata and coalescing logic that must be validated under real allocation/deallocation patterns; double-free detection and corruption guards are intentionally not included.
- Next recommended step: Run build-only checks later before any more kernel feature work, then boot and exercise heap-using shell commands.

### 2026-05-26 — Slice progress into commits and push
- Prompt/task: Split the current documentation/process, IRQ keyboard, and reusable heap progress into 20 commits and push the branch to GitHub.
- Files changed: `docs/AI_PROGRESS_LOG.md` plus the files already changed by the work being committed.
- What changed: Recorded the requested 20-commit slicing and GitHub push task after organizing the existing progress into a branch commit series.
- What was intentionally avoided: No feature changes beyond recording the commit/push task.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity and the runtime checks already listed for IRQ keyboard input and the reusable heap allocator.
- New risks introduced: None expected from this log entry; commit slicing itself should preserve existing file contents.
- Next recommended step: After push, run a dedicated build verification pass before adding more OS behavior.

### 2026-05-26 — Move active work to main
- Prompt/task: Set `main` as the main branch and push future work to `main`.
- Files changed: `docs/AI_PROGRESS_LOG.md`, `docs/NEXT_STEPS.md`.
- What changed: Fast-forwarded local `main` to include the previously pushed progress branch, recorded `main` as the active integration branch, and prepared the repository so future pushes land on `origin/main`.
- What was intentionally avoided: No branch deletion, force push, GitHub default-branch API change, feature work, broad refactor, or project rename.
- Runtime verified: no.
- If not verified, what still needs verification: Build/runtime verification remains the same: compile checks, QEMU boot, IRQ keyboard behavior, and reusable heap behavior.
- New risks introduced: Future direct pushes to `main` reduce branch isolation, so build checks should run before risky kernel changes.
- Next recommended step: Push `main`, then run build sanity checks before additional OS behavior changes.

### 2026-05-26 — Confirm main is fully synced
- Prompt/task: Compare the current repo with `main` and make sure everything in this repo is pushed to `main`.
- Files changed: `docs/AI_PROGRESS_LOG.md`.
- What changed: Fetched `origin/main`, confirmed local `main` and `origin/main` had no commit or file diff before this tracking entry, then prepared this log-only commit for push to `origin/main`.
- What was intentionally avoided: No feature work, branch deletion, force push, broad refactor, or project rename.
- Runtime verified: no.
- If not verified, what still needs verification: Build/runtime verification remains pending for the OS itself.
- New risks introduced: None expected; this is repository synchronization tracking only.
- Next recommended step: Keep future completed work on `main` and run build sanity before risky kernel edits.

### 2026-05-26 — Add ChronoFS checker and safe repair command
- Prompt/task: Add a conservative ChronoFS consistency checker with a read-only `fsck` shell command and optional `fsck repair` mode.
- Files changed: `kernel/src/fs.rs`, `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added a structured ChronoFS check report, superblock/table/name/extent/bitmap/duplicate-sector checks, read-only shell reporting, and repair for safe bitmap mismatches plus stale metadata in unused file table slots.
- What was intentionally avoided: No on-disk format change, journaling, ChronoFS rewrite, unrelated features, force repair of ambiguous damage, or QEMU/manual testing requirement.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity once Cargo is available, then later boot-level verification of `fsck` and `fsck repair` against clean and intentionally corrupted ChronoFS images.
- New risks introduced: The checker reads raw disk metadata and repair mode writes bitmap/table metadata; it refuses unsafe cases, but it still needs compile and runtime validation.
- Next recommended step: Restore Cargo/toolchain access and run a build-only check before further filesystem work.

### 2026-05-26 — Add tiny ChronoFS journal
- Prompt/task: Add a small ChronoFS journal for safer writes and deletes.
- Files changed: `kernel/src/fs.rs`, `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`, `docs/NEXT_STEPS.md`.
- What changed: Added a hidden one-sector `__chronofs_journal` record, write/remove intent and committed states, mount-time rollback/roll-forward recovery, conservative bitmap rebuilding, serial recovery logs, and a `journal` shell status command.
- What was intentionally avoided: No ChronoFS rewrite, directories, permissions, complex filesystem features, superblock format change, file table format change, bitmap format change, terminal commands, QEMU/manual testing request, or user-provided test data.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity once Cargo is available, then mount recovery scenarios for missing/clean/intent/committed/corrupt journals and user rejection of the reserved journal filename.
- New risks introduced: The journal consumes one hidden file slot and one data sector; recovery writes metadata during mount and must be compile/runtime validated.
- Next recommended step: Run a build-only check before adding more ChronoFS behavior, then test journal recovery with controlled disk images.

### 2026-05-26 — Slice journal progress into commits
- Prompt/task: Slice the ChronoFS journal progress into 12 commits and push to GitHub.
- Files changed: `docs/AI_PROGRESS_LOG.md` plus the files changed by the journal implementation.
- What changed: Recorded the requested commit slicing and push task for the journal work.
- What was intentionally avoided: No additional filesystem behavior beyond the journal implementation being committed.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity and the journal recovery scenarios listed above.
- New risks introduced: None expected from commit slicing itself.
- Next recommended step: After push, restore Cargo/toolchain access and run a build-only check.

### 2026-05-26 — Add text-first demo command
- Prompt/task: Add a safe `demo` shell command that guides users through the best current Time Capsule OS features without adding kernel subsystems.
- Files changed: `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added `demo` to shell dispatch/help and implemented a text-only tour covering the current era, era commands, museum pages, filesystem commands, sysinfo, apps, windows/tasks, and user-space previews.
- What was intentionally avoided: No era switching, file writes/deletes, `fsck repair`, app/window/task spawning, user-space execution, new low-level systems, terminal commands, QEMU/manual testing request, or project rename.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity once Cargo is available, then shell-level verification that `demo` prints the guide and `demo <anything>` prints usage.
- New risks introduced: Minimal; the command only prints text and reads the current era and file list.
- Next recommended step: Run a build-only check before adding more demo polish.

### 2026-05-26 — Slice demo command progress into commits
- Prompt/task: Slice the demo command progress into 3 commits and push to GitHub.
- Files changed: `docs/AI_PROGRESS_LOG.md` plus the files changed by the demo command implementation.
- What changed: Recorded the requested commit slicing and push task for the text-first demo work.
- What was intentionally avoided: No additional demo or kernel behavior beyond the implementation already being committed.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity and the demo shell scenarios listed above.
- New risks introduced: None expected from commit slicing itself.
- Next recommended step: After push, restore Cargo/toolchain access and run a build-only check.

### 2026-05-26 — Add inside-OS tour command
- Prompt/task: Add a read-only `tour` shell command family that explains Time Capsule OS from inside the OS.
- Files changed: `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added `tour`, `tour boot`, `tour memory`, `tour files`, `tour apps`, `tour userspace`, and `tour future`; added `tour` to help; implemented beginner-friendly, era-aware text sections that explain code-present boot, memory, ChronoFS, app/window, user-space, and future-roadmap concepts.
- What was intentionally avoided: No new kernel subsystems, era switching, file writes/deletes, `fsck repair`, app/window spawning, task spawning, user-space execution, terminal commands, manual QEMU testing request, or runtime-verification claim.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity is blocked in this shell because `cargo` is unavailable; shell checks for every `tour` form and an invalid form such as `tour now` are still needed.
- New risks introduced: Low; the command only prints text, reads the active era, and optionally lists current files through the existing read-only path.
- Next recommended step: Run a build-only check once terminal/toolchain use is allowed, then verify the shell output in the OS.

### 2026-05-26 — Push tour command to GitHub
- Prompt/task: Push the tour command work to GitHub.
- Files changed: `docs/AI_PROGRESS_LOG.md` plus the tour command implementation files.
- What changed: Recorded the push request for the inside-OS tour command work.
- What was intentionally avoided: No additional OS behavior beyond the tour command changes already prepared.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity is blocked in this shell because `cargo` is unavailable; shell checks for all `tour` forms are still needed.
- New risks introduced: None expected from committing and pushing the existing scoped changes.
- Next recommended step: Run a build-only check before extending the tour further.

### 2026-05-26 — Add capsule timeline command
- Prompt/task: Add a read-only `capsule` shell command family that shows completed, partial, and planned OS milestones using the current quest/museum style.
- Files changed: `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added `capsule`, `capsule milestones`, `capsule current`, and `capsule next`; added `capsule` to help; implemented era-aware timeline text that separates `code-present`, `partial`, `planned`, and `runtime verification needed`.
- What was intentionally avoided: No new kernel systems, roadmap file edits, era switching, file writes/deletes, fs repair, journal mutation, app/window/task spawning, user-space execution, terminal commands, manual testing request, or runtime-verification claim.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity plus shell checks for `capsule`, `capsule milestones`, `capsule current`, `capsule next`, and an invalid form such as `capsule now`.
- New risks introduced: Low; the command only prints text and reads the active era/profile.
- Next recommended step: Run a build-only check once terminal/toolchain use is allowed, then verify the capsule output in the OS shell.

### 2026-05-26 — Add doctor subsystem health command
- Prompt/task: Add a read-only `doctor` shell command that reports subsystem health without faking checks.
- Files changed: `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added `doctor` to shell dispatch/help and implemented an era-aware health report for serial, framebuffer, timer, keyboard, mouse, filesystem, heap, network, SMP/core count, scheduler, and userspace/ELF support.
- What was intentionally avoided: No new diagnostics framework, hardware probes, filesystem repairs, journal mutation, network tests, allocations for heap testing, task/app/window spawning, user-space execution, terminal commands, roadmap file edits, manual testing request, or runtime-verification claim.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity plus shell checks for `doctor` and an invalid form such as `doctor now`.
- New risks introduced: Low; the command only prints text and reads the active era/profile.
- Next recommended step: Run a build-only check once terminal/toolchain use is allowed, then verify the doctor output in the OS shell.

### 2026-05-26 — Improve apps text launcher
- Prompt/task: Improve the `apps` command into a text-based app launcher with era-specific styling.
- Files changed: `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added an era-styled `apps` launcher with entries for notes, calc, sysinfo, files, clock, museum, theme, and tasks; added `apps <name>` routing for existing simple app commands and text cards for files, museum, and theme command groups.
- What was intentionally avoided: No full desktop, heavy GUI architecture, new kernel subsystem, file mutation from the launcher, automatic era switching, terminal commands, manual testing request, or runtime-verification claim.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity plus shell checks for `apps`, every `apps <name>` entry, and an invalid form such as `apps paint`.
- New risks introduced: Low to moderate; `apps <name>` delegates to existing shell commands for notes, calc, sysinfo, clock, and tasks, so those paths still need runtime confirmation.
- Next recommended step: Run a build-only check once terminal/toolchain use is allowed, then verify the launcher output in each era.

### 2026-05-26 — Add travel year-to-era command
- Prompt/task: Add a `travel <year>` command that maps years to existing Time Capsule OS era profiles and explains the mapping.
- Files changed: `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added `travel <year>` to shell dispatch/help; mapped 1980s to `era 1984`, 1990s to `era 1995`, 2000s to `era 2007`, and 2010+ years to `era 2040`; prints the requested year, mapped era, explanation, and equivalent era command before delegating to the existing era command path.
- What was intentionally avoided: No new theme system, new era profiles, GUI/desktop behavior, app behavior, file writes, terminal commands, manual testing request, or runtime-verification claim.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity plus shell checks for `travel 1987`, `travel 1998`, `travel 2004`, `travel 2049`, malformed years, years before 1980, and extra arguments.
- New risks introduced: Low; the only intentional state change is switching the active era through the existing `era` command path.
- Next recommended step: Run a build-only check once terminal/toolchain use is allowed, then verify travel mappings in the OS shell.

### 2026-05-26 — Expand deep museum pages
- Prompt/task: Expand museum mode into deeper pages for disk, filesystem, userspace, syscalls, ELF, networking, SMP, and scheduler.
- Files changed: `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added dispatch and discoverability for `museum disk`, `museum filesystem`, `museum userspace`, `museum syscalls`, `museum elf`, `museum networking`, `museum smp`, and `museum scheduler`; each page explains what the concept means, what Time Capsule OS currently does, what real operating systems do, and what is still missing.
- What was intentionally avoided: No new kernel systems, probes, drivers, networking behavior, scheduler behavior, filesystem behavior, userspace execution behavior, terminal commands, manual testing request, roadmap file edits, or runtime-verification claim.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity plus shell checks for each new `museum <topic>` page and museum topic discoverability text.
- New risks introduced: Low; pages are text-only and read only the active era/profile for context.
- Next recommended step: Run a build-only check once terminal/toolchain use is allowed, then verify every new museum page in the OS shell.

### 2026-05-26 — Add poster mode screenshot screens
- Prompt/task: Add `poster` mode screens that create screenshot-friendly displays for build-in-public updates.
- Files changed: `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added `poster`, `poster boot`, `poster system`, `poster roadmap`, and `poster eras`; added `poster` to help; implemented compact ASCII-only text cards for overview, boot flow, subsystem status, roadmap state, and era showcase.
- What was intentionally avoided: No external assets, full graphics system, GUI/desktop architecture, file mutation, era switching, runtime checks, app/window/task spawning, userspace execution, terminal commands, manual testing request, roadmap file edits, or runtime-verification claim.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity plus shell checks for `poster`, `poster boot`, `poster system`, `poster roadmap`, `poster eras`, and an invalid form such as `poster gallery`.
- New risks introduced: Low; poster screens only print text and read the active era/profile.
- Next recommended step: Run a build-only check once terminal/toolchain use is allowed, then verify screenshots in the framebuffer/text UI.

### 2026-05-26 — Improve notes into persistent text app
- Prompt/task: Improve `notes` into a small persistent app with read/write/clear/save behavior and optional window mode if the existing window manager supports it.
- Files changed: `kernel/src/shell.rs`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added `notes`, `notes read`, `notes write <text>`, `notes clear`, `notes save`, and `notes open`; the app stores notes in the existing ChronoFS-style file name `notes` by delegating to existing `cat`, `write`, and `rm` shell commands, and `notes open` delegates to the existing `open notes` path.
- What was intentionally avoided: No complex editor, new storage subsystem, new window manager behavior, full desktop, terminal commands, manual testing request, or runtime-verification claim.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity plus shell checks for `notes`, `notes read`, `notes write hello`, `notes clear`, `notes save`, `notes open`, and invalid forms.
- New risks introduced: Moderate; notes now delegates through existing shell commands, so persistence depends on current ChronoFS command behavior and window mode depends on the existing `open notes` support.
- Next recommended step: Run a build-only check once terminal/toolchain use is allowed, then verify note persistence across reads and clears in the OS shell.

### 2026-05-26 — Add portfolio showcase case study
- Prompt/task: Create `docs/showcase.md` explaining Time Capsule OS like a portfolio case study.
- Files changed: `docs/showcase.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Added a showcase document with project concept, uniqueness, architecture overview, implemented systems, partial systems, product identity, screenshots/GIFs to capture later, resume bullet ideas, future roadmap, and build-in-public story.
- What was intentionally avoided: No runtime success claims, screenshots, GIFs, generated assets, external links, code changes, roadmap priority changes, kernel behavior changes, project rename, terminal commands, or manual testing request.
- Runtime verified: no.
- If not verified, what still needs verification: Editorial review plus later build/runtime evidence before converting capture targets into actual screenshots or success claims.
- New risks introduced: Low; this is documentation-only and uses conservative `code-present`, `partial`, `planned`, and `runtime verification needed` language.
- Next recommended step: After runtime verification, capture the listed screenshots/GIFs and link them from the showcase.

### 2026-05-26 — Slice recent progress into two commits
- Prompt/task: Slice recent progress into 2 commits and push to GitHub.
- Files changed: `kernel/src/shell.rs`, `docs/showcase.md`, `docs/AI_PROGRESS_LOG.md`.
- What changed: Recorded the requested two-commit slicing and push for the recent shell education/app features and showcase documentation.
- What was intentionally avoided: No additional OS behavior beyond the scoped changes already prepared.
- Runtime verified: no.
- If not verified, what still needs verification: Build sanity plus OS-shell checks for the recently added commands and docs review for `docs/showcase.md`.
- New risks introduced: None expected from commit slicing itself.
- Next recommended step: After push, run a build-only check once Cargo/toolchain access is available.
