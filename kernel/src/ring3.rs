//! Tiny opt-in ring 3 demo.
//!
//! This is not a process model yet. It installs a few bytes of user code into
//! one user-accessible page, builds an `iretq` frame, and lets the CPU enforce
//! that ring 3 cannot execute privileged instructions.

use crate::{gdt, memory};

const USER_CODE: [u8; 5] = [
    0xF4, // hlt         ; privileged in ring 3, should raise #GP
    0xF3, 0x90, // pause ; simple low-power spin hint
    0xEB, 0xFC, // jmp -4 ; loop back to pause after the #GP handler skips hlt
];

const SYSHELLO_CODE: [u8; 69] = [
    0xB8, 0x01, 0x00, 0x00, 0x00, // mov eax, SYS_WRITE
    0xBF, 0x01, 0x00, 0x00, 0x00, // mov edi, STDOUT
    0x48, 0x8D, 0x35, 0x14, 0x00, 0x00, 0x00, // lea rsi, [rip + message]
    0xBA, 0x20, 0x00, 0x00, 0x00, // mov edx, message_len
    0x0F, 0x05, // syscall
    0xB8, 0x03, 0x00, 0x00, 0x00, // mov eax, SYS_EXIT
    0x31, 0xFF, // xor edi, edi
    0x0F, 0x05, // syscall
    0xF3, 0x90, // pause
    0xEB, 0xFC, // jmp -4
    b'H', b'e', b'l', b'l', b'o', b' ', b'f', b'r',
    b'o', b'm', b' ', b'r', b'i', b'n', b'g', b' ',
    b'3', b' ', b'v', b'i', b'a', b' ', b's', b'y',
    b's', b'_', b'w', b'r', b'i', b't', b'e', b'\n',
];

pub fn run_demo() -> ! {
    install_user_code();
    enter_user_mode(memory::USER_CODE_START)
}

pub fn run_syshello() -> ! {
    install_syshello_code();
    enter_user_mode(memory::USER_CODE_START)
}

fn enter_user_mode(user_entry: u64) -> ! {
    let selectors = gdt::selectors();
    let user_stack_top = memory::USER_STACK_START + memory::USER_STACK_SIZE;

    crate::serial_println!("[CHRONO] kernel: entered ring 3");

    unsafe {
        core::arch::asm!(
            "push {user_data}",
            "push {user_stack}",
            "pushfq",
            "pop rax",
            "or rax, 0x200",
            "push rax",
            "push {user_code}",
            "push {user_entry}",
            "iretq",
            user_data = in(reg) selectors.user_data.0 as u64,
            user_stack = in(reg) user_stack_top,
            user_code = in(reg) selectors.user_code.0 as u64,
            user_entry = in(reg) user_entry,
            out("rax") _,
            options(noreturn),
        );
    }
}

pub fn is_demo_privilege_fault(rip: u64) -> bool {
    rip == memory::USER_CODE_START
}

fn install_user_code() {
    install_code(&USER_CODE);
}

fn install_syshello_code() {
    install_code(&SYSHELLO_CODE);
}

fn install_code(code: &[u8]) {
    let destination = memory::USER_CODE_START as *mut u8;

    unsafe {
        core::ptr::copy_nonoverlapping(code.as_ptr(), destination, code.len());
    }
}
