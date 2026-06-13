# ChronoOS App Platform

Status: implemented in code, needs runtime verification for the new app polish
commands.

ChronoOS uses a tiny static app registry to describe shell-first app surfaces.
It is compiled into the kernel as metadata. It is not a package manager,
dynamic linker, dynamic app loader, app store, installer, or permission model.

## Registry Fields

Each app manifest records:

- name
- category
- short description
- launch command
- implementation status
- verification status
- risk level
- related commands
- help text
- safe demo command list
- featured-app flag

## Categories

- Core
- Files
- Learning
- System
- Networking
- Visual
- Debug/Lab
- Roadmap/Future

## Commands

- `apps` / `apps list`: show the static registry with verification and risk
  badges.
- `apps featured`: show the best shell-first demo app surfaces.
- `apps recent`: show app launcher routes used since boot; this is in-memory
  only.
- `apps category <name>`: browse by category.
- `apps info <name>`: show one manifest plus related commands.
- `apps help <name>`: show app-specific help and related commands.
- `apps demo <name>`: show a safe demo path without running commands.
- `apps launch <name>`: run the existing launch command when the app is
  implemented and has a launch route.
- `apps verified`: list entries with recorded partial QEMU evidence.
- `apps roadmap`: list roadmap/design-only app ideas.

Existing aliases such as `apps notes`, `apps calc`, `apps sysinfo`,
`apps files`, `apps clock`, `apps museum`, `apps theme`, and `apps tasks`
remain available for beginner-friendly navigation.

## Featured Apps

- `notes`: tiny ChronoFS-backed notes surface.
- `calc`: integer calculator.
- `sysinfo`: era-aware system information screen.
- `files`: shell-first ChronoFS usability surface.
- `museum` / `learn`: educational OS concept paths.
- `theme`: era/theme preview.
- `status`: workspace, verification, and doctor surfaces.

## Current Registry

| App | Category | Launch | Status | Verification | Risk | Featured | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| notes | Core | `notes` | implemented in code | partially verified in QEMU | medium | yes | Notes home was observed; persistence still needs checks. |
| calc | Core | `calc` | implemented in code | partially verified in QEMU | low | yes | `calc 6 - 7` was observed; broader calculator paths need checks. |
| sysinfo | System | `sysinfo` | implemented in code | needs runtime verification | low | yes | Shell `sysinfo` still needs current QEMU proof. |
| files | Files | `files` | implemented in code | needs runtime verification | medium | yes | Shell-first ChronoFS usability surface, not a windowed explorer. |
| museum | Learning | `apps museum` | implemented in code | needs runtime verification | low | yes | Routes to museum pages. |
| learn | Learning | `learn` | implemented in code | needs runtime verification | low | yes | Guided learning paths. |
| theme | Visual | `apps theme` | partially implemented | needs runtime verification | low | yes | Preview card only, not a theme studio. |
| tasks | System | `tasks` | implemented in code | needs runtime verification | medium | no | Cooperative task inspection, not process management. |
| clock | System | `clock` | implemented in code | needs runtime verification | low | no | Timer tick helper. |
| status | System | `workspace` | implemented in code | needs runtime verification | low | yes | Workspace, verify, doctor, and mode status surfaces. |
| paint | Roadmap/Future | none | roadmap/design-only | roadmap/design-only | future | no | Future tiny paint idea. |
| network | Networking | `net` | roadmap/design-only | roadmap/design-only | high | no | Future guided network demo; ARP/UDP remains separate and unverified. |
| userspace | System | `userspace status` | partially implemented | needs runtime verification | high | no | Teaching surface for Ring 3/syscalls/ELF. |
| timeline | Learning | `capsule` | partially implemented | needs runtime verification | low | no | Text timeline surface; graphical timeline is future work. |
| crashlab | Debug/Lab | none | roadmap/design-only | roadmap/design-only | future | no | Future controlled crash/fault lab. |
| doctor | System | `doctor` | implemented in code | needs runtime verification | low | no | Conservative status/verification center. |

## Verification Boundary

The app registry itself is static code. Runtime verification still requires
visible QEMU or hardware evidence for each command. Current evidence covers
only narrow app paths such as `apps`, `apps list`, notes home, `calc 6 - 7`,
`open notes`, and `open sysinfo`.

New commands from the app-platform polish pass are implemented in code, not
verified: `apps featured`, `apps recent`, `apps category <name>`,
`apps help <name>`, and `apps demo <name>`.

## Future Boundary

Future dynamic app loading would need verified static ELF execution, safer
process lifecycle, clear file/storage semantics, stable app manifests,
permission and failure boundaries, and stronger runtime evidence. Do not
present the current registry as dynamic app loading or package management.
