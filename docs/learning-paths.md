# ChronoOS Learning Paths

Status: implemented in code, not runtime-verified.

ChronoOS learning paths are shell-first curriculum screens. They connect the
existing museum, quest, guide, tour, status, app, filesystem, userspace, and
networking commands without running probes or changing system state.

## Commands

- `learn`: curriculum overview.
- `learn boot`: firmware, bootloader, kernel handoff, and boot evidence.
- `learn memory`: memory map, paging, heap, and memory status.
- `learn interrupts`: IDT, timer, keyboard, mouse, and input boundaries.
- `learn filesystem`: ChronoFS, `fs status`, `fs check`, fsck, and journal.
- `learn gui`: shell apps, app registry, tiny windows, and window limits.
- `learn userspace`: Ring 3, syscalls, static ELF, and process-model limits.
- `learn networking`: RTL8139, static IPv4, ARP, UDP, and observability.
- `learn scheduler`: cooperative tasks, SMP/AP status, and scheduler limits.
- `learn eras`: era themes, travel commands, and visual identity.
- `learn roadmap`: future systems marked as roadmap/design-only.
- `learn map`: compact status and verification map for major learning areas.
- `learn progress`: static progress summary, badges pointer, and next route.
- `learn beginner`: safe first route through boot, memory, filesystem, apps/UI,
  and status.
- `learn advanced`: verification-oriented route through userspace, networking,
  scheduler/SMP, fsck/journal, and boot-boundary follow-up.
- `learn next`: recommends the learning map and the next safe route.
- `explain <term>`: short glossary entry for common OS terms.
- `museum index`: compact index of core and deeper museum pages.
- `quest dependencies`: static dependency-style learning route.
- `quest badges`: static learning badges derived from quest state.

Useful aliases include `learn fs`, `learn apps`, `learn windows`, `learn net`,
`learn smp`, `learn theme`, and `learn future`.

## Design

Each path prints:

- a short beginner-friendly explanation,
- current implementation status,
- verification status,
- related ChronoOS commands,
- one safe next command.

`learn map` adds a compact table-like view for Boot, Memory,
Interrupts/Input, Filesystem, Apps/UI, Userspace, Networking, Scheduler/SMP,
and Roadmap/Future. It shows status, verification level, suggested command, and
one related command for each area.

`learn progress`, `quest dependencies`, and `quest badges` are static
presentation screens. They do not track what the current user has actually run.

The learning paths intentionally route to existing content instead of duplicating
large museum or tour pages. They should stay compact enough for the framebuffer
console.

## Verification Boundary

Learning paths are educational shell screens. They do not certify that a
subsystem works. Runtime labels should only be upgraded after QEMU or hardware
evidence is captured and recorded in `docs/AI_PROGRESS_LOG.md` and
`docs/VERIFICATION_MATRIX.md`.

## Future Improvements

- Add screenshots after visible QEMU verification.
- Add one-command-at-a-time demo scripts for `learn boot`, `learn filesystem`,
  `learn gui`, and `learn networking`.
- Expand quest integration after runtime evidence is stronger.
