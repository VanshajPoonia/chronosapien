# ChronoOS v0.2 Roadmap

ChronoOS v0.2 should be a reliability-and-depth release, not a random feature
grab. The release should make the existing OS easier to verify, easier to demo,
and easier to explain before risky systems expand.

Current evidence is tracked in `docs/VERIFICATION_MATRIX.md`. Do not upgrade
runtime claims from this roadmap alone.

## Recommendation

Primary v0.2 track: **Goal A - Reliability And Verification**.

Secondary v0.2 track: **Goal B - ChronoFS Hardening**.

This pairing gives ChronoOS the strongest portfolio step: visible proof,
screenshots, reliable demo paths, and one serious technical subsystem that can
be inspected deeply. It also avoids piling networking expansion, USB, package
management, dynamic linking, full compositing, and preemptive scheduling into
the same release.

## Goal A - Reliability And Verification

- Why it matters: ChronoOS has many code-present systems, but the project is
  only credible when the docs show exactly what was observed.
- User/product value: Makes demos, screenshots, README claims, and portfolio
  posts trustworthy.
- Technical value: Converts existing shell, app, input, and storage surfaces
  from "implemented in code" into evidence-backed status where possible.
- Risk level: Low to medium. QEMU verification can expose bugs, but it should
  not require new subsystems.
- Dependencies: QEMU, PowerShell, BIOS image build, ChronoFS data image, serial
  logs, QEMU monitor `screendump`, and `docs/release-checklist.md`.
- Suggested Codex prompt order:
  1. Visible single-core BIOS QEMU proof for framebuffer shell, keyboard input,
     safe/demo commands, and screenshots.
  2. One-command-at-a-time product command verification for `start`, `guide`,
     `learn`, `mode`, `doctor`, `poster`, `capsule`, and `apps`.
  3. Screenshot/GIF evidence pass with exact image/log paths.
  4. Update `docs/VERIFICATION_MATRIX.md`, `docs/CURRENT_STATUS.md`, and release
     docs with only observed evidence.
- Acceptance criteria: Build checks pass; BIOS QEMU reaches the visible shell;
  keyboard input is observed; demo-safe commands run; screenshots or a blocked
  screenshot reason are recorded; docs contain exact commands, logs, and dates.
- Verification method: QEMU single-core BIOS run, serial log capture, QEMU
  monitor screenshots, and command-by-command shell notes.
- What not to build yet: New product commands, new kernel features, networking
  protocols, UEFI/custom BIOS fixes, SMP/AP changes, hardware claims, or broad
  UI rewrites.

## Goal B - ChronoFS Hardening

- Why it matters: ChronoFS is one of the most concrete "real OS" systems in
  ChronoOS, and it is understandable enough to harden carefully.
- User/product value: Gives demos a serious technical center: inspect storage,
  write files, check consistency, explain repair limits, and show persistence.
- Technical value: Improves confidence in file operations, fsck diagnostics,
  journal status, repair boundaries, metadata assumptions, and fallback modes.
- Risk level: Medium. Filesystem repair and recovery can mutate disk images, so
  tests must use controlled data images.
- Dependencies: Current `fs` inspection commands, `fsck`, `journal`, ChronoFS
  data image, manual-testing checklist, and verification matrix.
- Suggested Codex prompt order:
  1. QEMU ChronoFS verification for `fs status`, `fs info`, `ls`, `write`,
     `cat`, `rm`, `fs check`, `fs journal`, `fsck`, and `journal`.
  2. Controlled clean-disk `fsck repair` test with before/after evidence.
  3. Persistence test across reboot using a throwaway file.
  4. Metadata exploration docs for layout, file slots, limits, and journal
     reservation without changing the disk format.
- Acceptance criteria: Basic shell file workflows are observed in QEMU; repair
  is tested only on a controlled image or left unverified with a reason; journal
  output is documented; no disk format break is introduced.
- Verification method: Single-core QEMU shell run, serial/screenshot evidence,
  controlled disk image notes, and updated status docs.
- What not to build yet: Directories, permissions, large-file support, POSIX
  behavior, complex journaling, broad recovery rewrite, or hidden repair magic.

## Goal C - User-Space Foundation

- Why it matters: Ring 3, syscalls, and static ELF execution are high-value
  teaching paths, but they should not be mistaken for a full Unix-like process
  model.
- User/product value: Helps users understand privilege, syscalls, ELF loading,
  and future process work without overclaiming.
- Technical value: Clarifies syscall ABI, static ELF boundaries, return-to-shell
  behavior, fault behavior, and future process-table needs.
- Risk level: Medium to high. User-mode transitions and ELF loading can expose
  low-level bugs.
- Dependencies: `userspace status`, `userspace syscalls`, `userspace elf`,
  `ring3`, `syshello`, `exec <name>`, ChronoFS file availability, and build-user
  scripts if used.
- Suggested Codex prompt order:
  1. Documentation consistency pass for syscall numbers and static ELF limits.
  2. QEMU verification of `userspace status` and `userspace syscalls`.
  3. Controlled `ring3` and `syshello` run with serial/framebuffer evidence.
  4. Static ELF test only after a known test ELF exists through existing scripts.
- Acceptance criteria: Docs match source; status commands run; at least one
  user-mode teaching path is observed or honestly blocked; future argv/env and
  process-table plans are documented without implementation.
- Verification method: Single-core QEMU run, serial log, screenshot evidence,
  and exact command notes.
- What not to build yet: Dynamic linker, package manager, argv/env, fork/exec
  semantics, libc, broad file descriptors, full process isolation, or preemptive
  multitasking.

## Goal D - Product Learning Experience

- Why it matters: ChronoOS is strongest when it teaches the OS from inside the
  OS instead of only through external docs.
- User/product value: Makes first boot, demos, museum pages, quests, and status
  screens feel coherent for beginners.
- Technical value: Improves command organization and routes users toward
  existing systems without adding risky code.
- Risk level: Low. Most work should be shell text and docs.
- Dependencies: `start`, `guide`, `learn`, `museum`, `quest`, `tour`, `capsule`,
  `poster`, `doctor`, `mode`, and verification/status docs.
- Suggested Codex prompt order:
  1. Verify current onboarding and learning-path commands in QEMU.
  2. Tighten museum/quest unlock text only where source truth is stale.
  3. Add small era-specific explanations only if they route to existing systems.
  4. Refresh demo and screenshot docs after evidence is collected.
- Acceptance criteria: First-run path is visible and readable; educational
  commands point to next steps; status labels stay conservative; no command
  duplicates or fake verification surfaces are added.
- Verification method: Visible QEMU shell walkthrough, screenshots, and docs
  updates tied to observed command output.
- What not to build yet: Large museum rewrites, complex quest persistence,
  automatic first-run state, broad new app systems, or runtime claims without
  evidence.

## Goal E - UI/App Shell Polish

- Why it matters: The app/window layer helps ChronoOS feel like a tiny desktop,
  but it should stay within the current lightweight model.
- User/product value: Makes app discovery, window lifecycle, and demo surfaces
  easier to navigate.
- Technical value: Exercises task/window lifecycle, static app metadata, shell
  fallbacks, and framebuffer redraw behavior.
- Risk level: Medium. UI changes are visible and can interact with input,
  windows, and cooperative tasks.
- Dependencies: app registry, `apps`, `apps info`, `apps launch`, `windows`,
  `open notes`, `open sysinfo`, mouse path, and QEMU screenshot workflow.
- Suggested Codex prompt order:
  1. QEMU verify `apps`, `apps list`, `apps info notes`, `apps launch calc`,
     `apps verified`, and `apps roadmap`.
  2. Verify `open notes`, `open sysinfo`, `windows list`, `windows status`,
     `windows focus <id>`, and `windows close <id>`.
  3. Polish only shell/window wording found confusing during verification.
  4. Keep file explorer, tiny paint, and dashboard/status screens scoped as
     roadmap unless an existing safe text surface is enough.
- Acceptance criteria: App and window commands are discoverable; shell fallback
  paths remain usable; window lifecycle docs match observed behavior; roadmap
  apps do not pretend to launch.
- Verification method: Visible QEMU input, screendumps, serial/task notes, and
  updated verification matrix rows.
- What not to build yet: Full compositor, GUI toolkit, animations, GPU
  acceleration, complex event loop rewrite, package manager, dynamic app
  loading, or full file explorer window mode.

## Intentionally Deferred For v0.2

- Networking expansion beyond current static IPv4 ARP/UDP observability.
- TCP, DHCP, DNS, sockets, or packet capture.
- USB HID, USB storage, USB serial, or broad hardware support.
- Dynamic linker, package manager, or dynamic app loading.
- Full compositor, GUI toolkit, or production desktop environment.
- Production-grade preemptive scheduler.
- Broad SMP/AP work unless a narrow verification task specifically targets it.

## Exact Next Prompt

Implement the v0.2 primary track: Reliability and Verification, starting with
visible single-core BIOS QEMU proof for framebuffer shell, keyboard input,
safe/demo commands, screenshots, and release-checklist updates. Do not add
features.
