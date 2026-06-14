# ChronoOS v0.3 Roadmap

Theme: "ChronoOS as a tiny educational OS workspace."

ChronoOS v0.3 should turn the current code-present product surfaces into a
small, credible, evidence-backed educational workspace. The release should
focus on verification, discoverability, and reliable shell-first workflows
instead of broad new kernel systems.

This roadmap is planning only. It does not upgrade runtime verification labels.
Use `docs/VERIFICATION_MATRIX.md` for evidence and
`docs/POST_VERIFICATION_SUMMARY.md` for the latest product decision point.

## Executive Summary

The verified base is useful but narrow:

- single-core BIOS boot, serial logs, framebuffer shell, screenshots, narrow
  onboarding/status/app registry, a basic disposable-image ChronoFS flow, and
  the fixed `ring3` teaching demo have QEMU evidence.
- shell workspace polish, ChronoFS usability, app platform polish, learning
  progress map, and many product/status commands are implemented in code but
  still need focused QEMU smoke verification.
- manual keyboard input, Backspace, Shift, mouse movement/drag/close, visual
  window close, `tasks`, `kill`, `syshello`, static ELF execution, UEFI kernel
  handoff, and hardware remain unverified, blocked, or high risk.

v0.3 should therefore be a verification-and-workspace release, not a networking,
USB, dynamic linker, package manager, compositor, or preemptive scheduler
release.

## Recommendation

Primary v0.3 track: **Reliability and verification for the educational
workspace**.

Secondary v0.3 track: **Window/input stabilization**.

Optional stretch track: **Userspace/process foundation**, limited to read-only
foreground ELF metadata and verification planning.

This keeps the release realistic: make the BIOS-based educational workspace
trustworthy first, reduce the biggest interaction risk second, and keep
userspace/process work small enough to avoid pretending ChronoOS has a full
process model.

## Track Evaluation

| Track | Priority | Why | Risk | Acceptance Criteria |
| --- | --- | --- | --- | --- |
| Reliability and verification | Primary | Turns code-present workspace features into honest evidence-backed release material. | Low-medium | BIOS QEMU smoke passes record serial logs/screenshots for workspace, apps, learning, ChronoFS usability, and status commands. |
| Shell workspace polish | Primary support | Main user surface for the educational OS workspace. | Low | `workspace`, `shortcuts`, `whereami`, `recent`, `status`, `verify`, `help search`, and typo suggestions are verified or honestly blocked. |
| ChronoFS usability | Primary support | Gives the release a concrete "real OS" storage story. | Medium | Disposable-image proof exists for `files` commands, copy refusal/overwrite behavior, and docs keep repair/recovery separate. |
| App platform polish | Primary support | Makes built-in apps discoverable without dynamic loading. | Low-medium | Static app metadata commands are verified; roadmap apps do not launch or overclaim. |
| Learning progress map | Primary support | Makes ChronoOS feel educational from inside the OS. | Low | `learn map`, `learn progress`, glossary, museum index, and quest views are verified as static/read-only. |
| Window/input stabilization | Secondary | Reduces the biggest product polish risk around real interaction. | Medium-high | Manual typing, Backspace, Shift, window close, `tasks`, `kill`, and mouse behavior have evidence or honest blockers. |
| Userspace/process foundation | Stretch | Valuable teaching depth, but easy to overbuild. | Medium-high | Only read-only foreground ELF metadata or verification planning; no full process model. |
| UEFI/hardware readiness | Separate technical track | Useful portability story, but not required for BIOS-based v0.3. | High | UEFI `Out of Resources` is investigated separately; no hardware claims without proof. |

## Track Details

### 1. Reliability And Verification

- Product value: Makes v0.3 portfolio-ready by showing exactly what works.
- Technical value: Converts recent code-present product layers into evidence or
  honest blockers.
- Current status: Verified BIOS core plus many unverified shell/product
  surfaces.
- Verification status: Partially verified in QEMU.
- Risk level: Low-medium.
- Dependencies: BIOS image build, QEMU, serial logs, monitor screenshots,
  disposable ChronoFS images, and `docs/VERIFICATION_MATRIX.md`.
- Acceptance criteria: Focused QEMU passes record exact command lines, serial
  logs, screenshots, and honest blocked notes for workspace, apps, learning,
  ChronoFS usability, and status commands.
- What not to build yet: New subsystems, networking expansion, USB, dynamic
  loading, package management, full compositor, preemption, or hardware claims.

### 2. Shell Workspace Polish

- Product value: Gives users a clear first surface: dashboard, shortcuts,
  current context, status, verification summary, and help search.
- Technical value: Exercises stable shell dispatch without touching risky
  low-level systems.
- Current status: Implemented in code.
- Verification status: Implemented in code, not verified.
- Risk level: Low.
- Dependencies: Existing BIOS shell, framebuffer output, serial command logs,
  and recent command storage.
- Acceptance criteria: `workspace`, `shortcuts`, `whereami`, `recent`, `status`,
  `verify`, `theme`, `help search fs`, `help search app`, and typo suggestions
  for `hlep`, `apss`, `verfy`, and `lern` are observed or honestly blocked.
- What not to build yet: New line editor, persistent history, shell scripting,
  login/session model, or runtime certification command.

### 3. ChronoFS Usability

- Product value: Gives v0.3 a practical storage workflow that feels like a tiny
  OS.
- Technical value: Exercises file metadata inspection, search, copy refusal,
  copy success, and error reporting through existing ChronoFS paths.
- Current status: `files` namespace is implemented in code.
- Verification status: Basic ChronoFS is partially verified in QEMU; the
  usability namespace is not verified.
- Risk level: Medium because `files copy` writes to disk.
- Dependencies: Disposable data image, existing `write`/`cat`/`rm`, `fs check`,
  `journal`, and screenshot/serial capture.
- Acceptance criteria: `files`, `files sample`, `files list`, `files info`,
  `files search`, `files demo`, `files copy`, overwrite refusal, rename refusal,
  `fs check`, and `journal` are verified on a throwaway image.
- What not to build yet: Directories, permissions, timestamps, large file
  support, complex journaling, automatic repair, or POSIX behavior.

### 4. App Platform Polish

- Product value: Makes built-in apps easier to discover without pretending
  ChronoOS has dynamic apps.
- Technical value: Validates static app metadata, categories, risk labels,
  verification badges, and launch delegation.
- Current status: Implemented in code.
- Verification status: `apps` and `apps list` have narrow QEMU evidence; newer
  app platform commands are not verified.
- Risk level: Low-medium.
- Dependencies: Static `AppManifest`, shell command routing, app launcher
  history, and existing notes/calc/sysinfo paths.
- Acceptance criteria: `apps featured`, `apps recent`, app categories,
  `apps info notes`, `apps help notes`, `apps demo notes`, `apps launch calc`,
  `apps verified`, and `apps roadmap` are observed or honestly blocked.
- What not to build yet: Package manager, dynamic app loading, dynamic linker,
  app store, installer, permissions model, or app sandbox.

### 5. Learning Progress Map

- Product value: Makes ChronoOS feel like an educational workspace from inside
  the OS.
- Technical value: Connects existing museum, quest, guide, status, userspace,
  storage, app, and roadmap surfaces without adding risky kernel behavior.
- Current status: Implemented in code.
- Verification status: Implemented in code, not verified.
- Risk level: Low.
- Dependencies: `learn`, `explain`, `museum`, `quest`, and conservative status
  labels.
- Acceptance criteria: `learn map`, `learn progress`, `learn beginner`,
  `learn advanced`, `learn next`, `explain kernel`, `explain filesystem`,
  `explain syscall`, `explain ARP`, `museum index`, `quest dependencies`, and
  `quest badges` are observed with readable framebuffer output.
- What not to build yet: Persistent progress tracking, automatic quest unlocks,
  large museum rewrite, certification badges, or hidden runtime probes.

### 6. Userspace/Process Foundation

- Product value: Preserves the high-value teaching story around Ring 3,
  syscalls, ELF, and future process work.
- Technical value: Clarifies the boundary between fixed demos, foreground ELF
  execution, and future process metadata.
- Current status: Partially implemented; `docs/USERSPACE_NEXT.md` defines the
  staged next path.
- Verification status: `userspace status`, `userspace syscalls`, and fixed
  `ring3` have QEMU evidence; `syshello`, `userspace elf`, and `exec hello.elf`
  remain unverified or blocked.
- Risk level: Medium-high.
- Dependencies: Reliable input, known safe `hello.elf`, disposable ChronoFS
  image, syscall logs, and foreground ELF metadata design.
- Acceptance criteria: Stretch only: add or plan read-only foreground ELF
  metadata, or verify existing `userspace elf`/`syshello`/`exec hello.elf`
  without expanding the process model.
- What not to build yet: Full process table, fork/exec semantics, argv/env,
  libc, dynamic linker, package manager, permissions, or preemptive scheduling.

### 7. Window/Input Stabilization

- Product value: Reduces the biggest user-facing polish risk: whether
  interaction behaves reliably.
- Technical value: Tests keyboard, mouse, window lifecycle, tasks, kill, and
  scheduler cleanup evidence.
- Current status: Partially implemented.
- Verification status: Open/list/focus and one serial-backed close have narrow
  QEMU evidence; manual input, Backspace, Shift, mouse movement/drag/close,
  `tasks`, and `kill` remain unverified.
- Risk level: Medium-high.
- Dependencies: Visible QEMU session, reliable manual input or better input
  harness, disposable image, serial logs, and screenshots.
- Acceptance criteria: Manual typing, Backspace, Shift, window close with
  follow-up list/tasks evidence, `tasks`, safe `kill <observed-id>`, visible
  mouse movement, drag, and mouse close are verified or honestly blocked.
- What not to build yet: Full compositor, GUI toolkit, complex event loop
  rewrite, animations, file explorer window, or desktop environment.

### 8. UEFI/Hardware Readiness

- Product value: Keeps a future portability story alive without blocking the
  BIOS-based product release.
- Technical value: Narrows the UEFI loader `Out of Resources` failure and keeps
  hardware claims honest.
- Current status: UEFI build/image path is fixed; OVMF starts the loader.
- Verification status: Partially verified in QEMU UEFI; kernel handoff and shell
  are not verified; no hardware proof exists.
- Risk level: High.
- Dependencies: OVMF, QEMU UEFI command, serial logs, screenshot capture, and
  loader memory-map/resource investigation.
- Acceptance criteria: Separate technical pass either fixes or explains the
  `Out of Resources` failure with evidence. Hardware remains unclaimed unless
  actual hardware logs/screenshots exist.
- What not to build yet: Broad hardware support, USB HID/storage/serial, custom
  installer, hardware compatibility matrix, or UEFI as a v0.3 release blocker.

## Intentionally Deferred For v0.3

- TCP, DHCP, DNS, sockets, and broader networking.
- USB HID, USB storage, USB serial, and broad hardware support.
- Dynamic linker, package manager, dynamic app loading, and app store behavior.
- Full compositor, GUI toolkit, file explorer window, and tiny paint.
- Preemptive scheduler, production process scheduling, and SMP/AP expansion.
- Full userspace process model, fork/exec semantics, argv/env, libc, and
  permissions.

## Exact Next Prompt

```text
Run a v0.3 educational workspace verification pass in visible single-core BIOS QEMU. Verify workspace, shortcuts, whereami, status, verify, help search, apps featured/category/info/demo, learn map/progress/beginner/advanced, museum index, quest badges/dependencies, files list/info/search/sample/demo, and capture serial logs plus screenshots. Do not add features or upgrade labels without evidence.
```
