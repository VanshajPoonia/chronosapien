; ChronoOS Stage 2 BIOS loader.
; Stage 1 loads this binary to physical address 0x8000 and jumps here.

bits 16
org 0x8000

stage2_start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    sti

    mov [boot_drive], dl
    call serial_init
    mov si, stage2_ok_message
    call serial_write_string

    call enable_a20
    call collect_e820_map
    call set_vbe_mode
    call load_kernel_segments
    call build_boot_info
    call enter_protected_mode

.halt:
    hlt
    jmp .halt

enable_a20:
    in al, 0x92
    or al, 0000_0010b
    and al, 1111_1110b
    out 0x92, al
    ret

collect_e820_map:
    xor ebx, ebx
    xor di, di
    mov es, di
    mov di, MEMORY_MAP_ADDR
    xor bp, bp
.next:
    mov eax, 0xE820
    mov edx, 0x534D4150
    mov ecx, 24
    int 0x15
    jc .done
    cmp eax, 0x534D4150
    jne .done
    mov eax, [es:di]
    mov [es:di + 0], eax
    mov eax, [es:di + 4]
    mov [es:di + 4], eax
    mov eax, [es:di + 8]
    add eax, [es:di + 0]
    mov [es:di + 8], eax
    mov eax, [es:di + 12]
    adc eax, [es:di + 4]
    mov [es:di + 12], eax
    mov eax, [es:di + 16]
    cmp eax, 1
    je .usable
    mov eax, 2
    jmp .store_kind
.usable:
    mov eax, 1
.store_kind:
    mov [es:di + 16], eax
    add di, 24
    inc bp
    test ebx, ebx
    jne .next
.done:
    mov [memory_region_count], bp
    ret

set_vbe_mode:
    mov ax, 0x4F01
    mov cx, 0x4118
    mov di, VBE_MODE_INFO_ADDR
    int 0x10
    mov ax, 0x4F02
    mov bx, 0x4118
    int 0x10
    ret

load_kernel_segments:
    mov bx, segment_table
    mov cx, [segment_count]
.next_segment:
    test cx, cx
    jz .done
    push cx
    mov eax, [bx + 0]
    mov [dap_lba], eax
    mov eax, [bx + 4]
    mov [dap_lba + 4], eax
    mov ax, [bx + 8]
    mov [dap_count], ax
    mov eax, [bx + 16]
    mov [dap_dest], eax
    mov eax, [bx + 20]
    mov [dap_dest + 4], eax
    mov si, disk_address_packet
    mov ah, 0x42
    mov dl, [boot_drive]
    int 0x13
    jc .read_error
    pop cx
    add bx, SEGMENT_ENTRY_SIZE
    loop .next_segment
.done:
    ret
.read_error:
    pop cx
    jmp boot_error

build_boot_info:
    push di
    push cx
    xor ax, ax
    mov es, ax
    mov di, BOOT_INFO_ADDR
    xor eax, eax
    mov cx, 96 / 4
    rep stosd
    pop cx
    pop di
    mov dword [BOOT_INFO_ADDR + 0], 0x4F524843
    mov dword [BOOT_INFO_ADDR + 4], 0x54424F4E
    mov dword [BOOT_INFO_ADDR + 8], 1
    mov eax, [VBE_MODE_INFO_ADDR + 0x28]
    mov [BOOT_INFO_ADDR + 16], eax
    mov dword [BOOT_INFO_ADDR + 24], 3145728
    mov ax, [VBE_MODE_INFO_ADDR + 0x12]
    movzx eax, ax
    mov [BOOT_INFO_ADDR + 32], eax
    mov ax, [VBE_MODE_INFO_ADDR + 0x14]
    movzx eax, ax
    mov [BOOT_INFO_ADDR + 36], eax
    mov ax, [VBE_MODE_INFO_ADDR + 0x10]
    shr ax, 2
    movzx eax, ax
    mov [BOOT_INFO_ADDR + 40], eax
    mov dword [BOOT_INFO_ADDR + 44], 4
    mov dword [BOOT_INFO_ADDR + 48], 2
    mov dword [BOOT_INFO_ADDR + 56], MEMORY_MAP_ADDR
    movzx eax, word [memory_region_count]
    mov [BOOT_INFO_ADDR + 64], eax
    mov dword [BOOT_INFO_ADDR + 72], 0
    mov eax, [kernel_addr]
    mov [BOOT_INFO_ADDR + 80], eax
    mov eax, [kernel_len]
    mov [BOOT_INFO_ADDR + 88], eax
    ret

enter_protected_mode:
    cli
    lgdt [gdt_descriptor]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp CODE32_SEL:protected_entry

boot_error:
    mov si, stage2_error_message
    call serial_write_string
    cli
.hang:
    hlt
    jmp .hang

serial_init:
    mov dx, 0x3F8 + 1
    mov al, 0x00
    out dx, al
    mov dx, 0x3F8 + 3
    mov al, 0x80
    out dx, al
    mov dx, 0x3F8 + 0
    mov al, 0x03
    out dx, al
    mov dx, 0x3F8 + 1
    mov al, 0x00
    out dx, al
    mov dx, 0x3F8 + 3
    mov al, 0x03
    out dx, al
    mov dx, 0x3F8 + 2
    mov al, 0xC7
    out dx, al
    mov dx, 0x3F8 + 4
    mov al, 0x0B
    out dx, al
    ret

serial_write_string:
    lodsb
    test al, al
    jz .done
    call serial_write_byte
    jmp serial_write_string
.done:
    ret

serial_write_byte:
    push ax
    mov dx, 0x3F8 + 5
.wait:
    in al, dx
    test al, 0x20
    jz .wait
    mov dx, 0x3F8 + 0
    pop ax
    out dx, al
    ret

bits 32
protected_entry:
    mov ax, DATA32_SEL
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov esp, 0x80000
    call setup_identity_page_tables
    mov eax, 0x90000
    mov cr3, eax
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax
    jmp CODE64_SEL:long_mode_entry

setup_identity_page_tables:
    mov edi, 0x90000
    xor eax, eax
    mov ecx, 4096 * 3 / 4
    rep stosd
    mov dword [0x90000], 0x91003
    mov dword [0x91000], 0x92003
    mov edi, 0x92000
    mov eax, 0x00000083
    mov ecx, 512
.map:
    mov [edi], eax
    mov dword [edi + 4], 0
    add eax, 0x200000
    add edi, 8
    loop .map
    ret

bits 64
long_mode_entry:
    mov ax, DATA32_SEL
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov rsp, 0x80000
    mov rdi, BOOT_INFO_ADDR
    mov rax, [kernel_entry]
    jmp rax

bits 16
align 8
gdt_start:
    dq 0
    dq 0x00CF9A000000FFFF
    dq 0x00CF92000000FFFF
    dq 0x00AF9A000000FFFF
gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start

CODE32_SEL equ 0x08
DATA32_SEL equ 0x10
CODE64_SEL equ 0x18

BOOT_INFO_ADDR equ 0x7000
MEMORY_MAP_ADDR equ 0x5000
VBE_MODE_INFO_ADDR equ 0x6000
SEGMENT_ENTRY_SIZE equ 32

boot_drive db 0
memory_region_count dw 0

align 8
disk_address_packet:
    db 0x18
    db 0
dap_count:
    dw 0
    dw 0
    dw 0
dap_lba:
    dq 0
dap_dest:
    dq 0

stage2_ok_message:
    db "[CHRONO] custom bootloader: stage2 ok", 13, 10, 0
stage2_error_message:
    db "[CHRONO] custom bootloader: stage2 disk read failed", 13, 10, 0

align 8
manifest_magic:
    db "CHRONO2M"
kernel_entry:
    dq 0
kernel_addr:
    dq 0
kernel_len:
    dq 0
segment_count:
    dw 0
    times 6 db 0
segment_table:
    times 32 * 8 db 0
