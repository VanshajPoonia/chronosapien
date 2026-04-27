# Time Capsule OS

Time Capsule OS is a beginner-friendly hobby operating system project in Rust. Milestone 1 keeps things intentionally small: boot a Rust kernel in QEMU and print one welcome message.

## Folder structure

```text
time-capsule-os/
|-- Cargo.toml
|-- rust-toolchain.toml
|-- .cargo/
|   `-- config.toml
|-- kernel/
|   |-- Cargo.toml
|   `-- src/
|       |-- main.rs
|       |-- panic.rs
|       |-- serial.rs
|       |-- theme.rs
|       `-- vga_text/
|           |-- color.rs
|           |-- mod.rs
|           `-- writer.rs
|-- scripts/
|   |-- build.ps1
|   |-- debug-serial.ps1
|   `-- run.ps1
|-- docs/
|   |-- architecture.md
|   |-- boot-flow.md
|   `-- roadmap.md
`-- .gitignore
```

## What each file is for

- `Cargo.toml` sets up the Rust workspace.
- `rust-toolchain.toml` pins the nightly toolchain and required components.
- `.cargo/config.toml` sets the default build target to `x86_64-unknown-none`.
- `kernel/Cargo.toml` defines the kernel crate and its dependency on `bootloader`.
- `kernel/src/main.rs` is the kernel entrypoint and first boot flow.
- `kernel/src/panic.rs` handles panics in a `no_std` environment.
- `kernel/src/serial.rs` writes debug text to QEMU's emulated COM1 port.
- `kernel/src/theme.rs` defines simple era profiles for startup colors.
- `kernel/src/vga_text/` contains the minimal VGA text writer used for screen output.
- `scripts/build.ps1` builds the bootable disk image.
- `scripts/run.ps1` runs the image in QEMU.
- `scripts/debug-serial.ps1` runs QEMU with display disabled and serial output enabled.
- `docs/roadmap.md` lists Milestone 1 and the next steps.
- `docs/architecture.md` explains what code is ours and what is borrowed.
- `docs/boot-flow.md` explains the startup path in plain language.

## Dependency

- `bootloader`
  Borrowed infrastructure for the first version. It loads the kernel and jumps into our Rust entrypoint so we can focus on kernel development before writing our own bootloader.

That is the only dependency in the kernel. VGA text output and serial output are implemented directly in this repo.

## Current milestone

The kernel currently does four things:

- boots through the borrowed `bootloader` crate,
- initializes serial output,
- initializes VGA text output with a fixed era profile,
- prints `Welcome to Time Capsule OS` and halts.

That small scope is deliberate. It gives us a clean baseline before adding input, interrupts, memory management, or a shell.

## Exact setup commands

Install Rust and the required components:

```powershell
winget install Rustlang.Rustup
rustup toolchain install nightly
rustup component add rust-src llvm-tools-preview --toolchain nightly
rustup target add x86_64-unknown-none --toolchain nightly
```

Install the boot image helper:

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
cargo build -p kernel
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

QEMU emulates an x86_64 machine and boots a disk image. The borrowed `bootloader` crate performs the early machine setup we are intentionally skipping for now, then jumps into our Rust kernel entrypoint. Our code starts in `kernel/src/main.rs`, initializes serial output, configures VGA text output from the selected era profile, prints `Welcome to Time Capsule OS`, and then halts.

## What to build next

1. Add keyboard input.
2. Add a tiny shell.
3. Set up interrupts and a timer.
4. Add memory-management pieces once the text-only boot path feels comfortable.

## What is ours and what is borrowed

Ours:
- Kernel entry and startup flow
- Panic handling
- VGA text output
- Theme and era model
- Startup welcome message
- Scripts and docs

Borrowed:
- The `bootloader` crate
- QEMU
- Rust bare-metal toolchain support

That split is deliberate: the early boot path is borrowed so the kernel itself can stay raw, readable, and educational.
