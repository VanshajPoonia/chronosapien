//! Cooperative round-robin task scheduler.
//!
//! # What a task stack is
//!
//! Every task runs on its own private stack — a contiguous region of memory
//! the CPU uses for function call frames, local variables, and saved return
//! addresses. The stack pointer register (RSP) always points at the most-
//! recently-pushed value. Because x86-64 stacks grow *downward*, the highest
//! address is the beginning and the current "top" is the lowest in-use address.
//!
//! Each task needs its own stack so that task A's call frames do not overwrite
//! task B's. We pre-allocate `TASK_STACK_SIZE` bytes per slot in a static
//! array (`STACKS`) and carve them up at spawn time.
//!
//! # How context switching works at the register level
//!
//! Switching from task A to task B:
//!   1. Push all *callee-saved* registers (RBX, RBP, R12–R15) onto A's stack.
//!      Caller-saved registers (RAX, RCX, RDX, RSI, RDI, R8–R11) are already
//!      saved by A's caller per the System V ABI, so we skip them.
//!   2. Write A's current RSP into `tasks[A].rsp`.
//!   3. Load `tasks[B].rsp` into RSP — we are now on B's stack.
//!   4. Pop B's callee-saved registers from B's stack.
//!   5. `ret` — the return address on B's stack takes us to where B last
//!      yielded, or to B's entry function on its very first run.
//!
//! From each task's perspective `yield_now` is just a function that was called
//! and returned; the context switch is invisible.
//!
//! # Round-robin scheduling
//!
//! On every `yield_now` the scheduler walks the task table starting one slot
//! past the current task, wrapping around, and picks the first `Ready` task.
//! After the last slot it wraps to slot 0. Every task gets exactly one turn
//! before any task gets a second turn — simple, fair, no priorities.

use core::cell::UnsafeCell;

pub const MAX_TASKS: usize = 8;
const TASK_STACK_SIZE: usize = 16 * 1024; // 16 KiB per task
const TASK_NAME_LEN: usize = 16;

/// Lifecycle state of a scheduler slot.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Slot is unused and available for a future spawn.
    Empty,
    /// Task is currently executing on the CPU.
    Running,
    /// Task is waiting for its next turn on the CPU.
    Ready,
    /// Task is waiting for an external event (reserved for future use).
    Blocked,
    /// Task has been terminated; the slot will be reused on the next spawn.
    Dead,
}

#[derive(Clone, Copy)]
struct TaskInfo {
    id: u8,
    name: [u8; TASK_NAME_LEN],
    name_len: usize,
    state: TaskState,
    /// Saved RSP. Written by `context_switch` each time the task yields, read
    /// back when the task is resumed.
    rsp: u64,
}

impl TaskInfo {
    const fn empty() -> Self {
        Self {
            id: 0,
            name: [0; TASK_NAME_LEN],
            name_len: 0,
            state: TaskState::Empty,
            rsp: 0,
        }
    }

    fn name_str(&self) -> &str {
        // SAFETY: name bytes are always written from valid UTF-8 input via
        // `set_task_name`, so the slice is guaranteed to be valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.name_len]) }
    }
}

/// 16 KiB stack, 16-byte aligned as required by the x86-64 System V ABI.
#[repr(align(16))]
#[derive(Clone, Copy)]
struct TaskStack {
    bytes: [u8; TASK_STACK_SIZE],
}

struct Global<T>(UnsafeCell<T>);
// SAFETY: All accesses are serialised through `without_interrupts`. The kernel
// is single-core and cooperative, so no two tasks run concurrently.
unsafe impl<T> Sync for Global<T> {}

struct Scheduler {
    tasks: [TaskInfo; MAX_TASKS],
    current: usize,
    count: usize,
}

impl Scheduler {
    const fn new() -> Self {
        const EMPTY: TaskInfo = TaskInfo::empty();
        Self {
            tasks: [EMPTY; MAX_TASKS],
            current: 0,
            count: 0,
        }
    }
}

static SCHED: Global<Scheduler> = Global(UnsafeCell::new(Scheduler::new()));
static STACKS: Global<[TaskStack; MAX_TASKS]> =
    Global(UnsafeCell::new([TaskStack { bytes: [0; TASK_STACK_SIZE] }; MAX_TASKS]));

// ─── public API ──────────────────────────────────────────────────────────────

/// Register the current execution context as task 0 ("shell") and mark it
/// Running. Must be called exactly once, before any `spawn` or `yield_now`.
pub fn init() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let sched = unsafe { &mut *SCHED.0.get() };
        let task = &mut sched.tasks[0];
        task.id = 0;
        set_task_name(task, "shell");
        task.state = TaskState::Running;
        sched.current = 0;
        sched.count = 1;
    });
    crate::serial_println!("[CHRONO] sched: initialized, task 0 = shell");
}

// ─── internals ───────────────────────────────────────────────────────────────

fn set_task_name(task: &mut TaskInfo, name: &str) {
    let bytes = name.as_bytes();
    let len = bytes.len().min(TASK_NAME_LEN);
    task.name[..len].copy_from_slice(&bytes[..len]);
    task.name_len = len;
}

/// Write an initial call frame onto a fresh task stack so the first
/// `context_switch` into the task jumps to `entry`.
///
/// Frame layout (high address → low address, each cell is 8 bytes):
///
/// ```text
///   stack_top - 8   │ 0            │ alignment dummy
///   stack_top - 16  │ entry as u64 │ ← "return address" consumed by `ret`
///   stack_top - 24  │ 0            │ r15
///   stack_top - 32  │ 0            │ r14
///   stack_top - 40  │ 0            │ r13
///   stack_top - 48  │ 0            │ r12
///   stack_top - 56  │ 0            │ rbp
///   stack_top - 64  │ 0            │ rbx  ← initial RSP saved here
/// ```
///
/// After `context_switch` pops six registers and executes `ret`, RSP =
/// `stack_top - 8`. Since `stack_top` is 16-byte aligned (repr(align(16))),
/// `stack_top - 8` is 8 mod 16 — exactly the alignment the x86-64 ABI
/// requires at a function's entry point.
unsafe fn init_task_stack(bytes: &mut [u8; TASK_STACK_SIZE], entry: fn() -> !) -> u64 {
    let top = bytes.as_mut_ptr().add(TASK_STACK_SIZE) as *mut u64;
    top.sub(1).write(0);                         // alignment dummy
    top.sub(2).write((entry as usize) as u64);   // first "return address"
    top.sub(3).write(0);                         // r15
    top.sub(4).write(0);                         // r14
    top.sub(5).write(0);                         // r13
    top.sub(6).write(0);                         // r12
    top.sub(7).write(0);                         // rbp
    top.sub(8).write(0);                         // rbx
    top.sub(8) as u64                            // initial RSP
}

/// Save the current task's callee-saved registers and RSP, then load the next
/// task's RSP and restore its callee-saved registers before returning into it.
///
/// `extern "C"` maps arguments to RDI (current_rsp) and RSI (next_rsp) via
/// the System V AMD64 ABI. The function is `#[naked]` so the compiler emits
/// no prologue or epilogue — only our assembly.
#[naked]
unsafe extern "C" fn context_switch(current_rsp: *mut u64, next_rsp: *const u64) {
    core::arch::naked_asm!(
        // Save callee-saved registers onto the current (outgoing) stack.
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // Persist the outgoing RSP (RDI holds current_rsp pointer).
        "mov [rdi], rsp",
        // Switch to the incoming stack (RSI holds next_rsp pointer).
        "mov rsp, [rsi]",
        // Restore the incoming task's callee-saved registers.
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        // Jump into the incoming task: on first run this is its entry function;
        // on subsequent runs this is the instruction after its last `ret`.
        "ret",
    )
}
