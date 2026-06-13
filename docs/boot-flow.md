# Boot Flow

ChronoOS has three boot paths in the repository. They share the same kernel
startup after handoff, but none should be called runtime-verified until QEMU or
hardware evidence is recorded.

## BIOS Path

Status: implemented in code, verified in QEMU for the normal single-core BIOS
path; custom, UEFI, and SMP/multi-core paths still need separate verification.

1. `.\scripts\build.ps1` builds the `kernel` crate for `x86_64-unknown-none`.
2. The `bootloader` 0.11 BIOS builder wraps that kernel into a disk image.
3. QEMU boots the generated `chronosapien-bios.img` image.
4. The borrowed bootloader performs early CPU setup, framebuffer setup, and the kernel handoff.
5. The kernel enters `kernel/src/main.rs`.
6. ChronoOS initializes serial, framebuffer console, GDT, syscall MSRs, IDT, memory, SMP discovery, ChronoFS, PIC, timer, mouse, networking, scheduler, AP startup, keyboard, and then the shell.

Example display text may still include legacy/internal strings until the source
is fully renamed:

```text
ChronoOS | Era: 1984 | Uptime: 0s

EXCEPTION: BREAKPOINT
CHRONO/84> _
```

Expected serial flow should only be treated as verified after an actual run:

```text
[CHRONO] boot start
[CHRONO] serial initialized
[CHRONO] console initialized
[CHRONO] GDT loaded
[CHRONO] IDT loaded
[CHRONO] mem: heap initialized at 0x200000 size 1MB
[CHRONO] timer: PIT initialized at 100Hz
[CHRONO] keyboard initialized (IRQ buffer ready)
[CHRONO] boot complete
```

## Custom BIOS Path

Status: partially implemented, blocked: build dependency missing.

The optional custom path starts from `boot/stage1/stage1.asm`, loads stage 2,
builds a ChronoOS v1 handoff, enters long mode, and jumps to
`chrono_custom_entry`. This path keeps the normal BIOS image as the fallback and
uses the `chronosapien-custom.img` generated image name.

The 2026-06-02 custom BIOS preflight found that `nasm` was not available on
PATH, so `scripts/build-custom.ps1` and `scripts/run-custom.ps1` were not run.

## UEFI Path

Status: implemented in code, partially verified in QEMU UEFI.

The UEFI build creates a GPT/FAT32 image with:

```text
EFI\BOOT\BOOTX64.EFI
CHRONO\KERNEL.ELF
```

The Rust UEFI loader reads the kernel ELF, allocates load segments, configures
GOP framebuffer output, reads the ACPI RSDP, exits Boot Services, writes the
ChronoOS v2 handoff, loads the new page table, and jumps to
`chrono_custom_entry`.

The 2026-06-13 UEFI build/boot pass fixed the current `uefi` crate API drift
and the UEFI image builder path/FAT sizing blockers. `scripts/build-uefi.ps1`
now builds `target/x86_64-unknown-none/debug/chronosapien-uefi.img` on this
host.

| Step | Status | Evidence | Notes |
| --- | --- | --- | --- |
| UEFI loader build | build fixed, boot not verified | `cargo build -p uefi-loader --target x86_64-unknown-uefi --offline --locked` passed. | Fixed `MemoryMap` import path and compile-time UEFI path literal usage. |
| UEFI image build | build fixed, boot not verified | `pwsh -NoLogo -NoProfile -File scripts/build-uefi.ps1` produced `target/x86_64-unknown-none/debug/chronosapien-uefi.img` (64 MiB). | Script path construction and FAT sizing were fixed narrowly for the image builder. |
| Single-core UEFI QEMU | partially verified in QEMU UEFI | `/private/tmp/chronoos-uefi-20260613-220234.serial.log`; `/private/tmp/chronoos-uefi-20260613-220234-boot.png`. | OVMF started the ChronoOS UEFI loader, then loader failed with `Out of Resources` before kernel framebuffer handoff. |
| Kernel framebuffer/shell | implemented in code, not verified | Same UEFI run did not reach `[CHRONO] uefi: framebuffer`, `handoff ok`, or `[CHRONO] boot complete`. | Do not claim UEFI kernel boot yet. |

## Startup Notes

- The heap is a 1 MiB free-list allocator implemented in code, not a bump-only allocator.
- IRQ keyboard buffering and polling fallback are implemented in code.
- PS/2 mouse support and small window interactions are implemented in code.
- SMP, userspace, syscalls, ELF execution, and networking are teaching systems and still need staged verification.
