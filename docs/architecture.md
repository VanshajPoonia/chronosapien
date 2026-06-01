# Architecture Notes

ChronoOS is split into **our code** and **borrowed infrastructure** on purpose.
The public/product name is ChronoOS; the repo/package name may remain
`chronosapien`.

## Status Labels

- implemented in code: present in the source tree.
- partially implemented: useful teaching version, not a complete production system.
- needs runtime verification: must be proven in QEMU or hardware before success is claimed.
- roadmap/design-only: intentionally not implemented yet.

## Ours

- Kernel entry and startup flow in `kernel/src/main.rs`.
- Panic handling, serial logging, framebuffer text output, and bitmap font rendering.
- GDT, IDT, PIC, PIT, exception handlers, timer IRQ, keyboard IRQ, and mouse IRQ paths.
- PS/2 keyboard decoding with IRQ buffering and polling fallback.
- PS/2 mouse packet decoding and fixed-capacity window interactions.
- Basic memory map handling, page mapping helpers, and a 1 MiB free-list heap.
- ATA PIO storage, ChronoFS, `fsck`, conservative repair, and a tiny journal.
- Era model, museum pages, quests, shell commands, small apps, and product/demo commands.
- Cooperative scheduler, early SMP work, ring 3 demo, syscall layer, static ELF execution, and ARP/UDP networking.
- PowerShell helper scripts and documentation.

## Borrowed Infrastructure

- The `bootloader` crate packages the kernel into a BIOS image.
- The `bootloader_api` crate provides boot information to the kernel.
- The `uefi` crate supports the UEFI loader.
- The `x86_64` crate provides low-level CPU structures and helpers.
- QEMU emulates the development machine.
- Rust bare-metal toolchain support provides `core`, `alloc`, and target support.

## Console, Serial, And Framebuffer

Status: implemented in code, needs runtime verification.

ChronoOS writes debug logs to COM1 at port `0x3F8` and draws text directly into
the linear framebuffer provided by the boot path. The console keeps a text cell
buffer so shell output can scroll below the persistent top bar.

The framebuffer path is not a GPU driver. BIOS/UEFI/bootloader code provides
pixel memory, and the kernel writes pixels using the reported width, height,
stride, bytes-per-pixel, and RGB/BGR format.

## Input And Windows

Status: implemented in code, needs runtime verification.

Keyboard input has an IRQ1 event buffer and a polling fallback through the PS/2
controller. Mouse input has an IRQ12 packet path that publishes simple events to
the window manager. The window manager is fixed-capacity and supports small
notes/sysinfo windows, dragging, focus order, and close buttons.

This is partially implemented as a teaching UI. It is not a full desktop,
compositor, or GUI toolkit.

## CPU Exceptions, Timer, And Sound

Status: implemented in code, needs runtime verification.

ChronoOS loads a GDT and IDT, handles breakpoint, page fault, double fault, and
general protection fault paths, and uses the PIT at 100 Hz for ticks and uptime.
The PIC is remapped so hardware IRQs do not collide with CPU exceptions. PIT
channel 2 is also used for simple PC speaker tones.

## Memory

Status: implemented in code, needs runtime verification.

ChronoOS reads the boot memory map, identity maps early kernel/heap pages, maps
fixed ring 3 demo pages, and provides helpers for user address spaces. The heap
starts at `0x200000`, is 1 MiB, and uses a simple free-list allocator with
splitting, deallocation, address-sorted reinsertion, and coalescing.

This allocator is no longer bump-only, but reuse behavior still needs runtime
verification under real file, app, task, window, and ELF workflows.

## Persistent Storage

Status: implemented in code, needs runtime verification.

ChronoOS attaches a second QEMU IDE data disk for ChronoFS. The kernel talks to
that disk with ATA PIO. ChronoFS stores a superblock, fixed file table, free
sector bitmap, contiguous file extents, and a small heap cache for shell reads.

`fsck`, `fsck repair`, and a tiny one-record journal are implemented in code.
Repair is intentionally conservative and journal recovery refuses unsafe
records. Crash recovery still needs controlled runtime verification.

## Userspace, Syscalls, And ELF

Status: partially implemented, needs runtime verification.

The `ring3` command demonstrates an opt-in transition to user mode. The
`syshello` path uses SYSCALL/SYSRET for a tiny syscall demo. The `exec <name>`
path reads a static ELF64 file from ChronoFS, validates a small `PT_LOAD`
subset, maps a foreground user address space, and enters ring 3.

There is no dynamic linker, argv/env, general process table, permissions model,
or mature lifecycle management yet.

## SMP And Scheduling

Status: partially implemented, needs runtime verification.

ChronoOS can discover CPUs through ACPI MADT, start APs through INIT-SIPI-SIPI,
and run a cooperative scheduler with fixed task slots. The shell stays on core
0 while tasks can be assigned to online cores.

Preemption, IOAPIC routing, x2APIC, CPU hotplug, blocking I/O integration, and
production-grade scheduling are roadmap/design-only.

## Networking

Status: partially implemented, needs runtime verification.

ChronoOS targets QEMU's RTL8139 NIC and implements a small static IPv4 path with
ARP and UDP. There is no DHCP, DNS, TCP, socket API, broad driver support, or
real-hardware network support yet.
