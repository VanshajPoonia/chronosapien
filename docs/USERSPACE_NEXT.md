# ChronoOS Userspace And Process Next Steps

Status: design only, source-truth planning document.

ChronoOS has enough userspace pieces to teach the boundary between kernel mode,
Ring 3 code, syscalls, and static ELF loading. It does not yet have a general
process model. This document defines the safest staged path from the current
boundary toward stronger process behavior without pretending that the missing
pieces already exist.

## Stage 1 — Current Boundary

Current implemented pieces:

- `ring3`: fixed teaching demo that enters Ring 3 and proves a privileged
  instruction traps back to the kernel.
- `userspace status`: read-only boundary summary.
- `userspace syscalls`: read-only syscall table for `write`, `read`, `exit`,
  and `uptime`.
- `syshello`: fixed user-code page demo for `SYS_WRITE` and `SYS_EXIT`.
- `userspace elf`: read-only static ELF support summary.
- `exec <name>`: foreground static ELF path that reads a file from ChronoFS,
  maps `PT_LOAD` segments, enters the ELF entry point, and returns through
  `SYS_EXIT` when the foreground ELF path is active.

Current evidence:

- `userspace status`, `userspace syscalls`, and the fixed `ring3` demo have
  narrow QEMU evidence from the 2026-06-13 userspace-boundary pass.
- `userspace elf` and `syshello` still need manual verification because QEMU
  monitor input was garbled.
- `exec hello.elf` is blocked until a known safe test ELF can be built and
  installed into a disposable ChronoFS image.

Known limitations:

- No process table.
- No fork/exec semantics.
- No argv/env.
- No libc.
- No dynamic linker.
- No package manager.
- No permissions model.
- No preemptive user-process scheduler.
- Static ELF execution is a foreground teaching path, not a general
  multiprocess platform.

## Stage 2 — Process Metadata

The next safe implementation step should be read-only process metadata for the
single foreground ELF path. It should not add scheduling semantics.

Suggested metadata:

- process ID: a small fixed ID for the foreground ELF slot, such as `1`.
- process name: copied from the `exec <name>` filename with a fixed capacity.
- state: `inactive`, `loading`, `running`, `exited`, or `faulted`.
- entry point: ELF entry address when known.
- memory range summary: number of mapped user ranges plus stack range.
- exit code: last foreground ELF exit code when available.
- verification status: for example `implemented in code, not verified` or
  `verified in QEMU` only after evidence exists.

Safe status surface:

```text
userspace process
```

or:

```text
process status
```

The command should be read-only, should work when no process is active, and
should clearly say that it is not a process table.

## Stage 3 — Safer App Execution

After metadata exists, the next execution step should make `exec hello.elf`
repeatable and safer to inspect.

Requirements:

- Use only the known `user/hello.c` test program or another documented test ELF.
- Build artifacts outside the repo when possible, such as under `/private/tmp`.
- Install the test ELF into a disposable ChronoFS image, not the repo's normal
  data image.
- Connect static app metadata to the test program only as a launch hint. Do not
  add dynamic app loading or package management.
- Improve error reporting only in small, conservative ways:
  - missing file
  - bad ELF magic
  - unsupported ELF
  - malformed ELF
  - out of memory
  - process already active

Verification gate:

- `exec hello.elf` must not be marked verified until serial logs show the exact
  command, ELF load lines, syscall write/exit lines if present, and visible
  framebuffer output or screenshot evidence.

## Stage 4 — argv/env Design

This stage is design-only until a future prompt explicitly asks for
implementation.

Possible future layout:

- user stack contains `argc`, `argv` pointers, a null `argv` terminator, `envp`
  pointers, and a null `envp` terminator.
- strings live below the pointer arrays on the initial user stack.
- arguments are copied from the shell command after `exec <name>`.
- environment starts empty until ChronoOS has a reason to expose variables.

Do not implement this yet:

- command-line parsing policy
- environment variables
- libc startup
- shell quoting
- process inheritance

## Stage 5 — Stronger Isolation

Before claiming stronger userspace isolation, ChronoOS needs better checks and
clearer failure behavior.

Future hardening targets:

- validate every user pointer and length before kernel access.
- make syscall error returns consistent and documented.
- keep user code/data/stack inside the configured user ranges.
- reject ELF mappings outside the user ELF window.
- report user faults with a readable status instead of hanging silently.
- keep kernel mappings supervisor-only after CR3 switches.
- add QEMU tests for bad syscall buffers, unsupported ELF files, and user faults.

This remains a hardening path, not a permissions model.

## Stage 6 — Future Preemption

Preemption is design-only and should not be a near-term userspace milestone
unless the repo has much stronger evidence for:

- reliable keyboard/input verification.
- stable task lifecycle commands.
- clear scheduler cleanup.
- repeatable ELF execution and exit.
- safe process metadata.
- fault handling that can return to a known kernel state.

Future preemption would require timer-driven scheduling, saved user register
state, per-process address spaces, kernel stacks, runnable/blocked states, and
careful locking. It should not be v0.3 primary work unless the evidence matrix
is much healthier.

## Acceptance Criteria For The Next Implementation Prompt

The next actual userspace implementation prompt should be limited to a
read-only process metadata/status layer.

Acceptance criteria:

- Add metadata for only the one foreground static ELF slot.
- Expose it through `userspace process` or `process status`.
- Show name, ID, active/inactive state, entry point, mapped range count, stack
  range, last exit code if available, and verification status.
- Work cleanly when no process has ever run.
- Preserve current `ring3`, `syshello`, `userspace status`, `userspace
  syscalls`, `userspace elf`, and `exec <name>` behavior.
- Do not add scheduling semantics, fork, argv/env, dynamic linking, packages,
  permissions, or preemption.
- Update `docs/userspace-model.md`, `docs/VERIFICATION_MATRIX.md`, and
  `docs/AI_PROGRESS_LOG.md` conservatively.
- Pass `cargo check -p kernel --offline --locked`.
- Pass `git diff --check`.
- Keep new runtime labels as `implemented in code, not verified` until a later
  QEMU pass records exact serial command lines and screenshots.

## Safest Future Prompt

```text
Implement a read-only userspace process metadata/status command for the single
foreground static ELF path. Do not implement a full process table, scheduler
changes, argv/env, dynamic linking, package management, fork/exec semantics, or
preemption. Keep all labels conservative and update docs.
```
