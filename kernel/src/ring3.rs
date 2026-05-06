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

pub fn run_demo() -> ! {
    install_user_code();

    let selectors = gdt::selectors();
    let user_stack_top = memory::USER_STACK_START + memory::USER_STACK_SIZE;
    let user_entry = memory::USER_CODE_START;

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
    let destination = memory::USER_CODE_START as *mut u8;

    unsafe {
        core::ptr::copy_nonoverlapping(USER_CODE.as_ptr(), destination, USER_CODE.len());
    }
}
