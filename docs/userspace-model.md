# ChronoOS User-Space Model

Status: partially implemented, risky, needs runtime verification.

ChronoOS has early user-space teaching paths, not a full Unix-like process
model. The current goal is to make privilege transitions, syscalls, and static
ELF loading understandable before adding broader process features.

## Current Pieces

- `ring3`: copies a tiny fixed byte sequence into the demo user page and enters
  ring 3 with `iretq`. The first instruction is privileged (`hlt`), so the demo
  is meant to show CPU-enforced privilege separation and exception handling.
- `syshello`: copies a small fixed machine-code program into the same demo user
  page. It calls `SYS_WRITE` and `SYS_EXIT` through `SYSCALL`.
- `exec <name>`: reads a static ELF64 file from ChronoFS, parses `PT_LOAD`
  segments, creates a user address space, maps code/data plus a small stack,
  enters the ELF entry point, and can return to the shell through `SYS_EXIT`.

These paths are implemented in code but are not runtime-verified in the current
evidence matrix.

## Syscall ABI

The ABI is intentionally tiny:

- `rax`: syscall number
- `rdi`, `rsi`, `rdx`: first three arguments
- `rax`: return value
- `rcx` and `r11`: clobbered by `SYSCALL` / `SYSRET`

| Syscall | Number | Inputs | Outputs | Status | Verification |
| --- | --- | --- | --- | --- | --- |
| `write` | `1` | `fd`, `buffer`, `len` | byte count or error value | implemented in code | not verified |
| `read` | `2` | `fd`, `buffer`, `len` | byte count or error value | implemented in code | not verified |
| `exit` | `3` | `code` | returns to shell for active ELF process; parks older fixed demo | implemented in code | not verified |
| `uptime` | `4` | none | PIT tick count | implemented in code | not verified |

Only stdin/stdout/stderr-style numeric descriptors are recognized in this tiny
ABI. There is no general file descriptor table.

## Static ELF Boundary

ChronoOS accepts a small static executable subset:

- ELF64
- little-endian
- `ET_EXEC`
- `EM_X86_64`
- `PT_LOAD` segments
- fixed virtual addresses in the ChronoOS user ELF window

The loader maps user pages, copies file bytes, leaves BSS-style memory zeroed,
maps a small stack, switches CR3, and enters the ELF entry point. Syscall buffer
validation uses the active ELF process ranges.

## Scheduler Boundary

The cooperative scheduler manages kernel/app task slots such as shell and small
window tasks. It is not a preemptive user-process scheduler. `exec <name>` is a
foreground teaching path, not a general multi-process platform.

## Not Supported

- dynamic linker
- package manager
- `fork`
- Unix-style `exec` replacement semantics
- argv/env setup
- libc
- general file descriptors
- full process table
- signals
- permissions model
- preemptive scheduling
- mature process isolation claims

## Risks

- Ring transitions, page-table changes, and syscall entry are fragile low-level
  paths.
- The fixed `ring3` / `syshello` demo page is separate from the static ELF
  process window.
- `SYS_EXIT` behavior depends on whether a foreground ELF process is active.
- Runtime behavior has not been proven for `ring3`, `syshello`, or
  `exec hello.elf` in the recorded QEMU evidence.

## Safe Next Verification

Use a controlled single-core BIOS QEMU pass:

```text
userspace status
userspace syscalls
userspace elf
ring3
syshello
exec hello.elf
```

Build/install the test ELF with `scripts/build-user.ps1` before testing
`exec hello.elf`. Record serial logs and screenshots before upgrading any
verification labels.
