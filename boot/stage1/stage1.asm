; ChronoOS Stage 1 boot sector.
; BIOS loads this 512-byte sector at physical address 0x0000:0x7C00.

bits 16                         ; Assemble 16-bit real-mode instructions.
org 0x7C00                      ; Tell NASM labels are based at BIOS load address.

stage1_start:                   ; First executed byte after the BIOS jump.
    jmp short stage1_entry       ; Skip over constants/data to executable setup.
    nop                         ; Keep the first bytes compatible with BPB-shaped tools.

stage2_sector_count dw 64        ; Number of sectors Stage 1 loads for Stage 2.
stage2_load_offset  dw 0x8000    ; Offset where Stage 2 is loaded.
stage2_load_segment dw 0x0000    ; Segment where Stage 2 is loaded.
boot_drive         db 0          ; BIOS passes the boot disk number in DL.

stage1_entry:                    ; Main Stage 1 code begins here.
    cli                         ; Disable interrupts while stack/segments change.
    xor ax, ax                  ; Set AX to zero for segment initialization.
    mov ds, ax                  ; DS=0 so data labels use physical addresses.
    mov es, ax                  ; ES=0 for BIOS disk destination buffers.
    mov ss, ax                  ; SS=0 so the stack lives in low conventional RAM.
    mov sp, 0x7C00              ; Stack grows downward below the boot sector.
    sti                         ; Re-enable interrupts after stack is valid.

    mov [boot_drive], dl        ; Preserve BIOS boot drive for INT 0x13 reads.

    call serial_init            ; Configure COM1 before printing boot progress.
    mov si, stage1_ok_message   ; SI points at the zero-terminated status string.
    call serial_write_string    ; Emit "[CHRONO] custom bootloader: stage1 ok".

    mov si, disk_address_packet ; SI points at the INT 0x13 extended-read packet.
    mov ah, 0x42                ; AH=0x42 selects BIOS extended disk read.
    mov dl, [boot_drive]        ; DL is the BIOS drive number to read from.
    int 0x13                    ; BIOS reads LBA sectors into 0000:8000.
    jc disk_error               ; Carry flag means the BIOS reported a read error.

    jmp 0x0000:0x8000           ; Far jump to Stage 2 at physical address 0x8000.

disk_error:                     ; Disk read failure path.
    mov si, disk_error_message  ; SI points at the failure text.
    call serial_write_string    ; Print the failure so serial logs explain the hang.
    cli                         ; Disable interrupts before halting forever.
.hang:                          ; Infinite halt loop label.
    hlt                         ; Sleep until an interrupt arrives.
    jmp .hang                   ; Continue halting if execution resumes.

serial_init:                    ; Configure COM1 16550-compatible UART.
    mov dx, 0x3F8 + 1           ; COM1 interrupt-enable register.
    mov al, 0x00                ; Disable serial interrupts.
    out dx, al                  ; Write interrupt-enable value.
    mov dx, 0x3F8 + 3           ; COM1 line-control register.
    mov al, 0x80                ; Set DLAB so divisor registers are accessible.
    out dx, al                  ; Enter divisor-latch mode.
    mov dx, 0x3F8 + 0           ; COM1 divisor low byte.
    mov al, 0x03                ; Divisor 3 gives 38400 baud with standard clock.
    out dx, al                  ; Write divisor low byte.
    mov dx, 0x3F8 + 1           ; COM1 divisor high byte.
    mov al, 0x00                ; High divisor byte is zero.
    out dx, al                  ; Write divisor high byte.
    mov dx, 0x3F8 + 3           ; COM1 line-control register.
    mov al, 0x03                ; 8 data bits, no parity, one stop bit.
    out dx, al                  ; Leave DLAB mode and set 8N1 framing.
    mov dx, 0x3F8 + 2           ; COM1 FIFO-control register.
    mov al, 0xC7                ; Enable FIFO, clear queues, use 14-byte threshold.
    out dx, al                  ; Apply FIFO configuration.
    mov dx, 0x3F8 + 4           ; COM1 modem-control register.
    mov al, 0x0B                ; Enable IRQs, RTS, and DTR outputs.
    out dx, al                  ; Apply modem-control configuration.
    ret                         ; Return to the caller.

serial_write_string:            ; Write a zero-terminated string from DS:SI.
    lodsb                       ; Load next byte from DS:SI into AL and advance SI.
    test al, al                 ; Check whether AL is the terminating zero.
    jz .done                    ; Stop printing at the string terminator.
    call serial_write_byte      ; Send the character in AL to COM1.
    jmp serial_write_string     ; Continue with the next character.
.done:                          ; String write is complete.
    ret                         ; Return to the caller.

serial_write_byte:              ; Write the byte in AL to COM1.
    push ax                     ; Save the byte while polling line status.
    mov dx, 0x3F8 + 5           ; COM1 line-status register.
.wait:                          ; Poll until the transmit register is empty.
    in al, dx                   ; Read the line-status register.
    test al, 0x20               ; Bit 5 means transmit holding register empty.
    jz .wait                    ; If not empty yet, keep polling.
    mov dx, 0x3F8 + 0           ; COM1 data register.
    pop ax                      ; Restore the byte to send.
    out dx, al                  ; Write the byte to COM1.
    ret                         ; Return to the caller.

align 4                         ; INT 0x13 packet must be aligned for old BIOSes.
disk_address_packet:            ; Disk Address Packet for INT 0x13 AH=0x42.
    db 0x10                     ; Packet size is 16 bytes.
    db 0x00                     ; Reserved byte must be zero.
    dw 64                       ; Sector count, patched by image builder if needed.
    dw 0x8000                   ; Destination offset.
    dw 0x0000                   ; Destination segment.
    dq 1                        ; Start at LBA 1 because LBA 0 is this boot sector.

stage1_ok_message:              ; Success text for serial logging.
    db "[CHRONO] custom bootloader: stage1 ok", 13, 10, 0 ; CRLF and NUL.

disk_error_message:             ; Failure text for serial logging.
    db "[CHRONO] custom bootloader: disk read failed", 13, 10, 0 ; CRLF and NUL.

times 510 - ($ - $$) db 0        ; Pad the sector to byte offset 510.
dw 0xAA55                       ; BIOS boot signature stored as bytes 55 AA.
