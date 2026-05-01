//! Global Descriptor Table setup.
//!
//! In 64-bit mode, x86 mostly stops using segmentation for everyday memory
//! addressing, but the CPU still needs valid segment descriptors. We also need
//! a Task State Segment (TSS) descriptor so the CPU can switch to a known-good
//! stack for critical exceptions such as double faults.

use x86_64::instructions::segmentation::{Segment, CS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

const DOUBLE_FAULT_STACK_SIZE: usize = 4096 * 5;

#[repr(align(16))]
struct InterruptStack {
    bytes: [u8; DOUBLE_FAULT_STACK_SIZE],
}

static mut DOUBLE_FAULT_STACK: InterruptStack = InterruptStack {
    bytes: [0; DOUBLE_FAULT_STACK_SIZE],
};
static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

pub fn init() {
    // SAFETY: Descriptor tables are initialized once during early boot before
    // interrupts are enabled. The stack memory and TSS live for the whole
    // kernel lifetime, so the CPU can safely reference them after `lgdt`.
    unsafe {
        let stack_start =
            VirtAddr::from_ptr(core::ptr::addr_of!(DOUBLE_FAULT_STACK.bytes) as *const u8);
        let stack_end = stack_start + DOUBLE_FAULT_STACK_SIZE;
        TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;

        let code_selector = GDT.append(Descriptor::kernel_code_segment());
        let tss_selector = GDT.append(Descriptor::tss_segment(&TSS));

        GDT.load();
        CS::set_reg(code_selector);
        load_tss(tss_selector);
    }

    crate::serial_println!("[CHRONO] GDT loaded");
}
