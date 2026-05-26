# Chronosapian Project Export

## 1. Project Name And Purpose

Project name in `Cargo.toml`: `chronosapien`.

Primary README title: `Chronosapian`.

Docs and source also use `ChronoOS` for the OS/boot protocol naming. This naming split is real in the repo and should be cleaned up later if a single public name is desired.

Chronosapian is a beginner-friendly Rust hobby operating system project. It builds a `no_std` x86_64 kernel for QEMU, with BIOS and UEFI boot paths, framebuffer text UI, serial logging, era-themed presentation, persistent storage, early networking, ring 3 demos, static ELF execution, SMP work, mouse input, small windows, and cooperative tasks.

This file describes the current repository state only. Features are marked as code-present unless runtime verification was possible.

## 2. What We Are Building

We are building a small educational operating system that boots on emulated PC-compatible x86_64 hardware and exposes kernel concepts through a visible shell/UI:

- Bootable kernel via the `bootloader` crate BIOS image path.
- Optional custom BIOS bootloader path with Stage 1 and Stage 2 code in this repo.
- Optional Rust UEFI loader that packages a GPT/FAT32 EFI System Partition image.
- Framebuffer console with era-specific colors, prompt, boot text, top bar, cursor timing, boot chimes, text frames, sysinfo formatting, and window styling.
- Tiny command shell with built-ins, apps, file commands, networking commands, museum/quest commands, windows, and task commands.
- Early kernel subsystems: GDT, TSS, IDT, PIC, PIT, serial, PS/2 keyboard polling with IRQ1 buffering, PS/2 mouse IRQ handling, ATA PIO, PCI scanning, RTL8139 networking, paging, reusable free-list heap, SMP discovery/AP startup, and cooperative scheduling.
- Small ring 3/user-mode path: fixed demo, SYSCALL/SYSRET services, and one foreground static ELF process loaded from ChronoFS.

## 3. Current Tech Stack

- Language: Rust 2021, nightly, `no_std` kernel.
- Targets in `rust-toolchain.toml`: `x86_64-unknown-none`, `x86_64-unknown-uefi`.
- Assembly: NASM-style `.asm` files for the optional custom BIOS bootloader.
- User demo: freestanding C compiled with `clang`, linked with `ld.lld`.
- Build/package tools: Cargo workspace, root `build.rs`, standalone Rust tools compiled with `rustc`, PowerShell scripts.
- Emulation target: QEMU `qemu-system-x86_64`.
- Main Rust dependencies declared in manifests:
  - root build dependency `bootloader = "0.11.15"`.
  - kernel dependencies `bootloader_api = "0.11.15"` and `x86_64 = "0.14"`.
  - UEFI loader dependency `uefi = "0.37.0"` with `alloc` and `global_allocator`.

Important warning: `Cargo.lock` is stale/inconsistent. It only lists `bootloader 0.9.34` and `kernel`, while the manifests use `bootloader 0.11.15`, `bootloader_api`, `x86_64`, and the `uefi-loader` workspace member. The lockfile needs regeneration once Cargo is available.

## 4. Current Folder/File Structure

Tracked files from `git ls-files` before creating this export:

```text
.cargo/config.toml
.gitignore
Cargo.lock
Cargo.toml
README.md
boot/stage1/stage1.asm
boot/stage2/stage2_long.asm
boot/stage2/stage2_pm.rs
boot/stage2/stage2_real.asm
build.rs
docs/architecture.md
docs/boot-flow.md
docs/custom-bootloader.md
docs/dev-log.md
docs/elf.md
docs/networking.md
docs/ring3.md
docs/roadmap.md
docs/storage.md
docs/syscalls.md
docs/uefi.md
kernel/Cargo.toml
kernel/src/apps/calc.rs
kernel/src/apps/mod.rs
kernel/src/apps/notes.rs
kernel/src/apps/sysinfo.rs
kernel/src/ata.rs
kernel/src/boot.rs
kernel/src/console.rs
kernel/src/elf.rs
kernel/src/framebuffer/font.rs
kernel/src/framebuffer/mod.rs
kernel/src/fs.rs
kernel/src/gdt.rs
kernel/src/interrupts.rs
kernel/src/io.rs
kernel/src/keyboard.rs
kernel/src/main.rs
kernel/src/memory.rs
kernel/src/mouse.rs
kernel/src/museum.rs
kernel/src/net.rs
kernel/src/panic.rs
kernel/src/pci.rs
kernel/src/pic.rs
kernel/src/process.rs
kernel/src/quest.rs
kernel/src/ring3.rs
kernel/src/sched.rs
kernel/src/serial.rs
kernel/src/shell.rs
kernel/src/smp.rs
kernel/src/sound.rs
kernel/src/spinlock.rs
kernel/src/syscall.rs
kernel/src/theme.rs
kernel/src/timer.rs
kernel/src/wm.rs
rust-toolchain.toml
scripts/build-custom.ps1
scripts/build-uefi.ps1
scripts/build-user.ps1
scripts/build.ps1
scripts/debug-serial.ps1
scripts/run-custom.ps1
scripts/run-uefi.ps1
scripts/run.ps1
scripts/write-usb.ps1
src/main.rs
tools/chronofs_put.rs
tools/custom_image_builder.rs
tools/uefi_image_builder.rs
uefi-loader/Cargo.toml
uefi-loader/src/main.rs
user/hello.c
user/user.ld
```

README's folder tree is stale. It does not list several current source files, including `kernel/src/boot.rs`, `kernel/src/mouse.rs`, `kernel/src/museum.rs`, `kernel/src/quest.rs`, `kernel/src/sched.rs`, and `kernel/src/wm.rs`.

## 5. Important Files

- `Cargo.toml`: root package/workspace and BIOS image build dependency setup.
- `build.rs`: uses `bootloader` 0.11 BIOS image builder and emits `CHRONOSAPIEN_BIOS_IMAGE`.
- `.cargo/config.toml`: default build target is `x86_64-unknown-none`; enables nightly `bindeps`.
- `rust-toolchain.toml`: pins nightly components and bare-metal/UEFI targets.
- `src/main.rs`: small host-side binary that prints the generated BIOS image path.
- `kernel/src/main.rs`: kernel entrypoints and boot initialization sequence.
- `kernel/src/boot.rs`: shared `BootContext`, bootloader crate handoff, custom/UEFI Chrono boot handoff v1/v2 parsing.
- `kernel/src/framebuffer/mod.rs` and `font.rs`: pixel drawing, text console, top bar, taskbar, mouse cursor, era rendering effects, window drawing primitives.
- `kernel/src/console.rs`: `print!` and `println!` macros over the framebuffer writer.
- `kernel/src/serial.rs`: COM1 serial output guarded by a spinlock.
- `kernel/src/gdt.rs`, `interrupts.rs`, `pic.rs`, `timer.rs`: descriptor tables, exception/IRQ handlers, PIC remap, PIT timer.
- `kernel/src/memory.rs`: boot memory map handling, paging helpers, fixed user pages, reusable free-list heap allocator, user address-space support.
- `kernel/src/keyboard.rs`: PS/2 keyboard decoder with IRQ1 buffering and polling fallback.
- `kernel/src/mouse.rs`: interrupt-driven PS/2 mouse packet handling and event publication.
- `kernel/src/shell.rs`: command loop and command dispatch.
- `kernel/src/theme.rs`: era profiles for prompts, colors, boot text, sounds, windows, text frames, and sysinfo.
- `kernel/src/wm.rs`: fixed-capacity framebuffer window manager for notes/sysinfo windows.
- `kernel/src/sched.rs`: cooperative round-robin task scheduler with fixed stacks.
- `kernel/src/smp.rs`: ACPI MADT parsing, AP startup trampoline, local APIC setup, per-core tracking.
- `kernel/src/ata.rs`, `fs.rs`: ATA PIO sector I/O and ChronoFS filesystem facade with disk or heap fallback.
- `kernel/src/pci.rs`, `net.rs`: PCI config scanning and RTL8139 ARP/UDP network stack.
- `kernel/src/ring3.rs`, `syscall.rs`, `elf.rs`, `process.rs`: ring 3 demo, syscall ABI, ELF64 parser, and one foreground ELF process path.
- `kernel/src/apps/`: built-in apps: `notes`, `calc`, and `sysinfo`.
- `kernel/src/museum.rs`, `quest.rs`: educational shell surfaces and RPG-style capability tracker.
- `boot/stage1/stage1.asm`: custom BIOS sector-0 boot stage.
- `boot/stage2/stage2_real.asm`: active custom BIOS Stage 2 loader and long-mode transition.
- `boot/stage2/stage2_long.asm`: reference long-mode handoff, not the active flat Stage 2 path.
- `boot/stage2/stage2_pm.rs`: typed helper/layout documentation for Stage 2 manifest and boot info.
- `uefi-loader/src/main.rs`: Rust UEFI application that loads `\CHRONO\KERNEL.ELF`, configures GOP, exits Boot Services, and jumps to the kernel.
- `tools/chronofs_put.rs`: host tool to create/update the ChronoFS data image and inject files.
- `tools/custom_image_builder.rs`: packages custom BIOS image and patches Stage 2 manifest.
- `tools/uefi_image_builder.rs`: creates GPT/FAT32 ESP image containing `BOOTX64.EFI` and `KERNEL.ELF`.
- `user/hello.c`, `user/user.ld`: freestanding user-space ELF demo for `exec hello.elf`.

## 6. Features Already Implemented In Code

These are code-present but not runtime-verified in this shell:

- BIOS boot image path through root `build.rs` and `bootloader` crate.
- Optional custom BIOS boot path with Stage 1, Stage 2, image builder, and `chrono_custom_entry`.
- Optional UEFI path with a Rust `BOOTX64.EFI`, GOP framebuffer handoff, ACPI RSDP handoff, GPT/FAT32 image builder, and QEMU run script.
- COM1 serial logging.
- Framebuffer console with top bar, scrollable shell text region, bitmap font, RGB/BGR/U8 pixel handling, scanline/chunky/smooth/thin effects, and mouse cursor drawing.
- Era profiles for 1984, 1995, 2007, and 2040.
- PC speaker boot chimes and `beep <hz>`.
- GDT/TSS setup with double-fault and ring 0 stacks.
- IDT handlers for breakpoint, general protection fault, page fault, double fault, timer IRQ, keyboard IRQ, and mouse IRQ.
- Legacy PIC remap and PIT 100 Hz tick counter.
- PS/2 keyboard IRQ1 buffering with polling fallback and ASCII decoding.
- PS/2 mouse initialization, IRQ12 packet decoding, click/move event publication.
- Shell commands for help/about/clear/reboot/era/uptime/clock/mem/cores/beep/ring3/syshello/files/exec/windows/tasks/network/museum/quests/apps.
- Built-in apps: one-line persistent notes, integer calculator, era-styled sysinfo.
- Tiny framebuffer window manager for notes and sysinfo windows, including dragging, close button, bring-to-front, and era styling.
- Cooperative scheduler with fixed task slots and task stacks; `tasks` and `kill <id>` shell commands.
- SMP discovery and AP startup using ACPI MADT and INIT-SIPI-SIPI, with fallback to one core if discovery/setup fails.
- Spinlock primitive for shared kernel state.
- Reusable free-list heap allocator with 1 MiB heap at `0x200000`, block splitting, free reinsertion, and adjacent-block coalescing.
- ATA PIO read/write of the primary slave IDE disk.
- ChronoFS disk format with superblock, file table, bitmap, contiguous file sectors, and heap fallback if disk mount fails.
- File shell commands: `ls`, `cat <name>`, `write <name> <content>`, `rm <name>`, `exec <name>`.
- RTL8139 PCI network initialization, static IPv4 settings, ARP, UDP send, polling receive path, and serial RX logging.
- Ring 3 privilege demo via `ring3`.
- SYSCALL/SYSRET setup with `sys_write`, `sys_read`, `sys_exit`, and `sys_uptime`.
- Static ELF64 parser and foreground ELF execution from ChronoFS.
- `build-user.ps1` path to compile/install `hello.elf` into the data disk.
- Museum and quest shell content using era profile text frames.

## 7. Features Partially Implemented Or Limited

- Runtime behavior is not verified in this environment because Rust/QEMU/PowerShell tools are unavailable on `PATH`.
- Keyboard input has code-present IRQ1 buffering plus a polling fallback, but runtime IRQ delivery and typing behavior are not verified.
- Mouse input is interrupt-driven in code, but the README still says mouse support is not part of the milestone.
- Heap allocation uses a code-present reusable free list, but allocation reuse and coalescing are not runtime-verified.
- ChronoFS has no journal or repair tool. Interrupted writes/removes can corrupt metadata.
- ChronoFS stores fixed-size metadata and contiguous file extents only. Current documented limits include 64 file slots, 32-byte filenames, no whitespace in filenames, one shell-line content via shell, and 64 KiB max file size.
- Networking is ARP/UDP only, uses static IPv4, and has no DHCP, TCP, DNS, checksum-complete UDP stack, or real-hardware support.
- UEFI real-hardware boot needs verification. Docs say Secure Boot must be disabled unless signing is added; USB HID/storage/serial are future work.
- Custom BIOS handoff v1 is still supported but lacks `rsdp_addr`; SMP ACPI discovery can fall back to single core on that path.
- Process support is early. There is one foreground static ELF path, no dynamic linker, no libc, no argv/env, no process scheduler integration, no general multi-process model.
- `syshello` is an older fixed-page demo and is separate from the newer ELF exec path.
- Scheduler is cooperative, not preemptive. Timer preemption, live migration, CPU hotplug, IOAPIC, and x2APIC are not implemented.
- Several docs are behind source reality, especially README structure/current-state text.

## 8. Planned But Not Started

Based on code/docs/quest data:

- ChronoFS journaling, crash recovery, repair tooling, or reusable free-space policy improvements.
- Richer graphics shell beyond the current tiny window manager.
- Broader real-hardware support, especially USB HID, USB storage, USB serial, and non-QEMU networking.
- More app-like shell programs after kernel basics stay stable.

## 9. Current Build, Run, And Test Commands

Commands documented or present in scripts:

```powershell
cargo build -p kernel
.\scripts\build.ps1
.\scripts\run.ps1
.\scripts\debug-serial.ps1
.\scripts\build-custom.ps1
.\scripts\run-custom.ps1
.\scripts\build-uefi.ps1
.\scripts\run-uefi.ps1
.\scripts\build-user.ps1
```

Optional direct command from README:

```powershell
$hostTarget = ((rustc -vV | Select-String "^host:").ToString() -split " ")[1]
cargo build -p chronosapien --target $hostTarget
qemu-system-x86_64 -smp 2 -drive format=raw,file=target\x86_64-unknown-none\debug\chronosapien-bios.img,if=ide,index=0,media=disk -drive format=raw,file=target\x86_64-unknown-none\debug\chronofs-data.img,if=ide,index=1,media=disk -serial stdio
```

Networking test documented in `docs/networking.md`:

```powershell
ncat -ul 9000
```

Then boot and run:

```text
net arp
net send
```

Storage smoke test documented in `docs/storage.md`:

```text
write hello.txt Hi there
reboot
cat hello.txt
```

There are no Rust unit tests found in the tracked source by search for `#[test]` or `mod tests`.

## 10. Current Known Errors, Warnings, Or Broken Areas

Verification attempted in this shell:

- `git status --branch --short` before export: `## persistent-chronofs-storage...origin/persistent-chronofs-storage`.
- The working tree had no modified or untracked files before creating `PROJECT_EXPORT.md`.
- `git diff --check` passed with no output.
- `cargo`, `rustc`, `qemu-system-x86_64`, and `pwsh` are not on `PATH`, so build/run/test verification could not be performed here.

Observed command failures:

```text
cargo not found
rustc not found
qemu-system-x86_64 not found
pwsh not found
```

Known repo issues:

- `Cargo.lock` is stale/inconsistent with current manifests and workspace membership.
- README's file tree and some milestone language are stale relative to current source files/features.
- Runtime correctness of BIOS boot, custom BIOS boot, UEFI boot, SMP, storage persistence, networking, mouse/window interaction, syscalls, and ELF execution needs verification on a machine with Rust/QEMU/PowerShell tooling.
- The code contains many expected `unsafe` blocks because this is kernel/device/boot code. This is not automatically a bug, but it raises review risk.
- Some source comments still describe old assumptions, for example `keyboard.rs` says the early kernel is single-core/non-preemptive even though SMP and a cooperative scheduler now exist.

## 11. Important Architecture Decisions Already Made

- Keep early boot approachable by using the `bootloader` crate as the normal BIOS boot image path.
- Keep optional ownership paths in-repo: custom BIOS Stage 1/Stage 2 and a Rust UEFI loader.
- Use a shared kernel-side `BootContext` abstraction for bootloader crate and custom/UEFI handoffs.
- Use framebuffer-provided graphics rather than implementing a GPU/VESA driver in the kernel.
- Use COM1 serial as the reliable debug path.
- Keep device drivers small and explicit with direct port I/O helpers.
- Use legacy PC-compatible devices in QEMU first: PS/2 keyboard/mouse, PIC, PIT, ATA PIO IDE, RTL8139.
- Use identity mapping heavily in early memory work to keep physical/virtual addresses easy to inspect.
- Start memory allocation with a fixed 1 MiB bump heap.
- Keep ChronoFS intentionally simple: fixed metadata layout, 512-byte sectors, contiguous extents, no journal.
- Keep networking intentionally small: static IPv4, ARP, UDP, polling receive.
- Use ring 3 demos and syscalls to teach privilege separation before building a full process model.
- Use fixed-capacity arrays and simple limits throughout kernel UI/scheduler/filesystem paths.
- Use era profiles as data that drives visual/sound/text presentation instead of scattering theme-specific branches.

## 12. What Changed Recently

Recent commits on `persistent-chronofs-storage`:

```text
205dc9a Use era profile quest frames
9ba691f Use era profile museum frames
f733bc0 Format sysinfo from era profiles
c66e057 Draw windows from era profiles
c8fae10 Use era profile cursor timing
72396f5 Drive boot sounds from era profiles
09fbabf Render profile-driven framebuffer effects
99005ce Expand era presentation profiles
3f94410 Update README for UEFI support
5b3a49e Document BIOS handoff compatibility
12a54c0 Describe UEFI boot flow
b2844a1 Document UEFI boot path
016bf88 Add guarded UEFI USB writer
afeae2f Add UEFI QEMU run script
ca8acc5 Add UEFI image build script
cc96103 Add UEFI ESP image builder
d24f1d0 Extend custom boot handoff to v2
51db57e Implement UEFI kernel loader
8f803bd Add UEFI loader crate
53f1a12 Add UEFI target to workspace
c6febdb Update README for SMP
e72a6ee Document SMP architecture
13371df Run QEMU with two CPUs
93c728a Add cores shell command
5ab976b Wire SMP into boot
3be64da Make scheduler SMP-aware
a0d1f2f Add per-core descriptor tables
b79231e Add SMP discovery and AP startup
8ca706e Expose ACPI and SMP boot memory
c17c650 Add SMP spinlock primitive
```

Branch-vs-main scope inspected earlier: this branch adds or modifies substantial OS feature work across storage, UEFI, user ELF loading, SMP, syscalls, networking docs/scripts/tools, frame rendering, themes, and shell/UI features. It is not a small documentation-only branch.

## 13. What Should Be Built Next

Recommended next engineering sequence:

1. Restore local build tooling and regenerate/validate `Cargo.lock`.
2. Run compile checks for `kernel`, root BIOS image build, and `uefi-loader`.
3. Fix compile errors only, if any, without refactoring unrelated code.
4. Boot QEMU BIOS path and verify baseline shell, serial logs, framebuffer, keyboard, timer, filesystem mount/fallback, and clean shutdown/reboot behavior.
5. Verify newer risky surfaces one at a time: SMP, mouse/window manager, scheduler tasks, ChronoFS persistence, RTL8139 ARP/UDP, `syshello`, and `exec hello.elf`.
6. Update README/docs to match the current source tree and shell commands.
7. Verify IRQ1 keyboard buffering, including shifted characters, Backspace, Enter, and the polling fallback.
8. Verify reusable heap allocation with repeated open/close, file, app, and task workflows before adding more kernel behavior.

## 14. Recommended Next 5 Codex Prompts

1. "Run the build checks for Chronosapian, fix only compile errors, and do not refactor unrelated code."
2. "Update README/docs to match the current source tree and implemented shell commands."
3. "Verify IRQ1 keyboard input in QEMU while preserving the existing polling fallback."
4. "Exercise reusable heap allocation paths and document any allocator limits or failures."
5. "Design and implement ChronoFS journaling or a repair tool for interrupted writes."

## 15. Risks, Confusing Areas, And Cleanup Needs

- Naming is inconsistent: `Chronosapian`, `chronosapien`, and `ChronoOS` all appear as project/OS identifiers.
- `Cargo.lock` likely blocks reproducible builds until regenerated.
- Docs are useful but cannot be trusted as complete truth. Source files show newer features than README describes.
- Many major features are code-present but unverified in this shell. Avoid claiming runtime success until QEMU evidence exists.
- SMP plus global `UnsafeCell` state needs careful review. Some comments still assume single-core behavior.
- The cooperative scheduler, window manager, mouse IRQ path, framebuffer writer, and serial output all interact with interrupts and shared state; race/reentrancy review is important.
- `ring3` intentionally enters a loop after handling the privileged instruction demo; this is expected demo behavior but can surprise users.
- UEFI loader sets `physical_memory_offset` to `0` and builds identity maps; kernel memory code requires a physical memory offset and treats `Some(0)` as valid. This needs runtime verification on UEFI.
- ATA path assumes QEMU primary slave IDE disk. UEFI and real hardware may not provide that, so ChronoFS may fall back to heap.
- Networking assumes RTL8139 on PCI bus 0 and QEMU user-mode network defaults.
- No automated tests are present. Current verification is script/manual/QEMU based.
