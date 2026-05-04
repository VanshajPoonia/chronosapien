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
