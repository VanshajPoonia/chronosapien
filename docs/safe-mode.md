# ChronoOS Reliability And Safe Mode

Status: implemented in code, not runtime-verified.

ChronoOS reliability mode is a shell-level product/UX safety layer. It helps
separate demo-safe commands from verification and experimental paths. It does
not change boot behavior, block commands, persist settings, or provide a
security sandbox.

## Commands

- `mode` / `mode status`: show the current reliability mode and command
  categories.
- `mode safe`: prefer read-only demo paths and print stronger warnings.
- `mode demo`: return to the default portfolio/demo mode.
- `mode experimental`: mark the shell as intentionally exploring lab paths.
- `safe` / `safe status`: aliases for `mode status`.
- `safe on`: alias for `mode safe`.
- `safe off`: alias for `mode demo`.

The mode is stored in memory only and resets to `demo` after reboot.

## Mode Meanings

- `safe`: best for screenshots, demos, and first-time exploration. Risky
  commands still run, but warnings say they are outside the safe demo path.
- `demo`: the default mode. It is meant for normal portfolio walkthroughs while
  still warning before risky paths.
- `experimental`: useful during intentional verification. It does not imply that
  a command has been runtime-verified.

## Command Categories

Demo-safe/read-only examples:

- `help`, `start`, `guide`, `learn`, `demo`, `tour`, `capsule`, `poster`,
  `doctor`
- `apps list`, `apps info <name>`
- `fs status`, `fs info`, `fs check`, `fs journal`, `journal`
- `net status`, `net config`, `net log`, `net demo`, `net roadmap`
- `userspace status`, `userspace syscalls`, `userspace elf`,
  `userspace roadmap`
- `windows status`, `windows list`

Verification or controlled-mutation examples:

- `write`, `rm`, `notes write`, `notes clear`
- `fsck repair`
- `net arp`, `net send`
- `open notes`, `open sysinfo`, `windows focus <id>`, `windows close <id>`
- `kill <id>`

Experimental or risky examples:

- `ring3`, `syshello`, `exec <name>`
- `reboot`
- SMP/AP, UEFI, custom BIOS, crash/fault paths, and hardware tests

## Boundaries

Safe mode is warning-only. It does not prevent typing or running commands, does
not isolate processes, does not protect the filesystem, and does not prove that
a subsystem works. Runtime status should only be upgraded after QEMU or hardware
evidence is captured and recorded.
