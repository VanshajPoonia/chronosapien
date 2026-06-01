# AI Progress Log

## Purpose

This file tracks every major Codex/AI-assisted change to ChronoOS.

## Current Source of Truth

- Public OS identity: ChronoOS.
- Repo/package name: `chronosapien`; older internal/source text may still mention Chronosapian.
- Current development phase: post-Phase-4 stabilization, documentation, and verification planning before new feature work.
- Current source-truth handoff: `docs/CURRENT_STATUS.md`.
- Verified in QEMU with limits: single-core BIOS serial-only boot reached `[CHRONO] boot complete`, and boot-time serial logging was observed through that point.
- Implemented in code but still needs broader or first runtime verification: optional custom BIOS bootloader, UEFI loader, framebuffer UI, visible shell prompt/input, era themes, keyboard polling fallback, IRQ1 keyboard buffering, mouse, timer behavior from the shell, GDT/IDT/PIC behavior beyond boot, reusable heap allocator reuse, ChronoFS shell workflows, ATA storage workflows, ChronoFS `fsck`, ChronoFS journal/recovery, networking, ring 3 demo, syscalls, ELF execution, SMP/AP startup, scheduler task lifecycle, apps, museum, quests, and product commands.
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
