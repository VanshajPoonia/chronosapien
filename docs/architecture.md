# Architecture Notes

Chronosapian is split into **our code** and **borrowed infrastructure** on purpose.

## Ours

- The kernel entrypoint in `kernel/src/main.rs`
- The panic path in `kernel/src/panic.rs`
- The serial logger in `kernel/src/serial.rs`
- The public console layer in `kernel/src/console.rs`
- The polling keyboard reader in `kernel/src/keyboard.rs`
- The framebuffer text renderer in `kernel/src/framebuffer/`
- The ATA PIO disk driver and ChronoFS filesystem
- The era model in `kernel/src/theme.rs`
- The startup welcome message
- The PowerShell helper scripts and documentation

## Borrowed infrastructure

- The `bootloader` crate, which packages the kernel into a BIOS image
- The `bootloader_api` crate, which provides boot information to the kernel
- QEMU, which emulates the machine we are developing against
- The Rust compiler and core library

## Why this split is good for learning

We keep the early boot plumbing borrowed so you can focus on kernel code first. That keeps Milestone 1 centered on a few core ideas: entrypoints, console output, panic handling, and the basic shape of a bare-metal Rust crate.

## COM1 serial logging

QEMU provides a virtual 16550-compatible serial device at COM1 port `0x3F8`.
When QEMU runs with `-serial stdio`, bytes written to that port appear in the
host terminal. Chronosapian uses that path for early boot logs and panic
messages because it still works when framebuffer output is hard to inspect.

## Framebuffer graphics

The bootloader asks the firmware for a linear framebuffer and passes its
address, dimensions, stride, bytes-per-pixel, and pixel format to the kernel.
Chronosapian writes pixels directly into that memory. RGB and BGR layouts store
the same red, green, and blue color channels in different byte orders, so the
renderer checks the bootloader metadata before writing each pixel.

Text is rendered with a tiny 8x8 bitmap font. Each glyph is eight bytes: one
byte per row, with set bits producing foreground pixels. The console keeps a
small text cell buffer so shell output can scroll below the top bar without
overwriting it.

## CPU exception handling

Chronosapian loads its own Global Descriptor Table (GDT) during early boot. In
x86_64 long mode, old-style segmentation is mostly disabled, but the CPU still
needs a valid code segment. The GDT also contains the Task State Segment (TSS),
which gives the CPU a dedicated stack to use for especially dangerous
exceptions.

The Interrupt Descriptor Table (IDT) is the CPU's exception and interrupt
vector table. Chronosapian registers Rust handlers for breakpoint, page fault, and
double fault entries, then loads the table with `lidt`. These handlers use
Rust's `extern "x86-interrupt"` ABI so the compiler preserves the stack layout
that the CPU expects.

A double fault happens when the CPU encounters a second exception while trying
to deliver or handle an earlier one. If the normal stack is already corrupted,
handling the double fault on that same stack can immediately become a triple
fault and reset the machine. Chronosapian assigns double faults a separate TSS
Interrupt Stack Table entry so the handler has a clean stack to report the
failure.

## PIT timer and PIC remapping

Chronosapian uses the legacy Programmable Interval Timer (PIT) for a simple uptime
clock. The PIT runs from a fixed input frequency of 1,193,182 Hz. Programming
channel 0 with a divisor turns that input clock into periodic IRQ0 interrupts;
for the current 100 Hz timer, the divisor is `1_193_182 / 100`.

Hardware IRQs arrive through the legacy 8259 Programmable Interrupt Controller
(PIC). By default, the PIC's IRQ vectors overlap the CPU exception vectors,
which occupy IDT entries 0 through 31. Chronosapian remaps the master PIC to start
at vector 32 and the slave PIC to start at vector 40, so IRQ0 becomes IDT vector
32 instead of colliding with an exception.

The timer path is intentionally small: PIT channel 0 fires IRQ0, the remapped
PIC forwards that as vector 32, the IDT dispatches the timer handler, the
handler increments an atomic tick counter, and then it sends an end-of-interrupt
command back to the PIC. The handler does not print per tick, because serial
and framebuffer output are not interrupt-safe yet.

## Basic memory management

Chronosapian reads the bootloader's memory map to learn which physical RAM ranges
exist and which ranges are safe for the kernel to use. A physical frame is a
fixed-size chunk of physical RAM; this kernel starts with 4KiB frames because
that is the default granularity used by x86_64 page tables.

The kernel's physical frames and the first heap are identity mapped, meaning a
virtual address such as `0x200000` points to physical address `0x200000`.
Identity mapping is the safest starting point here because the addresses
printed by the kernel match the hardware addresses being used, which keeps
early debugging straightforward.

The heap uses a bump allocator. A bump allocator keeps one pointer to the next
free byte and moves it forward for each allocation. That makes it tiny and
predictable, but freeing memory is a no-op, so used heap space never comes back
until the kernel grows a more complete allocator.

## Persistent storage

Chronosapian attaches a separate QEMU IDE data disk for shell files. The kernel
talks to that disk with ATA PIO on the primary IDE channel: commands and status
go through ports `0x1F0..0x1F7`, and sector data is copied as 256 16-bit words
per 512-byte sector.

The disk format is ChronoFS. Sector 0 is a superblock with the magic number,
layout, file count, and checksum. The next sectors hold a fixed file table and
a free-sector bitmap. File data is stored in contiguous sector runs. This is
easy to inspect, but it has no journal, so an interrupted write can leave
metadata inconsistent.

## What still hides low-level behavior

- `bootloader` still hides the CPU mode switch, stack setup, paging setup, and the exact boot handoff details.
- `bootloader` still hides host-side image packaging, but it does not hide the runtime behavior of the kernel inside QEMU.

## What to replace later for more ownership

If you want to own more of the boot path later, replace pieces in this order:

1. Replace the `BootInfo`-based handoff with your own chosen boot protocol.
2. Replace the borrowed bootloader with a more explicit protocol such as Multiboot2 or Limine.
3. Replace that protocol with your own bootloader stages and long-mode entry path.
