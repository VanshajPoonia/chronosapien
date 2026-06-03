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
- `learn next`: a safe first curriculum route.

Useful aliases include `learn fs`, `learn apps`, `learn windows`, `learn net`,
`learn smp`, `learn theme`, and `learn future`.

## Design

Each path prints:

- a short beginner-friendly explanation,
- current implementation status,
- verification status,
- related ChronoOS commands,
- one safe next command.

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
