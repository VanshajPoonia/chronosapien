//! CPU exception handling through the Interrupt Descriptor Table.
//!
//! The IDT is the CPU's vector table for exceptions and interrupts. Each entry
//! points at a handler with the special x86 interrupt ABI, and `lidt` tells the
//! CPU where the table lives.

use x86_64::instructions;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};

use crate::{gdt, mouse, pic, ring3, timer};
use crate::smp::MAX_CORES;

const TIMER_INTERRUPT_VECTOR: usize = pic::MASTER_PIC_OFFSET as usize;
const MOUSE_INTERRUPT_VECTOR: usize = pic::SLAVE_PIC_OFFSET as usize + 4;

static mut IDT: [InterruptDescriptorTable; MAX_CORES] =
    [const { InterruptDescriptorTable::new() }; MAX_CORES];

pub fn init_bsp() {
    init_core(0);
    crate::serial_println!("[CHRONO] IDT loaded");
}

pub fn init_ap(core_id: usize) {
    init_core(core_id);
}

pub fn init() {
    init_bsp();
}

fn init_core(core_id: usize) {
    // SAFETY: The IDT is built once during early boot. The handler functions
    // have the ABI expected by the CPU, and the table remains static forever.
    unsafe {
        IDT[core_id].breakpoint.set_handler_fn(breakpoint_handler);
        IDT[core_id].general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        IDT[core_id].page_fault.set_handler_fn(page_fault_handler);
        IDT[core_id].double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        IDT[core_id][TIMER_INTERRUPT_VECTOR].set_handler_fn(timer_interrupt_handler);
        IDT[core_id][MOUSE_INTERRUPT_VECTOR].set_handler_fn(mouse_interrupt_handler);
        IDT[core_id].load();
    }
}

pub fn trigger_test_breakpoint() {
    instructions::interrupts::int3();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::println!("EXCEPTION: BREAKPOINT");
    crate::serial_println!(
        "[CHRONO] interrupt: breakpoint at {:#x}",
        stack_frame.instruction_pointer.as_u64()
    );
    crate::serial_println!("[CHRONO] breakpoint resolved");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let fault_address = Cr2::read();

    crate::println!("EXCEPTION: PAGE FAULT");
    crate::println!("fault address: {:#x}", fault_address.as_u64());
    crate::serial_println!(
        "[CHRONO] interrupt: page fault at {:#x} error={:?}",
        fault_address.as_u64(),
        error_code
    );
    crate::serial_println!("[CHRONO] page fault stack frame: {:#?}", stack_frame);

    halt_forever();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    mut stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    let rip = stack_frame.instruction_pointer.as_u64();

    if ring3::is_demo_privilege_fault(rip) {
        crate::serial_println!("[CHRONO] ring3: transition ok");
        crate::serial_println!(
            "[CHRONO] ring3: privilege violation caught — GP fault at {:#x}",
            rip
        );
        unsafe {
            stack_frame
                .as_mut()
                .update(|frame| frame.instruction_pointer += 1u64);
        }
        return;
    }

    crate::println!("EXCEPTION: GENERAL PROTECTION FAULT");
    crate::serial_println!(
        "[CHRONO] interrupt: general protection fault at {:#x} error={:#x}",
        rip,
        error_code
    );
    crate::serial_println!("[CHRONO] #GP stack frame: {:#?}", stack_frame);

    halt_forever();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    crate::println!("EXCEPTION: DOUBLE FAULT");
    crate::serial_println!(
        "[CHRONO] interrupt: double fault error={:#x} stack_frame={:#?}",
        error_code,
        stack_frame
    );

    halt_forever();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    timer::handle_tick();
    crate::smp::eoi();
    pic::end_of_interrupt(pic::TIMER_IRQ);
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    mouse::handle_interrupt();
    pic::end_of_interrupt(pic::MOUSE_IRQ);
}

fn halt_forever() -> ! {
    loop {
        // SAFETY: The kernel cannot recover from these exceptions yet. Halting
        // keeps QEMU inspectable instead of spinning at full CPU usage.
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
