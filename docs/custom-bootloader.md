# ChronoOS Custom BIOS Bootloader

ChronoOS keeps the existing `bootloader` crate image as a fallback and adds a
separate custom image at:

```text
target\x86_64-unknown-none\debug\chronosapien-custom.img
```

Build and run it with:

```powershell
.\scripts\build-custom.ps1
.\scripts\run-custom.ps1
```

The custom path is BIOS-only. It starts from sector 0, runs our 512-byte Stage 1
boot sector, loads Stage 2 from disk, prepares a ChronoOS boot handoff, enters
long mode, and jumps to `chrono_custom_entry`.

This path still emits the v1 ChronoOS handoff. The kernel accepts it for BIOS
compatibility and treats `rsdp_addr` as missing. The UEFI loader emits the v2
handoff, which adds `rsdp_addr` for ACPI/MADT discovery after UEFI boot.

## Stage 1 Line By Line

Stage 1 lives in `boot/stage1/stage1.asm`. BIOS loads it at physical address
`0x7C00` and jumps to it in 16-bit real mode.

```asm
bits 16
```

Assemble 16-bit instructions because BIOS starts us in real mode.

```asm
org 0x7C00
```

Tell NASM that labels are based at the address BIOS uses for the boot sector.

```asm
jmp short stage1_entry
nop
```

Jump over the small data area. The `nop` keeps the first bytes friendly to tools
that expect a BIOS Parameter Block shape.

```asm
stage2_sector_count dw 64
stage2_load_offset  dw 0x8000
stage2_load_segment dw 0x0000
boot_drive         db 0
```

These bytes describe where Stage 2 goes and keep the BIOS boot drive number.
BIOS gives us that drive number in `DL`.

```asm
cli
xor ax, ax
mov ds, ax
mov es, ax
mov ss, ax
mov sp, 0x7C00
sti
```

Interrupts are disabled while the stack and segments are changed. `DS`, `ES`,
and `SS` become zero so addresses map directly to low physical memory. The stack
starts just below the boot sector and grows downward.

```asm
mov [boot_drive], dl
```

Save the boot disk number before any BIOS call or helper can overwrite `DL`.

```asm
call serial_init
mov si, stage1_ok_message
call serial_write_string
```

Configure COM1 and print:

```text
[CHRONO] custom bootloader: stage1 ok
```

Serial is useful here because framebuffer output does not exist yet.

```asm
mov si, disk_address_packet
mov ah, 0x42
mov dl, [boot_drive]
int 0x13
jc disk_error
```

Ask BIOS to read sectors from disk. `AH=0x42` means "extended read" and `DS:SI`
points at the Disk Address Packet. If BIOS sets the carry flag, the read failed.

```asm
jmp 0x0000:0x8000
```

Jump to Stage 2 after the disk read succeeds.

```asm
disk_error:
    mov si, disk_error_message
    call serial_write_string
    cli
.hang:
    hlt
    jmp .hang
```

If disk I/O fails, print a serial error and halt forever. There is no operating
system to return to.

```asm
serial_init:
```

This routine programs the classic 16550 UART at COM1 (`0x3F8`). It disables
serial interrupts, sets baud divisor 3, chooses 8N1 framing, enables FIFOs, and
enables modem control outputs.

```asm
serial_write_string:
```

This routine reads bytes from `DS:SI` until it sees zero. Each byte is passed to
`serial_write_byte`.

```asm
serial_write_byte:
```

This routine waits until COM1 line-status bit 5 says the transmit register is
empty, then writes one byte to the COM1 data port.

```asm
align 4
disk_address_packet:
    db 0x10
    db 0x00
    dw 64
    dw 0x8000
    dw 0x0000
    dq 1
```

This is the INT `0x13` extended-read packet. It says:

- packet size: 16 bytes
- reserved: zero
- sector count: 64
- destination: `0000:8000`
- starting LBA: 1

LBA 0 is Stage 1 itself, so Stage 2 starts at LBA 1.

```asm
times 510 - ($ - $$) db 0
dw 0xAA55
```

Pad the boot sector to byte 510, then write the BIOS boot signature. On disk
this appears as bytes `55 aa`.

## BIOS INT 0x13 Disk Reads

BIOS interrupt `0x13` is the firmware disk service. The old CHS interface reads
by cylinder/head/sector, which is awkward for generated disk images. ChronoOS
uses the extended interface:

```text
AH = 0x42
DL = BIOS drive number
DS:SI = Disk Address Packet
int 0x13
```

The carry flag reports success or failure. If carry is clear, the sectors were
read. If carry is set, `AH` contains a BIOS error code.

Stage 1 uses a 16-byte packet because it only reads Stage 2 into low memory.
Stage 2 uses a 24-byte packet so QEMU BIOS can DMA kernel segments to their
physical destinations.

## A20

Original 8086 CPUs had only 20 address lines, so addresses wrapped at 1 MiB.
For compatibility, early PCs kept that wrap behavior through the A20 gate. If
A20 is disabled, an address like `0x100000` aliases back to `0x00000`.

ChronoOS must enable A20 before using memory above 1 MiB. Stage 2 uses the fast
A20 gate at port `0x92`:

```asm
in al, 0x92
or al, 0000_0010b
and al, 1111_1110b
out 0x92, al
```

Bit 1 enables A20. Bit 0 is cleared to avoid accidentally requesting a reset on
some chipsets.

## Real Mode To Protected Mode To Long Mode

BIOS starts in real mode:

- 16-bit registers are the default.
- Segment registers form addresses as `segment * 16 + offset`.
- BIOS interrupts are available.
- Paging is off.

Stage 2 uses real mode while it needs BIOS calls: E820 memory map, VBE
framebuffer setup, and disk reads.

Protected mode begins by loading a GDT and setting `CR0.PE`:

```asm
lgdt [gdt_descriptor]
mov eax, cr0
or eax, 1
mov cr0, eax
jmp CODE32_SEL:protected_entry
```

The far jump reloads `CS`, which makes the CPU use the protected-mode code
descriptor.

Long mode requires protected mode plus paging:

1. Build page tables.
2. Load `CR3` with the PML4 address.
3. Set `CR4.PAE`.
4. Set `EFER.LME` through MSR `0xC0000080`.
5. Set `CR0.PG`.
6. Far jump to a 64-bit code descriptor.

ChronoOS initially identity maps the first 1 GiB with 2 MiB pages. That keeps
early addresses simple: virtual address equals physical address.

## Custom Handoff

The custom loader passes a pointer to `ChronoBootInfo` in `RDI`, following the
System V x86_64 calling convention. The kernel validates:

- magic: `CHRONOBT`
- version: `1`
- framebuffer address and geometry
- memory region pointer/count
- physical-memory offset
- kernel load address and length

The borrowed bootloader path is adapted into the same internal `BootContext`, so
both boot paths share the normal ChronoOS startup flow after handoff.
