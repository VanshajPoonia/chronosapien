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

use crate::smp::MAX_CORES;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

const DOUBLE_FAULT_STACK_SIZE: usize = 4096 * 5;
const RING0_STACK_SIZE: usize = 4096 * 5;

#[repr(align(16))]
#[derive(Clone, Copy)]
struct Stack<const SIZE: usize> {
    bytes: [u8; SIZE],
}

static mut DOUBLE_FAULT_STACK: Stack<DOUBLE_FAULT_STACK_SIZE> = Stack {
    bytes: [0; DOUBLE_FAULT_STACK_SIZE],
};
static mut RING0_STACK: Stack<RING0_STACK_SIZE> = Stack {
    bytes: [0; RING0_STACK_SIZE],
};
static mut AP_DOUBLE_FAULT_STACKS: [Stack<DOUBLE_FAULT_STACK_SIZE>; MAX_CORES] =
    [Stack { bytes: [0; DOUBLE_FAULT_STACK_SIZE] }; MAX_CORES];
static mut AP_RING0_STACKS: [Stack<RING0_STACK_SIZE>; MAX_CORES] =
    [Stack { bytes: [0; RING0_STACK_SIZE] }; MAX_CORES];
static mut TSS: [TaskStateSegment; MAX_CORES] =
    [const { TaskStateSegment::new() }; MAX_CORES];
static mut GDT: [GlobalDescriptorTable; MAX_CORES] =
    [const { GlobalDescriptorTable::new() }; MAX_CORES];
static mut SELECTORS: [Option<Selectors>; MAX_CORES] = [None; MAX_CORES];

#[derive(Clone, Copy)]
pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub tss: SegmentSelector,
}

pub fn init_bsp() {
    init_core(0);
    crate::serial_println!("[CHRONO] GDT loaded");
}

pub fn init_ap(core_id: usize) {
    init_core(core_id);
}

pub fn init() {
    init_bsp();
}

fn init_core(core_id: usize) {
    // SAFETY: Descriptor tables are initialized once during early boot before
    // interrupts are enabled. The stack memory and TSS live for the whole
    // kernel lifetime, so the CPU can safely reference them after `lgdt`.
    unsafe {
        let stack = if core_id == 0 {
            core::ptr::addr_of!(DOUBLE_FAULT_STACK.bytes)
        } else {
            core::ptr::addr_of!(AP_DOUBLE_FAULT_STACKS[core_id].bytes)
        };
        let stack_start = VirtAddr::from_ptr(stack as *const u8);
        let stack_end = stack_start + DOUBLE_FAULT_STACK_SIZE;
        TSS[core_id].interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;

        let ring0_stack = if core_id == 0 {
            core::ptr::addr_of!(RING0_STACK.bytes)
        } else {
            core::ptr::addr_of!(AP_RING0_STACKS[core_id].bytes)
        };
        let ring0_stack_start = VirtAddr::from_ptr(ring0_stack as *const u8);
        let ring0_stack_end = ring0_stack_start + RING0_STACK_SIZE;
        TSS[core_id].privilege_stack_table[0] = ring0_stack_end;

        let kernel_code = GDT[core_id].append(Descriptor::kernel_code_segment());
        let kernel_data = GDT[core_id].append(Descriptor::UserSegment(DescriptorFlags::KERNEL_DATA.bits()));
        let user_data = GDT[core_id].append(Descriptor::user_data_segment());
        let user_code = GDT[core_id].append(Descriptor::user_code_segment());
        let tss_selector = GDT[core_id].append(Descriptor::tss_segment(&TSS[core_id]));
        SELECTORS[core_id] = Some(Selectors {
            kernel_code,
            kernel_data,
            user_data,
            user_code,
            tss: tss_selector,
        });

        GDT[core_id].load();
        CS::set_reg(kernel_code);
        load_tss(tss_selector);
    }
}

pub fn selectors() -> Selectors {
    unsafe { SELECTORS[0].expect("GDT selectors initialized") }
}
