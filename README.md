# Time Capsule OS

Time Capsule OS is a beginner-friendly hobby operating system project in Rust. The project is organized for learning, not cleverness: small files, focused modules, minimal dependencies, and comments that explain the risky parts.

## Folder structure

```text
time-capsule-os/
├─ Cargo.toml
├─ rust-toolchain.toml
├─ .cargo/
│  └─ config.toml
├─ kernel/
│  ├─ Cargo.toml
│  └─ src/
│     ├─ main.rs
│     ├─ panic.rs
│     ├─ theme.rs
│     ├─ serial.rs
│     └─ vga_text/
│        ├─ mod.rs
│        ├─ color.rs
│        └─ writer.rs
├─ scripts/
│  ├─ build.ps1
│  ├─ run.ps1
│  └─ debug-serial.ps1
├─ docs/
│  ├─ roadmap.md
│  ├─ architecture.md
│  └─ boot-flow.md
├─ README.md
└─ .gitignore
```

## What each file is for

- `Cargo.toml` sets up the Rust workspace and keeps the root simple.
- `rust-toolchain.toml` pins a nightly Rust toolchain with the components needed for bare-metal builds.
- `.cargo/config.toml` sets the default target for bare-metal builds.
- `kernel/Cargo.toml` defines the kernel crate and its intentionally small dependency set.
- `kernel/src/main.rs` is the kernel entrypoint and first boot flow.
- `kernel/src/keyboard.rs` polls the PS/2 controller and decodes a tiny set of keyboard scancodes.
- `kernel/src/panic.rs` handles panics in a `no_std` environment.
- `kernel/src/shell.rs` runs the fixed-size command buffer and built-in shell commands.
- `kernel/src/theme.rs` holds era profiles for `1980s`, `1990s`, `2000s`, and `future`.
- `kernel/src/serial.rs` sends debug text to QEMU's emulated serial port.
- `kernel/src/vga_text/color.rs` defines VGA text colors.
- `kernel/src/vga_text/writer.rs` writes characters directly into VGA text memory.
- `kernel/src/vga_text/mod.rs` exposes the small public text-output surface and print macros.
- `scripts/build.ps1` builds the bootable image.
- `scripts/run.ps1` runs the image in QEMU with VGA and serial output.
- `scripts/debug-serial.ps1` runs QEMU in a serial-only mode for debugging.
- `docs/roadmap.md` lists the next milestones in order.
- `docs/architecture.md` explains what code is ours versus borrowed.
- `docs/boot-flow.md` explains the startup path in plain language.

## Dependencies and why they exist

- `bootloader`
  Borrowed infrastructure for the very first version. It loads the kernel and jumps into our Rust entrypoint so we can focus on kernel development instead of writing a bootloader first.

That is the only runtime dependency in the kernel now. Serial output and VGA text output are implemented directly in this project so the code stays closer to the hardware.

## Current milestone: keyboard input and tiny shell

The kernel now polls the classic PS/2 controller directly:

- status port `0x64`
- data port `0x60`

It decodes a small, readable subset of keyboard set-1 scancodes and feeds them into a fixed 64-byte command buffer. The shell supports:

- `help`
- `clear`
- `about`

This is intentionally a polling shell, not an interrupt-driven terminal yet. When interrupts arrive later, the keyboard path should move behind an IRQ-based input queue and the current global mutable state will need stronger synchronization.

## Exact setup commands

Install Rust and the required components:

```powershell
winget install Rustlang.Rustup
rustup toolchain install nightly
rustup default nightly
rustup target add x86_64-unknown-none
rustup component add rust-src llvm-tools-preview
```

Install the build helpers:

```powershell
cargo install bootimage
```

Install QEMU:

```powershell
winget install qemu
```

## Exact build and run commands

Build:

```powershell
.\scripts\build.ps1
```

Run:

```powershell
.\scripts\run.ps1
```

Serial-only debug run:

```powershell
.\scripts\debug-serial.ps1
```

Optional direct commands:

```powershell
cargo bootimage -p kernel
qemu-system-x86_64 -drive format=raw,file=target\x86_64-unknown-none\debug\bootimage-kernel.bin -serial stdio
```

## Boot flow in plain language

QEMU emulates an x86_64 machine and boots a disk image. The borrowed `bootloader` crate does the hard early boot work we are intentionally skipping in v0, then jumps into our Rust kernel entrypoint. Our code starts in `kernel/src/main.rs`, initializes serial output, configures VGA text output based on the selected era, prints `Welcome to Time Capsule OS`, and then enters a polling shell loop. The shell reads keyboard data from the PS/2 controller, appends characters to a fixed command buffer, and runs simple built-in commands when you press Enter.

## What to build next after the first shell

1. Add an `era` command that switches the prompt, palette, and banner at runtime.
2. Move keyboard handling from polling to interrupt-driven input once IRQ support exists.
3. Add a timer and basic interrupt setup so the kernel stops being purely synchronous.
4. Grow the shell with a few more commands only after the input path feels solid.

## What is ours and what is borrowed

**Ours**

- Kernel entry and startup flow
- Panic handling
- VGA text output
- Theme and era model
- Startup banner and prompt preview
- Scripts and docs

**Borrowed**

- The bootloader infrastructure
- QEMU
- Rust toolchain support for bare-metal builds

That split is deliberate: the early boot path is borrowed so the kernel itself can stay raw, readable, and educational.

## If you want to own more of the boot process later

Right now the biggest borrowed piece is the `bootloader` crate. Replacing it later would mean taking responsibility for:

1. Building a bootable disk image or boot sector yourself.
2. Entering 64-bit long mode.
3. Setting up paging and stack state before Rust starts.
4. Loading the kernel binary from disk or from a boot protocol handoff.
5. Defining your own kernel entry ABI instead of using `BootInfo`.

A good learning order is:

1. Keep the current kernel and own more device code first.
2. Then replace the boot handoff with a more explicit protocol such as Limine or Multiboot.
3. Finally write your own first-stage and second-stage boot path when you want full ownership.
