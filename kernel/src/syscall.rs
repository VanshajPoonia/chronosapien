//! Minimal SYSCALL/SYSRET system call layer.
//!
//! `SYSCALL` is the fast x86_64 path from ring 3 into a kernel entry point.
//! Unlike interrupts, it does not consult the TSS to switch stacks, so this
//! module switches to a dedicated kernel stack before calling Rust code.

use crate::keyboard::{self, KeyEvent};
use crate::{console, gdt, memory, timer};

pub const SYS_WRITE: u64 = 1;
pub const SYS_READ: u64 = 2;
pub const SYS_EXIT: u64 = 3;
pub const SYS_UPTIME: u64 = 4;

const FD_STDIN: u64 = 0;
const FD_STDOUT: u64 = 1;
const FD_STDERR: u64 = 2;

const ERR_UNSUPPORTED: u64 = (-1i64) as u64;
const ERR_BAD_FD: u64 = (-2i64) as u64;
const ERR_BAD_BUFFER: u64 = (-3i64) as u64;

const IA32_EFER: u32 = 0xC000_0080;
const IA32_STAR: u32 = 0xC000_0081;
const IA32_LSTAR: u32 = 0xC000_0082;
const IA32_FMASK: u32 = 0xC000_0084;

const EFER_SYSCALL_ENABLE: u64 = 1 << 0;
const RFLAGS_INTERRUPT_ENABLE: u64 = 1 << 9;
const SYSCALL_STACK_SIZE: usize = 4096 * 4;

#[repr(align(16))]
struct SyscallStack {
    bytes: [u8; SYSCALL_STACK_SIZE],
}

static mut SYSCALL_STACK: SyscallStack = SyscallStack {
    bytes: [0; SYSCALL_STACK_SIZE],
};

pub fn init() {
    let selectors = gdt::selectors();
    let kernel_code = selectors.kernel_code.0 as u64;
    let user_sysret_base = selectors
        .user_data
        .0
        .checked_sub(8)
        .expect("user data selector follows kernel data selector")
        as u64;

    let star = (user_sysret_base << 48) | (kernel_code << 32);

    unsafe {
        wrmsr(IA32_STAR, star);
        wrmsr(IA32_LSTAR, syscall_entry as usize as u64);
        wrmsr(IA32_FMASK, RFLAGS_INTERRUPT_ENABLE);
        wrmsr(IA32_EFER, rdmsr(IA32_EFER) | EFER_SYSCALL_ENABLE);
    }

    crate::serial_println!("[CHRONO] syscall: SYSCALL/SYSRET initialized");
}

#[naked]
unsafe extern "C" fn syscall_entry() -> ! {
    core::arch::naked_asm!(
        "mov r10, rsp",
        "lea rsp, [rip + {stack}]",
        "add rsp, {stack_size}",
        "push r11",
        "push rcx",
        "push r10",
        "sub rsp, 8",
        "sti",
        "mov rcx, rdx",
        "mov rdx, rsi",
        "mov rsi, rdi",
        "mov rdi, rax",
        "call {dispatch}",
        "cli",
        "add rsp, 8",
        "pop r10",
        "pop rcx",
        "pop r11",
        "mov rsp, r10",
        "sysretq",
        stack = sym SYSCALL_STACK,
        stack_size = const SYSCALL_STACK_SIZE,
        dispatch = sym dispatch,
    )
}

extern "C" fn dispatch(number: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    match number {
        SYS_WRITE => sys_write(arg0, arg1, arg2),
        SYS_READ => sys_read(arg0, arg1, arg2),
        SYS_EXIT => sys_exit(arg0),
        SYS_UPTIME => sys_uptime(),
        _ => {
            debug_log_unknown(number);
            ERR_UNSUPPORTED
        }
    }
}

fn sys_write(fd: u64, buffer: u64, len: u64) -> u64 {
    debug_log_write(fd, len);

    let Some(bytes) = user_slice(buffer, len) else {
        return ERR_BAD_BUFFER;
    };

    match fd {
        FD_STDOUT => {
            for byte in bytes {
                crate::print!("{}", *byte as char);
            }
            len
        }
        FD_STDERR => {
            for byte in bytes {
                crate::serial_print!("{}", *byte as char);
            }
            len
        }
        _ => ERR_BAD_FD,
    }
}

fn sys_read(fd: u64, buffer: u64, len: u64) -> u64 {
    debug_log_read(fd, len);

    if fd != FD_STDIN {
        return ERR_BAD_FD;
    }
    if len == 0 {
        return 0;
    }
    if !user_range_is_valid(buffer, len) {
        return ERR_BAD_BUFFER;
    }

    let destination = buffer as *mut u8;
    let capacity = len as usize;
    let mut count = 0usize;

    loop {
        match keyboard::read_key() {
            Some(KeyEvent::Char(byte)) => {
                if count + 1 < capacity {
                    unsafe {
                        destination.add(count).write(byte);
                    }
                    count += 1;
                    crate::print!("{}", byte as char);
                }
            }
            Some(KeyEvent::Backspace) => {
                if count > 0 {
                    count -= 1;
                    console::backspace();
                }
            }
            Some(KeyEvent::Enter) => {
                crate::println!();
                unsafe {
                    destination.add(count).write(0);
                }
                return count as u64;
            }
            None => cpu_relax(),
        }
    }
}

fn sys_exit(code: u64) -> ! {
    debug_log_exit(code);
    crate::serial_println!("[CHRONO] syscall: user task exited code={}", code);
    crate::println!("[user exited: {}]", code);

    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

fn sys_uptime() -> u64 {
    debug_log_uptime();
    timer::ticks()
}

fn user_slice(buffer: u64, len: u64) -> Option<&'static [u8]> {
    if !user_range_is_valid(buffer, len) {
        return None;
    }

    Some(unsafe { core::slice::from_raw_parts(buffer as *const u8, len as usize) })
}

fn user_range_is_valid(buffer: u64, len: u64) -> bool {
    let Some(end) = buffer.checked_add(len) else {
        return false;
    };

    buffer >= memory::USER_CODE_START && end <= memory::USER_STACK_START + memory::USER_STACK_SIZE
}

fn debug_log_write(fd: u64, len: u64) {
    #[cfg(debug_assertions)]
    crate::serial_println!("[CHRONO] syscall: write fd={} len={}", fd, len);
}

fn debug_log_read(fd: u64, len: u64) {
    #[cfg(debug_assertions)]
    crate::serial_println!("[CHRONO] syscall: read fd={} len={}", fd, len);
}

fn debug_log_exit(code: u64) {
    #[cfg(debug_assertions)]
    crate::serial_println!("[CHRONO] syscall: exit code={}", code);
}

fn debug_log_uptime() {
    #[cfg(debug_assertions)]
    crate::serial_println!("[CHRONO] syscall: uptime");
}

fn debug_log_unknown(number: u64) {
    #[cfg(debug_assertions)]
    crate::serial_println!("[CHRONO] syscall: unknown number={}", number);
}

fn cpu_relax() {
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}

unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;

    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack, preserves_flags)
    );

    ((high as u64) << 32) | low as u64
}

unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;

    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack, preserves_flags)
    );
}
