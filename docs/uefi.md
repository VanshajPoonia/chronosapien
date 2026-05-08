# UEFI Boot Port

ChronoOS now has a UEFI boot path alongside the BIOS bootloader paths. The UEFI
path builds a removable-media style disk image with a GPT partition table and a
FAT32 EFI System Partition.

## Boot Steps

1. UEFI firmware scans bootable devices for an EFI System Partition.
2. On removable media, it loads `\EFI\BOOT\BOOTX64.EFI` from that FAT partition.
3. The ChronoOS UEFI loader opens its own ESP and reads `\CHRONO\KERNEL.ELF`.
4. The loader parses ELF64 `PT_LOAD` segments, allocates pages at the requested
   physical addresses, copies segment bytes, and zeros BSS.
5. The loader opens the Graphics Output Protocol, chooses a usable linear
   framebuffer mode, and records framebuffer address, size, resolution, stride,
   bytes per pixel, and RGB/BGR format.
6. The loader reads the UEFI configuration table to find the ACPI RSDP address.
7. The loader allocates final boot-info storage and identity-mapped page tables.
8. The loader calls `ExitBootServices()`, receives the final firmware memory map,
   writes the ChronoOS v2 handoff, loads `CR3`, and jumps to
   `chrono_custom_entry`.

The loader logs the handoff to the UEFI console while Boot Services are active
and to COM1 for serial capture:

```text
[CHRONO] uefi: framebuffer at 0x...
[CHRONO] uefi: handoff ok
```

## EFI System Partition

The ESP is a FAT partition with standardized paths that firmware understands
without needing an operating system driver. ChronoOS packages:

```text
EFI\BOOT\BOOTX64.EFI
CHRONO\KERNEL.ELF
```

`BOOTX64.EFI` is the UEFI application. `KERNEL.ELF` stays as a normal ELF64
kernel binary so the loader can keep using program headers as the load contract.

## GOP vs VGA

BIOS-era boot code can call VBE/VGA firmware interrupts while it is still in
real mode. UEFI applications do not use those BIOS interrupts. Instead, they use
GOP, the Graphics Output Protocol, to obtain a linear framebuffer directly from
firmware.

After handoff, ChronoOS draws to that framebuffer exactly like it does in QEMU:
the only difference is where the framebuffer metadata came from.

## ExitBootServices

Boot Services own firmware allocation, protocol handles, file access, and device
setup. An OS loader must finish all firmware work before calling
`ExitBootServices()`.

UEFI also ties the exit call to a memory map key. Any allocation after reading a
memory map can change that key, so the final map must be fetched immediately
before exiting. The `uefi` crate wrapper performs that sequence and retries once
if firmware changes the key. ChronoOS does all loader allocations first, exits
Boot Services, copies the final memory map into its own boot-info buffer, and
then jumps without touching UEFI protocols again.

## BIOS vs UEFI

BIOS starts a 512-byte boot sector at `0x7c00` in 16-bit real mode. The custom
BIOS loader has to enable A20, use BIOS disk and video interrupts, switch through
protected mode into long mode, build page tables, and then jump to the kernel.

UEFI starts a PE/COFF application in a richer firmware environment. The loader
can use UEFI file protocols, GOP, configuration tables, and typed memory-map
descriptors before it exits firmware ownership and becomes responsible for the
machine.

The kernel-side abstraction is the same `BootContext`. BIOS custom handoff v1
does not include ACPI RSDP and remains supported. UEFI handoff v2 adds
`rsdp_addr` so SMP ACPI discovery can work after a UEFI boot.

## Build And Run

Build the UEFI loader and USB image:

```powershell
rustup target add x86_64-unknown-uefi
.\scripts\build-uefi.ps1
```

Run with QEMU and OVMF:

```powershell
.\scripts\run-uefi.ps1
```

If OVMF is not in a common install path, set `OVMF_CODE` to the firmware code
image path before running the script.

## Real Hardware Notes

Disable Secure Boot unless ChronoOS is later signed.

This first real-hardware milestone is about boot, display, memory handoff, and
serial logging. After the kernel starts, ChronoOS still expects legacy-style COM1
serial and PS/2-compatible keyboard I/O. USB HID, USB storage, and USB serial are
future driver milestones. ChronoFS persistence may fall back to heap storage on
machines without IDE compatibility.

To write the raw image on Windows, identify the USB disk carefully and use the
guarded writer:

```powershell
.\scripts\write-usb.ps1
.\scripts\write-usb.ps1 -DiskNumber <n> -ConfirmWrite
```
