# Time Capsule OS Showcase

Time Capsule OS is an educational `no_std` x86_64 hobby operating system presented as a time-travel learning environment. The project combines kernel engineering with a product-style shell experience: eras, museum pages, quests, filesystem tools, and screenshot-friendly progress screens.

This case study is portfolio-oriented, but intentionally conservative. Features are described as `code-present`, `partial`, `planned`, or `runtime verification needed` instead of claiming runtime success that has not been verified.

## Project Concept

Time Capsule OS is a beginner-friendly operating system that teaches how computers work by letting the user explore the system from inside the OS itself.

The shell acts like a museum lobby and command center. Users can switch visual eras, inspect educational pages, run small built-in apps, explore ChronoFS, and read guided explanations of boot, memory, filesystems, userspace, scheduling, and more.

## Why It Is Unique

- `code-present`: Era profiles turn the OS into a small time machine: 1984, 1995, 2007, and 2040.
- `code-present`: Museum, tour, demo, capsule, doctor, and poster commands explain the OS while it runs.
- `code-present`: ChronoFS gives the project a custom educational filesystem surface.
- `code-present`: The shell is not just a debug prompt; it is also the product interface.
- `partial`: Window, task, app, and userspace ideas are presented as teaching paths rather than a full desktop platform.
- `planned`: The project is designed for build-in-public storytelling, with clear labels for what exists, what is partial, and what still needs verification.

## Architecture Overview

- Kernel foundation: Rust `no_std` x86_64 educational kernel.
- Text UI: framebuffer and serial-style output paths for shell-first interaction.
- Shell: command dispatch for system info, eras, apps, files, museum pages, guides, and diagnostics.
- Theme system: existing era profiles used by commands like `era` and `travel <year>`.
- Input and time: keyboard/timer paths are code-present, with commands such as `clock`, `uptime`, and `mem`.
- Storage: ChronoFS provides named file behavior, consistency checking, repair, and a tiny journal where code-present.
- Learning surfaces: museum pages, guided tours, poster screens, and status summaries.
- Advanced teaching paths: window/task/user-space/ELF/scheduler/SMP/networking concepts are documented and surfaced conservatively.

## Implemented Systems

- `code-present`: Shell commands and help text for exploring the OS.
- `code-present`: Era switching and year mapping through `era` and `travel <year>`.
- `code-present`: Museum pages for core and deeper OS concepts.
- `code-present`: Guided commands including `demo`, `tour`, `capsule`, `doctor`, and `poster`.
- `code-present`: ChronoFS basics such as `ls`, `cat`, `write`, `rm`, and `exec <name>` where supported.
- `code-present`: ChronoFS `fsck`, conservative repair behavior, and journal status where present in the source tree.
- `code-present`: Text-based app launcher and small notes flow using existing file behavior.
- `code-present`: Built-in app commands such as notes, calc, and sysinfo where present.

## Partial Systems

- `partial`: Window mode and task commands are teaching previews, not a full desktop environment.
- `partial`: Userspace and ELF support are educational paths and still need broad verification.
- `partial`: Scheduler and SMP concepts are surfaced, but not claimed as production-grade behavior.
- `partial`: Networking is described conservatively; no full verified network stack is claimed here.
- `partial`: ChronoFS recovery is intentionally conservative and refuses ambiguous repairs.
- `partial`: Apps are small shell-first workflows, not a mature application platform.

## Product Identity

Time Capsule OS uses a retro-to-future identity to make kernel learning feel more approachable.

- 1984: monochrome early personal computer mood.
- 1995: classic desktop GUI era.
- 2007: glossy mobile-web transition.
- 2040: speculative future lab.

The voice is educational, honest, and build-in-public friendly. It should help a beginner understand what is happening without hiding the technical reality.

## Screenshots/GIFs To Capture Later

These are capture targets, not verified screenshots included in this document.

- Boot screen and first shell prompt.
- `era 1984`, `era 1995`, `era 2007`, and `era 2040`.
- `travel 1987`, `travel 1998`, `travel 2004`, and `travel 2049`.
- Museum pages such as `museum filesystem`, `museum userspace`, and `museum scheduler`.
- ChronoFS workflows: `ls`, `write`, `cat`, `rm`, `fsck`, and `journal`.
- Poster screens: `poster`, `poster boot`, `poster system`, `poster roadmap`, and `poster eras`.
- App launcher: `apps` and selected `apps <name>` entries.
- Notes flow: `notes write <text>`, `notes read`, and `notes clear`.
- Diagnostic/guided outputs: `doctor`, `capsule`, `tour`, and `demo`.

## Resume Bullet Ideas

- Built a Rust `no_std` x86_64 educational operating system with a shell-first product experience.
- Designed Time Capsule OS identity around era profiles, museum-style learning pages, and build-in-public progress tools.
- Implemented code-present ChronoFS tooling for basic file workflows, consistency checking, repair boundaries, and journal status.
- Created beginner-friendly OS education commands covering boot, memory, filesystems, userspace, syscalls, ELF, SMP, scheduling, and roadmap status.
- Developed text-based app and launcher flows that reuse existing kernel/shell paths without introducing heavy GUI architecture.
- Practiced honest systems documentation by separating code-present, partial, planned, and runtime-verification-needed work.

## Future Roadmap

- `planned`: Run a build-only verification pass once toolchain use is available.
- `planned`: Verify shell behavior in the OS for guide, filesystem, app, and poster commands.
- `planned`: Capture screenshots and GIFs for the showcase targets listed above.
- `planned`: Strengthen ChronoFS recovery examples without guessing at user data.
- `planned`: Expand userspace and ELF examples with clearer safety boundaries.
- `planned`: Keep improving apps and museum pages in small, understandable steps.

## Build-In-Public Story

Time Capsule OS has grown from kernel basics into a small educational environment. The project now emphasizes not just whether a subsystem exists, but how clearly it can be explained from inside the OS.

The current story is:

- Build the kernel foundation.
- Add shell commands that make the system observable.
- Add ChronoFS so the OS has its own small file story.
- Add consistency, repair, and journaling concepts conservatively.
- Add guides, museum pages, app launcher flows, and poster screens so progress can be shared clearly.

Runtime verification remains separate from code-present claims. The next strong portfolio step is to run a build/shell verification pass, capture screenshots, and attach real runtime evidence to the showcase.
