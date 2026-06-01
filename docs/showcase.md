# ChronoOS Showcase

ChronoOS is an educational `no_std` x86_64 hobby operating system presented as
a time-travel learning environment. The project combines kernel engineering with
a product-style shell experience: eras, museum pages, quests, filesystem tools,
small apps, and screenshot-friendly progress screens.

This case study is intentionally conservative. Features are described as
`implemented in code`, `partially implemented`, `needs runtime verification`, or
`roadmap/design-only` instead of claiming runtime success that has not been
verified.

For the current post-Phase-4 audit, use `docs/CURRENT_STATUS.md`. For demo and
release packaging, use `docs/demo-script.md`, `docs/screenshots.md`, and
`docs/release-checklist.md`.

## Project Concept

ChronoOS teaches how computers work by letting the user explore the system from
inside the OS itself. The shell acts like a museum lobby and command center:
users can switch visual eras, inspect educational pages, run small built-in
apps, explore ChronoFS, and read guided explanations of boot, memory,
filesystems, userspace, scheduling, and networking.

The repository/package name may remain `chronosapien` for now. ChronoOS is the
public/product name.

## Why It Is Unique

- implemented in code: era profiles turn the OS into a small time machine: 1984, 1995, 2007, and 2040.
- implemented in code: museum, tour, demo, capsule, doctor, and poster commands explain the OS while it runs.
- implemented in code: ChronoFS gives the project a custom educational filesystem surface.
- implemented in code: the shell is not just a debug prompt; it is also the product interface.
- partially implemented: window, task, app, userspace, SMP, and networking ideas are teaching paths rather than production platforms.
- verified in QEMU: single-core BIOS serial-only boot reached `[CHRONO] boot complete`.
- needs runtime verification: visual framebuffer, shell interaction, filesystem commands, input, userspace, and network behavior must be tested before being advertised as runtime-proven.

## Architecture Overview

- Kernel foundation: Rust `no_std` x86_64 educational kernel.
- Text UI: framebuffer and serial output paths for shell-first interaction.
- Shell: command dispatch for system info, eras, apps, files, museum pages, guides, and diagnostics.
- Theme system: era profiles used by `era` and `travel <year>`.
- Input and time: IRQ keyboard buffering, polling fallback, PS/2 mouse path, PIT timer, and uptime counters are implemented in code.
- Storage: ChronoFS provides named file behavior, consistency checking, repair boundaries, and a tiny journal.
- Learning surfaces: museum pages, guided tours, poster screens, quest/status/inventory views, and conservative health summaries.
- Advanced teaching paths: window/task/user-space/ELF/scheduler/SMP/networking concepts are surfaced conservatively.

## Screenshots/GIFs To Capture Later

These are capture targets, not verified screenshots included in this document.
Use `docs/screenshots.md` for naming and verification notes.

- Boot screen and first shell prompt.
- `era 1984`, `era 1995`, `era 2007`, and `era 2040`.
- `travel 1987`, `travel 1998`, `travel 2004`, and `travel 2049`.
- Museum pages such as `museum filesystem`, `museum userspace`, and `museum scheduler`.
- ChronoFS workflows: `ls`, `write`, `cat`, `rm`, `fsck`, and `journal`.
- Poster screens: `poster`, `poster boot`, `poster system`, `poster roadmap`, and `poster eras`.
- App launcher: `apps` and selected `apps <name>` entries.
- Notes flow: `notes write <text>`, `notes read`, `notes clear`, and `notes open`.
- Diagnostic/guided outputs: `doctor`, `capsule`, `tour`, and `demo`.

## Resume Bullet Ideas

- Built ChronoOS, a Rust `no_std` x86_64 educational operating system with a shell-first product experience.
- Designed an era-based OS identity around museum-style learning pages, quests, and build-in-public progress tools.
- Implemented ChronoFS tooling for basic file workflows, consistency checking, conservative repair, and journal status.
- Created beginner-friendly OS education commands covering boot, memory, filesystems, userspace, syscalls, ELF, SMP, scheduling, and roadmap status.
- Developed text-based app and launcher flows that reuse existing kernel/shell paths without introducing heavy GUI architecture.
- Practiced honest systems documentation by separating implemented-in-code, partially implemented, needs-runtime-verification, and roadmap/design-only work.

## Future Roadmap

- needs runtime verification: run build, boot, shell, filesystem, app, userspace, and network checks.
- roadmap/design-only: capture screenshots and GIFs after verification.
- roadmap/design-only: strengthen ChronoFS recovery examples without guessing at user data.
- roadmap/design-only: expand userspace and ELF examples with clearer safety boundaries.
- roadmap/design-only: keep improving apps and museum pages in small, understandable steps.

## Demo Package

The portfolio demo package is documentation-only until visible QEMU evidence is
captured. The intended path is:

- Use `docs/demo-script.md` to choose a 2-minute, 5-minute, or 10-minute command
  sequence.
- Use `docs/screenshots.md` to capture boot, shell, era, app, filesystem,
  museum, poster, userspace, and networking evidence with accurate status tags.
- Use `docs/release-checklist.md` before publishing a README update, showcase
  post, or demo build.
- Keep `docs/CURRENT_STATUS.md` as the final word on which claims are verified,
  partial, or roadmap/design-only.

## Build-In-Public Story

ChronoOS has grown from kernel basics into a small educational environment. The
project now emphasizes not just whether a subsystem exists, but how clearly it
can be explained from inside the OS.

Runtime verification remains separate from implemented-in-code claims. The next
strong portfolio step is a visible BIOS QEMU pass: framebuffer, shell prompt,
keyboard input, storage commands, product commands, and screenshots tied to real
evidence.
