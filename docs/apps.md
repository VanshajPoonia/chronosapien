# ChronoOS App Registry

Status: implemented in code, needs runtime verification.

ChronoOS uses a tiny static app registry to describe shell-first app surfaces.
It is compiled into the kernel as metadata. It is not a package manager,
dynamic linker, dynamic app loader, app store, or installer.

## Registry Fields

Each app manifest records:

- name
- category
- short description
- launch command
- implementation status
- verification status
- risk level

## Commands

- `apps` / `apps list`: show the static registry.
- `apps info <name>`: show one manifest.
- `apps launch <name>`: run the existing launch command when the app is
  implemented and safe to route through the shell.
- `apps verified`: list entries with recorded partial QEMU evidence.
- `apps roadmap`: list roadmap/design-only app ideas.

Existing aliases such as `apps notes`, `apps calc`, `apps sysinfo`,
`apps files`, `apps clock`, `apps museum`, `apps theme`, and `apps tasks`
remain available for beginner-friendly navigation.

## Current Registry

| App | Category | Launch | Status | Verification | Risk | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| notes | apps | `notes` | implemented in code | partially verified in QEMU | medium | Notes home was observed; persistence still needs checks. |
| calc | apps | `calc` | implemented in code | partially verified in QEMU | low | `calc 6 - 7` was observed; broader calculator paths need checks. |
| sysinfo | apps | `sysinfo` | implemented in code | needs runtime verification | low | Code-present, not observed in recorded app pass. |
| files | storage | `apps files` | implemented in code | needs runtime verification | medium | Text card for shell file commands, not a windowed explorer. |
| museum | education | `apps museum` | implemented in code | needs runtime verification | low | Routes to museum pages. |
| theme | eras | `apps theme` | partially implemented | needs runtime verification | low | Preview card only, not a theme studio. |
| tasks | system | `tasks` | implemented in code | needs runtime verification | medium | Cooperative task inspection, not process management. |
| paint | creative | none | roadmap/design-only | roadmap/design-only | future | Future tiny paint idea. |
| network | networking | `net` | roadmap/design-only | roadmap/design-only | high | Future guided network demo; ARP/UDP remains separate and unverified. |
| userspace | systems | `userspace status` | partially implemented | needs runtime verification | high | Teaching surface for Ring 3/syscalls/ELF. |
| timeline | product | `capsule` | partially implemented | needs runtime verification | low | Text timeline surface; graphical timeline is future work. |
| crashlab | debug | none | roadmap/design-only | roadmap/design-only | future | Future controlled crash/fault lab. |
| doctor | status | `doctor` | implemented in code | needs runtime verification | low | Conservative status/verification center. |

## Future Boundary

Future dynamic app loading would need verified static ELF behavior, safer
process lifecycle, clear file/storage semantics, and stronger runtime evidence.
Do not present the current registry as dynamic app loading or package
management.
