# Chronosapian

Chronosapian is a beginner-friendly hobby operating system project in Rust. It boots
a `no_std` x86_64 kernel in QEMU, renders a framebuffer graphics console, logs
to serial, runs a tiny era-themed shell, handles CPU exceptions and timer
interrupts, and now has early memory management, persistent ATA-backed storage,
networking, and a few built-in apps.

## Folder structure

```text
chronosapien/
|-- Cargo.toml
|-- build.rs
|-- boot/
|   |-- stage1/
|   |   `-- stage1.asm
|   `-- stage2/
|       |-- stage2_long.asm
|       |-- stage2_pm.rs
|       `-- stage2_real.asm
|-- rust-toolchain.toml
|-- src/
|   `-- main.rs
|-- .cargo/
|   `-- config.toml
|-- kernel/
|   |-- Cargo.toml
|   `-- src/
|       |-- apps/
|       |   |-- calc.rs
|       |   |-- mod.rs
|       |   |-- notes.rs
|       |   `-- sysinfo.rs
|       |-- ata.rs
|       |-- console.rs
|       |-- framebuffer/
|       |   |-- font.rs
|       |   `-- mod.rs
|       |-- fs.rs
|       |-- gdt.rs
|       |-- interrupts.rs
|       |-- io.rs
|       |-- keyboard.rs
|       |-- main.rs
|       |-- memory.rs
|       |-- net.rs
|       |-- panic.rs
|       |-- pci.rs
|       |-- pic.rs
|       |-- serial.rs
|       |-- shell.rs
|       |-- theme.rs
|       `-- timer.rs
|-- scripts/
|   |-- build.ps1
|   |-- build-custom.ps1
|   |-- debug-serial.ps1
|   |-- run-custom.ps1
|   `-- run.ps1
|-- docs/
|   |-- architecture.md
|   |-- boot-flow.md
|   |-- custom-bootloader.md
|   |-- networking.md
|   |-- storage.md
|   `-- roadmap.md
|-- tools/
|   `-- custom_image_builder.rs
`-- .gitignore
```

## What each file is for

- `Cargo.toml` sets up the Rust workspace.
- `build.rs` uses the bootloader 0.11 BIOS image builder to create the bootable image.
- `boot/` contains the optional custom BIOS bootloader stages.
- `src/main.rs` is a small host-side helper that reports the generated image path.
- `rust-toolchain.toml` pins the nightly toolchain and required components.
- `.cargo/config.toml` sets the default kernel target and enables nightly artifact dependencies.
- `kernel/Cargo.toml` defines the kernel crate and its dependencies on `bootloader_api` and `x86_64`.
- `kernel/src/apps/` contains tiny built-in apps for notes, integer math, and system info.
- `kernel/src/ata.rs` reads and writes sectors through QEMU's ATA PIO IDE disk.
- `kernel/src/console.rs` is the beginner-friendly text output layer with `print!` and `println!`.
- `kernel/src/framebuffer/` draws text and the top bar into the boot framebuffer.
- `kernel/src/fs.rs` mounts ChronoFS from disk, with a heap fallback if the disk is missing.
- `kernel/src/gdt.rs` loads the Global Descriptor Table and a TSS with a double-fault stack.
- `kernel/src/interrupts.rs` loads the Interrupt Descriptor Table and handles exceptions plus IRQ0.
- `kernel/src/io.rs` provides shared x86 port I/O helpers for device drivers.
- `kernel/src/keyboard.rs` polls the PS/2 keyboard and turns scancodes into simple key events.
- `kernel/src/main.rs` is the kernel entrypoint and first boot flow.
- `kernel/src/memory.rs` reads the bootloader memory map, identity maps early pages, and provides a bump heap.
- `kernel/src/net.rs` drives QEMU RTL8139 networking with ARP and UDP.
- `kernel/src/panic.rs` handles panics in a `no_std` environment.
- `kernel/src/pci.rs` scans PCI config space for supported devices.
- `kernel/src/pic.rs` remaps the legacy PIC so hardware IRQs start at IDT vector 32.
- `kernel/src/serial.rs` writes debug text to QEMU's emulated COM1 port.
- `kernel/src/shell.rs` runs the line-based command shell.
- `kernel/src/theme.rs` defines era profiles for prompts and framebuffer colors.
- `kernel/src/timer.rs` configures the PIT at 100Hz and tracks ticks.
- `scripts/build.ps1` builds the bootable disk image.
- `scripts/build-custom.ps1` builds the optional custom sector-0 BIOS image.
- `scripts/run.ps1` runs the image in QEMU.
- `scripts/run-custom.ps1` runs the custom BIOS image in QEMU.
- `scripts/debug-serial.ps1` runs QEMU with display disabled and serial output enabled.
- `docs/roadmap.md` lists Milestone 1 and the next steps.
- `docs/architecture.md` explains what code is ours and what is borrowed.
- `docs/boot-flow.md` explains the startup path in plain language.
- `docs/custom-bootloader.md` explains the custom bootloader path.
- `docs/networking.md` explains Ethernet, ARP, IPv4, UDP, and QEMU testing.
- `docs/storage.md` explains ATA PIO, LBA addressing, ChronoFS, and persistence testing.
- `tools/custom_image_builder.rs` packages the custom boot image.

## Dependencies

- `bootloader`
  Host-side disk image builder. It wraps the kernel ELF in a bootable BIOS image.
- `bootloader_api`
  The kernel-facing boot API. It provides the memory map, physical-memory
  offset, and framebuffer metadata.
- `x86_64`
  A `no_std` helper crate for descriptor tables, interrupt stack frames,
  control registers, and page-table types.

Framebuffer text output, serial output, keyboard polling, PIC/PIT setup, the
bump heap, ATA PIO storage, RTL8139 networking, and the tiny apps are
implemented directly in this repo.

## Current State

The kernel currently:

- boots through the borrowed `bootloader` crate,
- initializes COM1 serial logging,
- initializes a framebuffer graphics console with an era-colored top bar,
- loads a GDT and IDT,
- handles breakpoint, page fault, and double fault exceptions,
- remaps the PIC and runs a 100Hz PIT timer interrupt,
- initializes a 1MiB bump heap from the bootloader memory map,
- mounts a tiny ChronoFS disk so shell files and notes survive reboot,
- prints a compact boot banner below the top bar,
- polls the PS/2 keyboard and runs a small command shell,
- supports era switching,
- includes built-in `notes`, `calc`, and `sysinfo` apps,
- detects QEMU's RTL8139 NIC and can send ARP plus UDP packets,
- logs the boot sequence to the QEMU terminal,
- keeps the implementation intentionally small and readable.

That small scope is deliberate. It gives us real kernel pieces while keeping the
code approachable enough to learn from.

## Exact setup commands

Install Rust and the required components:

```powershell
winget install Rustlang.Rustup
rustup toolchain install nightly
rustup component add rust-src llvm-tools-preview --toolchain nightly
rustup target add x86_64-unknown-none --toolchain nightly
```

Install QEMU:

```powershell
winget install qemu
```

Install NASM if you want to build the optional custom bootloader path:

```powershell
winget install NASM.NASM
```

## Exact build and run commands

Build:

```powershell
cargo build -p kernel
.\scripts\build.ps1
```

Run:

```powershell
.\scripts\run.ps1
```

Optional custom bootloader image:

```powershell
.\scripts\build-custom.ps1
.\scripts\run-custom.ps1
```

The custom path starts from our own sector-0 Stage 1 loader. The normal
`chronosapien-bios.img` path remains the fallback.

Serial-only debug run:

```powershell
.\scripts\debug-serial.ps1
```

Graphics mode still requires the bootloader to provide a framebuffer. If a host
QEMU setup refuses to create one with `-display none`, use `.\scripts\run.ps1`
for graphical testing and keep serial output enabled there.

Optional direct commands:

```powershell
$hostTarget = ((rustc -vV | Select-String "^host:").ToString() -split " ")[1]
cargo build -p chronosapien --target $hostTarget
qemu-system-x86_64 -drive format=raw,file=target\x86_64-unknown-none\debug\chronosapien-bios.img,if=ide,index=0,media=disk -drive format=raw,file=target\x86_64-unknown-none\debug\chronofs-data.img,if=ide,index=1,media=disk -serial stdio
```

## Boot Flow in Plain Language

QEMU emulates an x86_64 machine and boots a disk image. The borrowed
`bootloader` crate performs the early machine setup we are intentionally
skipping for now, sets up a 1024x768 framebuffer, then jumps into our Rust
kernel entrypoint. Our code starts in `kernel/src/main.rs`, initializes serial
and framebuffer output, loads descriptor tables, triggers one test breakpoint
exception, initializes memory, starts the timer, prints the startup banner, and
enters the shell.

The graphical console shows a top bar plus shell output:

```text
Chronosapian | Era: 1984 | Uptime: 0s

EXCEPTION: BREAKPOINT
CHRONOSAPIAN
Era: 1984
CHRONO/84> _
```

With `-serial stdio`, the QEMU terminal shows:

```text
[CHRONO] boot start
[CHRONO] serial initialized
[CHRONO] fb: 1024x768 initialized
[CHRONO] console initialized
[CHRONO] GDT loaded
[CHRONO] IDT loaded
[CHRONO] interrupt: breakpoint at 0x...
[CHRONO] breakpoint resolved
[CHRONO] mem: heap initialized at 0x200000 size 1MB
[CHRONO] timer: PIT initialized at 100Hz
[CHRONO] active era: 1984
[CHRONO] keyboard initialized
[CHRONO] boot complete
```

Shell commands and apps add serial lines like:

```text
[CHRONO] cmd: sysinfo
[CHRONO] app: sysinfo launched
[CHRONO] cmd: write hello.txt Hi there
[CHRONO] fs: write hello.txt
[CHRONO] disk: write sector 42
```

## Shell Commands

Built-ins:

- `help` lists available commands.
- `clear` clears the framebuffer shell region and redraws the top bar.
- `about` prints the current Chronosapian version line.
- `reboot` asks the PS/2 controller to reset the machine.
- `era 1984|1995|2007|2040` switches the active era style.
- `uptime` prints elapsed seconds from the PIT tick counter.
- `clock` prints raw PIT ticks.
- `mem` prints total memory, heap location, and used heap space.
- `net` prints RTL8139 MAC, static IP, gateway state, and TX/RX counts.
- `net arp` sends an ARP request for QEMU's `10.0.2.2` gateway.
- `net send [ip port text]` sends a UDP packet.
- `ls` lists files from the mounted ChronoFS disk, or the heap fallback.
- `cat <name>` prints a file's contents.
- `write <name> <content>` creates or overwrites a persistent file.
- `rm <name>` deletes a file.

Tiny apps:

- `notes <text>` stores one short persistent note as `note.txt`.
- `notes read` prints `note.txt`.
- `calc <int> +|-|*|/ <int>` evaluates one integer operation.
- `sysinfo` prints era-styled OS, era, uptime, and memory usage.

## Framebuffer graphics in simple terms

The bootloader asks the firmware for a linear framebuffer and passes its address
and layout to Chronosapian. The kernel does not implement VESA or a GPU driver;
it receives ready-to-write pixel memory from the bootloader.

The framebuffer is a flat byte array. To draw pixel `(x, y)`, the renderer uses:

```text
offset = (y * stride + x) * bytes_per_pixel
```

`stride` is the number of pixels between the start of one scanline and the next.
It can be wider than the visible screen because some framebuffers add padding at
the end of each row. RGB and BGR formats store the same color channels in a
different byte order, so the renderer writes either red-green-blue or
blue-green-red depending on the bootloader metadata.

Text is drawn with a tiny 8x8 bitmap font. Each glyph is stored as eight bytes:
one byte per row. A set bit draws a foreground pixel; a cleared bit draws a
background pixel. The console keeps a small text cell buffer for the shell
region so it can scroll below the persistent top bar. Mouse support is not part
of this milestone.

## COM1 serial output in simple terms

QEMU exposes a virtual 16550 UART at the classic COM1 I/O port address `0x3F8`.
The kernel configures that serial port once during startup. After that, writing
bytes to the COM1 data port sends text to the host terminal when QEMU is run
with `-serial stdio`.

This repo uses small inline `in` and `out` assembly helpers instead of adding a
port I/O crate. That keeps the dependency list short and makes the hardware
access visible while the serial code is still tiny.

## PS/2 keyboard input in simple terms

The PS/2 controller exposes two important I/O ports in QEMU's PC-compatible
machine:

- `0x64` is the status port. The kernel checks it to see whether a keyboard byte is ready.
- `0x60` is the data port. The kernel reads one scancode from it after the status port says data is waiting.

The keyboard module uses a small scancode lookup table for common set-1 keys.
It turns those scancodes into ASCII bytes, backspace, or enter events. The input
buffer is still a fixed-size array on the stack, even though later features now
use the heap.

## Interrupts and timer in simple terms

The GDT gives the CPU valid segment descriptors and a Task State Segment for the
double-fault stack. The IDT tells the CPU which Rust handler to call for
exceptions and interrupts. Chronosapian currently handles breakpoints, page faults,
double faults, and timer IRQs.

The PIT is programmed to fire IRQ0 about 100 times per second. The legacy PIC is
remapped so IRQ0 reaches IDT vector 32 instead of colliding with CPU exception
vectors 0 through 31. Each timer interrupt increments an atomic tick counter and
sends an end-of-interrupt command back to the PIC.

## Memory management in simple terms

The bootloader gives Chronosapian a memory map. The kernel uses it to count total
memory and pick usable 4KiB physical frames. Early pages are identity mapped,
which means a virtual address points to the same physical address. That is the
safest starting point because printed addresses match the hardware addresses
being used.

The heap starts at `0x200000` and is 1MiB. It uses a bump allocator: each
allocation moves a pointer forward, and freeing is a no-op. This is tiny and
predictable, but replacing notes or allocating more objects consumes heap space
until reboot.

## Persistent filesystem in simple terms

Chronosapian now has a tiny disk-backed filesystem named ChronoFS. The run
scripts attach a separate 16 MiB IDE data disk, so the boot image remains
untouched and sector 0 of the data disk can hold filesystem metadata.

The ATA driver uses PIO mode: the CPU waits for the disk, then copies one
512-byte sector at a time through port `0x1F0`. ChronoFS uses LBA numbers, so
the disk is just sector `0`, sector `1`, sector `2`, and so on.

ChronoFS keeps a small heap cache for shell reads, but disk is the source of
truth after mount. If the second disk is missing or broken, the kernel falls
back to the old heap-only file list and logs that choice to serial.

Current limits:

- filenames are at most 32 bytes,
- filenames cannot contain whitespace,
- file contents are one shell line,
- each file can use at most 64 KiB,
- there are 64 file slots,
- no journaling exists yet, so killing QEMU during a write can corrupt files.

## What to build next

1. Add interrupt-driven keyboard input instead of polling.
2. Replace the bump heap with an allocator that can reuse freed memory.
3. Add crash recovery or a journal for disk writes.
4. Add more app-like shell programs after the kernel basics stay stable.

## What is ours and what is borrowed

Ours:
- Kernel entry and startup flow
- Panic handling
- Framebuffer text output and bitmap font rendering
- GDT, IDT, PIC, and PIT setup
- Exception and timer handlers
- Basic memory management and bump allocation
- ATA PIO storage and ChronoFS
- Theme and era model
- Shell commands and tiny built-in apps
- Scripts and docs

Borrowed:
- The `bootloader` crate
- The `x86_64` crate for low-level CPU data structures
- QEMU
- Rust bare-metal toolchain support

That split is deliberate: the early boot path is borrowed so the kernel itself can stay raw, readable, and educational.
