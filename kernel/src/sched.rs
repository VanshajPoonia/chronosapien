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

use crate::smp::{self, MAX_CORES};
use crate::spinlock::SpinLock;

pub const MAX_TASKS: usize = 8;
const TASK_STACK_SIZE: usize = 16 * 1024; // 16 KiB per task
const TASK_NAME_LEN: usize = 16;
const NO_TASK: usize = usize::MAX;

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
    core_id: usize,
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
            core_id: 0,
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

struct Scheduler {
    tasks: [TaskInfo; MAX_TASKS],
    current: [usize; MAX_CORES],
    idle_rsp: [u64; MAX_CORES],
    count: usize,
}

impl Scheduler {
    const fn new() -> Self {
        const EMPTY: TaskInfo = TaskInfo::empty();
        Self {
            tasks: [EMPTY; MAX_TASKS],
            current: [NO_TASK; MAX_CORES],
            idle_rsp: [0; MAX_CORES],
            count: 0,
        }
    }
}

static SCHED: SpinLock<Scheduler> = SpinLock::new(Scheduler::new());
static STACKS: SpinLock<[TaskStack; MAX_TASKS]> =
    SpinLock::new([TaskStack { bytes: [0; TASK_STACK_SIZE] }; MAX_TASKS]);

// ─── public API ──────────────────────────────────────────────────────────────

/// Register the current execution context as task 0 ("shell") and mark it
/// Running. Must be called exactly once, before any `spawn` or `yield_now`.
pub fn init() {
    let mut sched = SCHED.lock_irq();
    let task = &mut sched.tasks[0];
    task.id = 0;
    task.core_id = 0;
    set_task_name(task, "shell");
    task.state = TaskState::Running;
    sched.current[0] = 0;
    sched.count = 1;
    drop(sched);

    crate::serial_println!("[CHRONO] sched: initialized, task 0 = shell");
}

/// Spawn a new task that will start executing at `entry` on its next turn.
///
/// Returns the assigned task ID, or `None` if all 8 slots are occupied.
pub fn spawn(name: &str, entry: fn() -> !) -> Option<u8> {
    let mut sched = SCHED.lock_irq();

    // Reuse Dead slots so killing and re-spawning works indefinitely.
    let slot = (0..MAX_TASKS)
        .find(|&i| matches!(sched.tasks[i].state, TaskState::Empty | TaskState::Dead))?;
    let core_id = least_loaded_core(&sched);

    let task_rsp = {
        let mut stacks = STACKS.lock_irq();
        unsafe { init_task_stack(&mut stacks[slot].bytes, entry) }
    };

    let task = &mut sched.tasks[slot];
    *task = TaskInfo::empty();
    task.id = slot as u8;
    task.core_id = core_id;
    set_task_name(task, name);
    task.state = TaskState::Ready;
    task.rsp = task_rsp;
    sched.count += 1;

    drop(sched);
    crate::serial_println!(
        "[CHRONO] sched: spawned task {} ({}) on core {}",
        slot,
        name,
        core_id
    );
    Some(slot as u8)
}

/// Terminate the task with the given ID.
///
/// Returns `false` when:
/// - `id` is out of range or the slot is empty/already dead.
/// - `id` is the *currently running* task — a cooperative task must yield
///   to another task before it can be terminated.
pub fn kill(id: u8) -> bool {
    let mut sched = SCHED.lock_irq();
    let idx = id as usize;

    if idx >= MAX_TASKS {
        return false;
    }

    if matches!(sched.tasks[idx].state, TaskState::Empty | TaskState::Dead) {
        return false;
    }

    if sched.current.iter().any(|current| *current == idx) {
        // Cannot terminate a task currently running on any core.
        return false;
    }

    sched.tasks[idx].state = TaskState::Dead;
    sched.count = sched.count.saturating_sub(1);
    drop(sched);

    crate::serial_println!("[CHRONO] sched: killed task {}", id);
    true
}

/// Call `f(id, name, state)` for every non-empty, non-dead task slot.
pub fn for_each_task(mut f: impl FnMut(u8, &str, TaskState)) {
    let sched = SCHED.lock_irq();
    for task in &sched.tasks {
        match task.state {
            TaskState::Empty | TaskState::Dead => {}
            state => f(task.id, task.name_str(), state),
        }
    }
}

pub fn tasks_per_core() -> [usize; MAX_CORES] {
    let sched = SCHED.lock_irq();
    let mut counts = [0usize; MAX_CORES];
    for task in &sched.tasks {
        if !matches!(task.state, TaskState::Empty | TaskState::Dead)
            && task.core_id < MAX_CORES
        {
            counts[task.core_id] += 1;
        }
    }
    counts
}

fn find_next_ready(sched: &Scheduler, core_id: usize) -> Option<usize> {
    for idx in 0..MAX_TASKS {
        if sched.tasks[idx].state == TaskState::Ready && sched.tasks[idx].core_id == core_id {
            return Some(idx);
        }
    }

    for offset in 1..=MAX_TASKS {
        let current = sched.current[core_id];
        let base = if current == NO_TASK { 0 } else { current };
        let idx = (base + offset) % MAX_TASKS;
        if sched.tasks[idx].state == TaskState::Ready {
            return Some(idx);
        }
    }

    None
}

/// Voluntarily give up the CPU to the next Ready task.
///
/// Performs a round-robin scan starting one slot past the current task. If no
/// other task is Ready the function returns immediately without switching.
/// Every switch is logged to serial: `[CHRONO] sched: switch A -> B`.
pub fn yield_now() {
    switch_from_current();
}

pub fn run_idle_loop() -> ! {
    loop {
        switch_from_current();
        cpu_relax();
    }
}

fn switch_from_current() {
    let core_id = smp::current_core_id();
    let (sched_ptr, restore_interrupts) = unsafe { SCHED.lock_irq_raw() };
    let sched = unsafe { &mut *sched_ptr };
    let current = sched.current[core_id];
    let Some(next) = find_next_ready(sched, core_id) else {
        unsafe {
            SCHED.unlock_irq_raw(restore_interrupts);
        }
        return;
    };

    if current == next {
        unsafe {
            SCHED.unlock_irq_raw(restore_interrupts);
        }
        return;
    }

    let curr_rsp = if current == NO_TASK {
        &mut sched.idle_rsp[core_id] as *mut u64 as usize
    } else {
        sched.tasks[current].state = TaskState::Ready;
        &mut sched.tasks[current].rsp as *mut u64 as usize
    };
    sched.tasks[next].state = TaskState::Running;
    sched.tasks[next].core_id = core_id;
    sched.current[core_id] = next;
    let next_rsp = &sched.tasks[next].rsp as *const u64 as usize;
    let lock = SCHED.raw_lock_byte() as usize;

    if current != NO_TASK {
        crate::serial_println!(
            "[CHRONO] sched: core {} switch {} -> {}",
            core_id,
            sched.tasks[current].name_str(),
            sched.tasks[next].name_str(),
        );
    }

    unsafe {
        // SAFETY: Both pointers come from the static SCHED array. The naked
        // function uses the System V AMD64 calling convention (RDI, RSI).
        context_switch(
            curr_rsp as *mut u64,
            next_rsp as *const u64,
            lock as *mut u8,
            restore_interrupts as u64,
        );
    }
}

fn least_loaded_core(sched: &Scheduler) -> usize {
    let core_count = smp::core_count().min(MAX_CORES).max(1);
    let mut counts = [0usize; MAX_CORES];
    for task in &sched.tasks {
        if !matches!(task.state, TaskState::Empty | TaskState::Dead)
            && task.core_id < core_count
        {
            counts[task.core_id] += 1;
        }
    }

    let mut best = 0usize;
    for core_id in 1..core_count {
        if counts[core_id] < counts[best] {
            best = core_id;
        }
    }

    best
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
unsafe extern "C" fn context_switch(
    current_rsp: *mut u64,
    next_rsp: *const u64,
    sched_lock: *mut u8,
    restore_interrupts: u64,
) {
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
        // Release the scheduler lock only after the outgoing RSP is safe.
        "mov byte ptr [rdx], 0",
        // Switch to the incoming stack (RSI holds next_rsp pointer).
        "mov rsp, [rsi]",
        // Restore the incoming task's callee-saved registers.
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        "test rcx, rcx",
        "jz 2f",
        "sti",
        "2:",
        // Jump into the incoming task: on first run this is its entry function;
        // on subsequent runs this is the instruction after its last `ret`.
        "ret",
    )
}

fn cpu_relax() {
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}
