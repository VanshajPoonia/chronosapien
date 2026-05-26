# AI Progress Log

## Purpose

This file tracks every major Codex/AI-assisted change to Time Capsule OS.

## Current Source of Truth

- Public OS identity: Time Capsule OS.
- Repo/package name: Chronosapian / `chronosapien`.
- Current development phase: stabilization, documentation, and verification planning before new feature work.
- Code-present but unverified: BIOS boot path, optional custom BIOS bootloader, UEFI loader, framebuffer UI, serial logging, shell, era themes, keyboard polling, IRQ1 keyboard buffering, mouse, timer, GDT/IDT/PIC, memory, ChronoFS, ATA storage, networking, ring 3 demo, syscalls, ELF execution, SMP, scheduler, apps, museum, and quests.
- Partial/missing systems: ChronoFS repair/journaling, richer graphics shell, fuller process model, preemptive scheduler, broader networking, real hardware/USB support, and runtime verification of IRQ keyboard input and reusable heap allocation.
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
