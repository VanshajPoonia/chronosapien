//! Foreground ELF process execution.

use core::cell::UnsafeCell;

use x86_64::structures::paging::{PageTableFlags, PhysFrame, Size4KiB};

use crate::{elf, gdt, memory};

const MAX_USER_RANGES: usize = 16;

#[derive(Clone, Copy)]
struct UserRange {
    start: u64,
    end: u64,
}

impl UserRange {
    const fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    fn contains(&self, start: u64, end: u64) -> bool {
        self.start <= start && end <= self.end
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KernelContext {
    rsp: u64,
    rbp: u64,
    rbx: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rip: u64,
}

impl KernelContext {
    const fn empty() -> Self {
        Self {
            rsp: 0,
            rbp: 0,
            rbx: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
        }
    }
}

struct ProcessState {
    active: bool,
    kernel_cr3_start: u64,
    kernel_cr3_flags_bits: u64,
    context: KernelContext,
    exit_code: u64,
    ranges: [UserRange; MAX_USER_RANGES],
    range_count: usize,
}

impl ProcessState {
    const fn new() -> Self {
        Self {
            active: false,
            kernel_cr3_start: 0,
            kernel_cr3_flags_bits: 0,
            context: KernelContext::empty(),
            exit_code: 0,
            ranges: [UserRange::empty(); MAX_USER_RANGES],
            range_count: 0,
        }
    }

    fn clear_ranges(&mut self) {
        self.ranges = [UserRange::empty(); MAX_USER_RANGES];
        self.range_count = 0;
    }

    fn add_range(&mut self, start: u64, end: u64) -> Result<(), ExecError> {
        if self.range_count >= MAX_USER_RANGES {
            return Err(ExecError::TooManyRanges);
        }

        self.ranges[self.range_count] = UserRange { start, end };
        self.range_count += 1;
        Ok(())
    }
}

struct GlobalProcess(UnsafeCell<ProcessState>);

unsafe impl Sync for GlobalProcess {}

static PROCESS: GlobalProcess = GlobalProcess(UnsafeCell::new(ProcessState::new()));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecError {
    AlreadyRunning,
    BadElf(elf::ElfError),
    OutOfMemory,
    TooManyRanges,
}

pub fn exec_elf(name: &str, bytes: &[u8]) -> Result<u64, ExecError> {
    if is_active() {
        return Err(ExecError::AlreadyRunning);
    }

    let image = elf::parse(bytes).map_err(ExecError::BadElf)?;
    crate::serial_println!("[CHRONO] elf: loading {}, entry {:#x}", name, image.entry);

    let address_space = memory::create_user_address_space().ok_or(ExecError::OutOfMemory)?;

    {
        let state = unsafe { &mut *PROCESS.0.get() };
        state.clear_ranges();

        for segment in &image.segments {
            map_segment(address_space, bytes, *segment)?;
            let start = align_down(segment.virtual_address);
            let end = align_up(segment.virtual_address + segment.memory_size);
            state.add_range(start, end)?;
            crate::serial_println!(
                "[CHRONO] elf: PT_LOAD vaddr={:#x} mem={} file={} flags={:#x}",
                segment.virtual_address,
                segment.memory_size,
                segment.file_size,
                segment.flags
            );
        }

        map_stack(address_space)?;
        state.add_range(
            memory::USER_ELF_STACK_TOP - memory::USER_ELF_STACK_SIZE,
            memory::USER_ELF_STACK_TOP,
        )?;

        let (kernel_cr3, kernel_cr3_flags) = memory::active_cr3();
        state.kernel_cr3_start = kernel_cr3.start_address().as_u64();
        state.kernel_cr3_flags_bits = kernel_cr3_flags.bits();
        state.exit_code = 0;
        state.active = true;
    }

    let resumed = unsafe { save_context(context_ptr()) };
    if resumed != 0 {
        let state = unsafe { &mut *PROCESS.0.get() };
        let code = state.exit_code;
        state.active = false;
        state.clear_ranges();
        crate::serial_println!("[CHRONO] elf: process exited code={}", code);
        return Ok(code);
    }

    unsafe {
        memory::switch_cr3(address_space.pml4_frame, x86_64::registers::control::Cr3Flags::empty());
        enter_user_mode(image.entry, memory::USER_ELF_STACK_TOP);
    }
}

pub fn exit_current_if_active(code: u64) -> bool {
    let state = unsafe { &mut *PROCESS.0.get() };
    if !state.active {
        return false;
    }

    state.exit_code = code;
    let kernel_cr3 = PhysFrame::containing_address(x86_64::PhysAddr::new(state.kernel_cr3_start));
    let kernel_cr3_flags =
        x86_64::registers::control::Cr3Flags::from_bits_truncate(state.kernel_cr3_flags_bits);
    let context = &state.context as *const KernelContext;

    unsafe {
        memory::switch_cr3(kernel_cr3, kernel_cr3_flags);
        restore_context(context, 1)
    }
}

pub fn user_range_is_valid(start: u64, len: u64) -> bool {
    let Some(end) = start.checked_add(len) else {
        return false;
    };

    let state = unsafe { &*PROCESS.0.get() };
    if !state.active {
        return false;
    }

    state.ranges[..state.range_count]
        .iter()
        .any(|range| range.contains(start, end))
}

pub fn is_active() -> bool {
    unsafe { (&*PROCESS.0.get()).active }
}

fn map_segment(
    address_space: memory::AddressSpace,
    bytes: &[u8],
    segment: elf::ProgramSegment,
) -> Result<(), ExecError> {
    let start = align_down(segment.virtual_address);
    let end = align_up(segment.virtual_address + segment.memory_size);
    let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
    if segment.flags & elf::PF_W != 0 {
        flags |= PageTableFlags::WRITABLE;
    }

    let mut page_address = start;
    while page_address < end {
        let frame = memory::map_user_frame(address_space, page_address, flags)
            .ok_or(ExecError::OutOfMemory)?;
        copy_segment_page(frame, page_address, bytes, segment)?;
        page_address += memory::PAGE_SIZE;
    }

    Ok(())
}

fn copy_segment_page(
    frame: PhysFrame<Size4KiB>,
    page_address: u64,
    bytes: &[u8],
    segment: elf::ProgramSegment,
) -> Result<(), ExecError> {
    let file_start = segment.virtual_address;
    let file_end = segment.virtual_address + segment.file_size;
    let page_end = page_address + memory::PAGE_SIZE;
    let copy_start = page_address.max(file_start);
    let copy_end = page_end.min(file_end);

    if copy_start >= copy_end {
        return Ok(());
    }

    let source_start = (segment.offset + (copy_start - segment.virtual_address)) as usize;
    let source_end = source_start + (copy_end - copy_start) as usize;
    let destination_offset = (copy_start - page_address) as usize;

    if !memory::copy_to_frame(frame, destination_offset, &bytes[source_start..source_end]) {
        return Err(ExecError::OutOfMemory);
    }

    Ok(())
}

fn map_stack(address_space: memory::AddressSpace) -> Result<(), ExecError> {
    let stack_start = memory::USER_ELF_STACK_TOP - memory::USER_ELF_STACK_SIZE;
    let mut address = stack_start;

    while address < memory::USER_ELF_STACK_TOP {
        memory::map_user_frame(
            address_space,
            address,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
        )
        .ok_or(ExecError::OutOfMemory)?;
        address += memory::PAGE_SIZE;
    }

    Ok(())
}

fn align_down(address: u64) -> u64 {
    address & !(memory::PAGE_SIZE - 1)
}

fn align_up(address: u64) -> u64 {
    (address + memory::PAGE_SIZE - 1) & !(memory::PAGE_SIZE - 1)
}

unsafe fn enter_user_mode(entry: u64, stack_top: u64) -> ! {
    let selectors = gdt::selectors();

    crate::serial_println!("[CHRONO] kernel: entered ELF ring 3");

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
        user_stack = in(reg) stack_top,
        user_code = in(reg) selectors.user_code.0 as u64,
        user_entry = in(reg) entry,
        out("rax") _,
        options(noreturn),
    );
}

fn context_ptr() -> *mut KernelContext {
    let state = unsafe { &mut *PROCESS.0.get() };
    &mut state.context
}

#[inline(never)]
unsafe fn save_context(context: *mut KernelContext) -> u64 {
    let result: u64;

    core::arch::asm!(
        "mov [rdi + 0x00], rsp",
        "mov [rdi + 0x08], rbp",
        "mov [rdi + 0x10], rbx",
        "mov [rdi + 0x18], r12",
        "mov [rdi + 0x20], r13",
        "mov [rdi + 0x28], r14",
        "mov [rdi + 0x30], r15",
        "lea rax, [rip + 2f]",
        "mov [rdi + 0x38], rax",
        "xor eax, eax",
        "2:",
        in("rdi") context,
        lateout("rax") result,
    );

    result
}

unsafe fn restore_context(context: *const KernelContext, value: u64) -> ! {
    core::arch::asm!(
        "mov rsp, [rdi + 0x00]",
        "mov rbp, [rdi + 0x08]",
        "mov rbx, [rdi + 0x10]",
        "mov r12, [rdi + 0x18]",
        "mov r13, [rdi + 0x20]",
        "mov r14, [rdi + 0x28]",
        "mov r15, [rdi + 0x30]",
        "mov rax, rsi",
        "jmp qword ptr [rdi + 0x38]",
        in("rdi") context,
        in("rsi") value,
        options(noreturn),
    )
}
