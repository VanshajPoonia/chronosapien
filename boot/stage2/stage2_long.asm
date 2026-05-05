; Long-mode trampoline reference for ChronoOS Stage 2.
; The active long-mode code is currently in stage2_real.asm so NASM can assemble
; one flat Stage 2 binary. This file keeps the 64-bit handoff isolated for study.

bits 64

chrono_long_entry:
    mov rsp, 0x80000            ; Use the loader stack below the boot info block.
    mov rdi, 0x7000             ; System V ABI first argument: ChronoBootInfo*.
    mov rax, [kernel_entry_ptr] ; Load the patched kernel entry address.
    jmp rax                     ; Transfer ownership to the kernel permanently.

kernel_entry_ptr:
    dq 0                        ; Patched by the image builder in the flat Stage 2.
