# ChronoOS

ChronoOS is a beginner-friendly Rust `no_std` x86_64 educational hobby
operating system. It is an OS learning project with a product-minded shell:
eras, museum pages, quests, small apps, ChronoFS, guided tours, and
screenshot-friendly status screens.

The repository/package name may remain `chronosapien` for now. Generated image
names such as `chronosapien-bios.img` and `chronosapien-uefi.img` are legacy
internal names and should not be renamed casually.

ChronoOS is not a Linux replacement. It is a small, readable teaching OS.

## Portfolio Snapshot

ChronoOS exists to make low-level systems work understandable and memorable. It
combines a real Rust `no_std` x86_64 kernel with an indie educational product
layer: era themes, a museum-style shell, quests, small apps, ChronoFS, posters,
and guided demo commands.

What makes it different:

- It treats the shell as both a debugger and a product surface.
- It explains operating-system concepts from inside the OS itself.
- It keeps beginner-friendly docs beside source-truth status labels.
- It separates implemented-in-code work from runtime-verified evidence.

Implemented in code includes BIOS/UEFI/custom boot paths, framebuffer and serial
output paths, shell commands, era themes, keyboard and mouse paths, small
windows, ChronoFS, `fsck`, a tiny journal, apps, museum/quest/product commands,
a cooperative scheduler, SMP work, ARP/UDP networking, Ring 3 demos, syscalls,
and static ELF execution.

Verified so far is intentionally narrower: single-core BIOS serial-only QEMU
reached `[CHRONO] boot complete`, and boot-time serial logging was observed
through that point. Visual framebuffer output, shell interaction, apps,
filesystem workflows, userspace, networking, mouse/windows, UEFI, custom BIOS,
SMP/AP, and hardware still need dedicated verification.

Roadmap/design-only work includes TCP, DHCP, DNS, USB, a dynamic linker, package
manager, full desktop compositor, and production-grade preemptive scheduler.

What this project teaches:

- how a small OS boots and hands off to a Rust kernel,
- how serial/framebuffer output, interrupts, timers, input, memory, and storage
  fit together,
- how to keep ambitious systems work honest with verification boundaries,
- how technical depth and product storytelling can reinforce each other.

Portfolio/demo docs:

- `docs/CURRENT_STATUS.md`: current source-truth audit.
- `docs/demo-script.md`: 2-minute, 5-minute, and 10-minute demo paths.
- `docs/screenshots.md`: screenshot and GIF capture checklist.
- `docs/release-checklist.md`: release gate checklist.
- `docs/showcase.md`: portfolio case-study narrative.

## Status Labels

- implemented in code: present in the source tree.
- partially implemented: useful teaching version, not a complete production system.
- needs runtime verification: must be proven in QEMU or hardware before success is claimed.
- verified in QEMU: actual QEMU evidence is recorded in this repo.
- verified on hardware: actual hardware evidence is recorded in this repo.
- roadmap/design-only: intentionally not implemented yet.

No runtime success is claimed by this README unless the repo contains matching
verification evidence.

For the current post-Phase-4 source-truth audit, see
`docs/CURRENT_STATUS.md`. The older `docs/status-audit.md` remains the
Phase 2-specific risk audit.

## Current State

Verified in QEMU, with important limits:

- Single-core BIOS serial-only boot reached `[CHRONO] boot complete`.
- Boot-time serial logging was observed through that point.
- The run used `-display none`, so framebuffer output, visible shell prompt,
  keyboard input, shell commands, apps, storage workflows, windows, and graphics
  were not verified by that evidence.

Implemented in code, needs broader or first runtime verification:

- BIOS boot path through the `bootloader` crate, beyond the limited single-core
  serial-only smoke.
- Optional custom BIOS bootloader path.
- Optional UEFI loader path.
- Framebuffer console and top bar.
- Serial logging outside the recorded boot-time smoke.
- Era themes and an era-aware shell.
- IRQ1 PS/2 keyboard buffering with polling fallback.
- PS/2 mouse input and small draggable windows.
- GDT, IDT, PIC, PIT timer, exceptions, and PC speaker tones.
- Boot memory map handling, page mapping helpers, and a 1 MiB free-list heap.
- ATA PIO storage and ChronoFS.
- ChronoFS `fsck`, conservative `fsck repair`, and a tiny one-record journal.
- Built-in apps and product commands.
- Museum pages, quests, stats, and inventory.
- Cooperative scheduler and early SMP work.
- RTL8139 ARP/UDP networking.
- Ring 3 demo, syscall layer, and static ELF execution.

Partially implemented:

- Graphics shell and windows.
- App platform.
- Process/userspace model.
- Scheduler/SMP behavior.
- ChronoFS recovery.
- Networking.
- Real hardware support.

Roadmap/design-only:

- TCP, DHCP, DNS, sockets.
- USB HID, USB storage, USB serial.
- Dynamic linker.
- Package manager.
- Full desktop compositor or GUI toolkit.
- Production-grade preemptive scheduler.

## Known Limitations

- ChronoOS is not a Linux replacement.
- ChronoOS is not production-ready.
- Runtime verification is incomplete; many systems are implemented in code but
  still need visible QEMU or hardware evidence.
- Networking is limited to static IPv4 ARP/UDP code paths unless future
  verification and implementation expand it.
- USB, TCP, DHCP, DNS, dynamic linking, package management, a full compositor,
  and a preemptive scheduler are long-term roadmap/design-only items unless a
  future audit records implementation and proof.

## Folder Structure

```text
chronosapien/
|-- Cargo.toml
|-- build.rs
|-- boot/
|-- kernel/
|   `-- src/
|       |-- apps/
|       |-- framebuffer/
|       |-- ata.rs
|       |-- boot.rs
|       |-- console.rs
|       |-- elf.rs
|       |-- fs.rs
|       |-- gdt.rs
|       |-- interrupts.rs
|       |-- keyboard.rs
|       |-- memory.rs
|       |-- mouse.rs
|       |-- net.rs
|       |-- process.rs
|       |-- sched.rs
|       |-- shell.rs
|       |-- smp.rs
|       |-- syscall.rs
|       |-- theme.rs
|       |-- timer.rs
|       `-- wm.rs
|-- uefi-loader/
|-- scripts/
|-- docs/
|-- tools/
`-- user/
```

## Key Files

- `kernel/src/main.rs`: kernel entrypoint and startup sequence.
- `kernel/src/shell.rs`: line-based shell and most product/demo commands.
- `kernel/src/memory.rs`: memory map handling, page helpers, and free-list heap.
- `kernel/src/fs.rs`: ChronoFS, `fsck`, repair, and journal code.
- `kernel/src/keyboard.rs`: IRQ keyboard buffer plus polling fallback.
- `kernel/src/mouse.rs` and `kernel/src/wm.rs`: PS/2 mouse and small windows.
- `kernel/src/net.rs`: RTL8139, ARP, static IPv4, and UDP.
- `kernel/src/process.rs`, `kernel/src/syscall.rs`, `kernel/src/elf.rs`: userspace teaching paths.
- `docs/CURRENT_STATUS.md`: post-Phase-4 current-state source of truth.
- `docs/manual-testing.md`: staged verification checklist.
- `docs/status-audit.md`: Phase 2 status and risk audit.
- `docs/shell-commands.md`: current shell command reference.

## Build And Run Commands

Install Rust nightly, QEMU, and NASM if using the custom BIOS path.

Build the normal BIOS image:

```powershell
cargo build -p kernel
.\scripts\build.ps1
```

Run the normal BIOS image:

```powershell
.\scripts\run.ps1
```

Optional custom BIOS image:

```powershell
.\scripts\build-custom.ps1
.\scripts\run-custom.ps1
```

Optional UEFI image:

```powershell
.\scripts\build-uefi.ps1
.\scripts\run-uefi.ps1
```

Serial-only debug run:

```powershell
.\scripts\debug-serial.ps1
```

Use `docs/manual-testing.md` to record what was actually observed. Do not mark
anything runtime-verified from commands alone.

## Shell Surface

See `docs/shell-commands.md` for the command reference generated from the
current source shape.

Highlights:

- Core: `help`, `clear`, `about`, `reboot`, `uptime`, `clock`, `mem`, `cores`, `beep <hz>`.
- Era/product: `era`, `travel <year>`, `demo`, `tour`, `capsule`, `doctor`, `poster`.
- Filesystem: `ls`, `cat`, `write`, `rm`, `fsck`, `fsck repair`, `journal`.
- Apps: `apps`, `notes`, `calc`, `sysinfo`.
- Windows/tasks: `open notes`, `open sysinfo`, `tasks`, `kill <id>`.
- Userspace: `ring3`, `syshello`, `exec <name>`.
- Networking: `net`, `net arp`, `net send`.
- Museum/quest: `museum ...`, `quest list`, `quest status`, `stats`, `inventory`.

## Storage In Plain Language

ChronoOS uses a small educational filesystem named ChronoFS on a second QEMU IDE
disk. The layout is intentionally simple: superblock, file table, allocation
bitmap, and contiguous file data.

`fsck`, `fsck repair`, and a tiny journal are implemented in code. Repair and
recovery are conservative and still need runtime verification under controlled
disk states.

## Memory In Plain Language

The heap starts at `0x200000` and is 1 MiB. It uses a simple free-list allocator
with splitting, freeing, sorted reinsertion, and coalescing. This is implemented
in code but still needs runtime verification to prove reuse behavior across
real shell/app/window/filesystem workflows.

## What Is Ours And What Is Borrowed

Ours:

- Kernel startup after handoff
- Serial, console, framebuffer text rendering
- GDT, IDT, PIC, PIT, keyboard, mouse, sound, and basic device paths
- Memory helpers and free-list heap
- ATA PIO and ChronoFS
- Shell, eras, apps, museum, quests, and product commands
- Scheduler, SMP work, networking, userspace demos, syscalls, and ELF loader
- UEFI loader and image builders

Borrowed:

- `bootloader` and `bootloader_api`
- `uefi`
- `x86_64`
- QEMU
- Rust bare-metal toolchain support

That split is deliberate: early boot and CPU helpers are borrowed where they
keep the project readable, while ChronoOS owns the teaching surface and kernel
pieces it is meant to explain.
