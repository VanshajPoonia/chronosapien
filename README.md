# ChronoOS

ChronoOS is a tiny `no_std` Rust kernel that writes text directly to the VGA
text buffer.

## Build

Install the required tools:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
brew install qemu
cargo install bootimage
rustup component add llvm-tools-preview
```

Build the bootable image:

```sh
cargo bootimage
```

Run it in QEMU:

```sh
qemu-system-x86_64 -drive format=raw,file=target/x86_64-time_capsule_os/debug/bootimage-chrono_os.bin
```

## VGA Text Buffer

VGA text mode gives the kernel a small screen buffer at memory address
`0xb8000`. Each visible character uses two bytes:

- one byte for the ASCII character
- one byte for the color

Writing those bytes into the buffer makes text appear on the screen. The
`vga` module wraps that hardware detail so the kernel can use `print!` and
`println!`.
