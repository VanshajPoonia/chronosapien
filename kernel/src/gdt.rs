//! Global Descriptor Table setup.
//!
//! In 64-bit mode, x86 mostly stops using segmentation for everyday memory
//! addressing, but the CPU still needs valid segment descriptors. We also need
//! a Task State Segment (TSS) descriptor so the CPU can switch to a known-good
//! stack for critical exceptions and ring 3 -> ring 0 transitions.

use x86_64::instructions::segmentation::{Segment, CS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{
    Descriptor, DescriptorFlags, GlobalDescriptorTable, SegmentSelector,
};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

const DOUBLE_FAULT_STACK_SIZE: usize = 4096 * 5;
const RING0_STACK_SIZE: usize = 4096 * 5;

#[repr(align(16))]
struct Stack<const SIZE: usize> {
    bytes: [u8; SIZE],
}

static mut DOUBLE_FAULT_STACK: Stack<DOUBLE_FAULT_STACK_SIZE> = Stack {
    bytes: [0; DOUBLE_FAULT_STACK_SIZE],
};
static mut RING0_STACK: Stack<RING0_STACK_SIZE> = Stack {
    bytes: [0; RING0_STACK_SIZE],
};
static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
static mut SELECTORS: Option<Selectors> = None;

#[derive(Clone, Copy)]
pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub tss: SegmentSelector,
}

pub fn init() {
    // SAFETY: Descriptor tables are initialized once during early boot before
    // interrupts are enabled. The stack memory and TSS live for the whole
    // kernel lifetime, so the CPU can safely reference them after `lgdt`.
    unsafe {
        let stack_start =
            VirtAddr::from_ptr(core::ptr::addr_of!(DOUBLE_FAULT_STACK.bytes) as *const u8);
        let stack_end = stack_start + DOUBLE_FAULT_STACK_SIZE;
        TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;

        let ring0_stack_start =
            VirtAddr::from_ptr(core::ptr::addr_of!(RING0_STACK.bytes) as *const u8);
        let ring0_stack_end = ring0_stack_start + RING0_STACK_SIZE;
        TSS.privilege_stack_table[0] = ring0_stack_end;

        let kernel_code = GDT.append(Descriptor::kernel_code_segment());
        let kernel_data = GDT.append(Descriptor::UserSegment(DescriptorFlags::KERNEL_DATA.bits()));
        let user_data = GDT.append(Descriptor::user_data_segment());
        let user_code = GDT.append(Descriptor::user_code_segment());
        let tss_selector = GDT.append(Descriptor::tss_segment(&TSS));
        SELECTORS = Some(Selectors {
            kernel_code,
            kernel_data,
            user_data,
            user_code,
            tss: tss_selector,
        });

        GDT.load();
        CS::set_reg(kernel_code);
        load_tss(tss_selector);
    }

    crate::serial_println!("[CHRONO] GDT loaded");
}

pub fn selectors() -> Selectors {
    unsafe { SELECTORS.expect("GDT selectors initialized") }
}
