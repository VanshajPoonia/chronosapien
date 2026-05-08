//! Symmetric multiprocessing discovery and AP startup.
//!
//! The bootstrap processor (BSP) is the CPU that firmware starts first. The
//! other CPUs are application processors (APs); they sit idle until the BSP
//! sends INIT-SIPI-SIPI startup IPIs through the local APIC.

use core::arch::global_asm;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};

use crate::boot::BootContext;
use crate::memory;

pub const MAX_CORES: usize = 8;

const IA32_APIC_BASE: u32 = 0x1B;
const APIC_BASE_ENABLE: u64 = 1 << 11;

const LAPIC_ID: u64 = 0x020;
const LAPIC_EOI: u64 = 0x0B0;
const LAPIC_SPURIOUS: u64 = 0x0F0;
const LAPIC_ICR_LOW: u64 = 0x300;
const LAPIC_ICR_HIGH: u64 = 0x310;
const LAPIC_ICR_DELIVERY_PENDING: u32 = 1 << 12;
const LAPIC_ENABLE: u32 = 1 << 8;

const ICR_INIT: u32 = 0x0000_0500;
const ICR_STARTUP: u32 = 0x0000_0600;
const ICR_LEVEL_ASSERT: u32 = 1 << 14;
const ICR_TRIGGER_LEVEL: u32 = 1 << 15;

const AP_BOOT_STACK_SIZE: usize = 16 * 1024;
const AP_START_TIMEOUT_MS: u64 = 250;

#[repr(align(16))]
#[derive(Clone, Copy)]
struct ApStack {
    bytes: [u8; AP_BOOT_STACK_SIZE],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct ApBootData {
    pml4: u64,
    stack_top: u64,
    entry: u64,
    core_id: u64,
}

static CORE_COUNT: AtomicUsize = AtomicUsize::new(1);
static ONLINE_COUNT: AtomicUsize = AtomicUsize::new(1);
static BSP_APIC_ID: AtomicU8 = AtomicU8::new(0);
static LAPIC_VIRT: AtomicU64 = AtomicU64::new(0);
static SMP_READY: AtomicBool = AtomicBool::new(false);

static CORE_APIC_IDS: [AtomicU8; MAX_CORES] = [
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
];
static CORE_ONLINE: [AtomicBool; MAX_CORES] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];
static mut AP_STACKS: [ApStack; MAX_CORES] = [ApStack {
    bytes: [0; AP_BOOT_STACK_SIZE],
}; MAX_CORES];

global_asm!(
    r#"
    .intel_syntax noprefix
    .section .ap_trampoline, "ax"
    .global __ap_trampoline_start
    .global __ap_trampoline_end

__ap_trampoline_start:
    .code16
    cli
    cld
    xor ax, ax
    mov ds, ax
    lgdt [0x8000 + ap_gdt_descriptor - __ap_trampoline_start]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    ljmp 0x08, 0x8000 + ap_protected - __ap_trampoline_start

ap_protected:
    .code32
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    mov eax, cr4
    or eax, 0x20
    mov cr4, eax

    mov eax, dword ptr [0x7000]
    mov cr3, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 0x100
    wrmsr

    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax
    ljmp 0x18, 0x8000 + ap_long - __ap_trampoline_start

ap_long:
    .code64
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov rsp, qword ptr [0x7000 + 8]
    mov rax, qword ptr [0x7000 + 16]
    mov rdi, qword ptr [0x7000 + 24]
    jmp rax

    .align 8
ap_gdt:
    .quad 0
    .quad 0x00cf9a000000ffff
    .quad 0x00cf92000000ffff
    .quad 0x00af9a000000ffff
ap_gdt_descriptor:
    .word ap_gdt_descriptor - ap_gdt - 1
    .long 0x8000 + ap_gdt - __ap_trampoline_start

__ap_trampoline_end:
    .code64
    .att_syntax prefix
"#
);

extern "C" {
    static __ap_trampoline_start: u8;
    static __ap_trampoline_end: u8;
}

pub fn init_bsp(boot_context: &'static BootContext) {
    let bsp_apic_id = initial_apic_id();
    BSP_APIC_ID.store(bsp_apic_id, Ordering::SeqCst);
    CORE_APIC_IDS[0].store(bsp_apic_id, Ordering::SeqCst);
    CORE_ONLINE[0].store(true, Ordering::SeqCst);

    crate::serial_println!("[CHRONO] smp: BSP online (core 0)");

    let Some(physical_memory_offset) = boot_context.physical_memory_offset else {
        crate::serial_println!("[CHRONO] smp: physical memory offset missing; single core");
        return;
    };

    let Some(madt) = acpi::discover_madt(boot_context, physical_memory_offset) else {
        crate::serial_println!("[CHRONO] smp: MADT unavailable; single core");
        return;
    };

    LAPIC_VIRT.store(
        physical_memory_offset + madt.local_apic_address,
        Ordering::SeqCst,
    );
    enable_local_apic();

    let mut count = 1usize;
    for apic_id in madt.apic_ids[..madt.apic_id_count].iter().copied() {
        if apic_id == bsp_apic_id || count >= MAX_CORES {
            continue;
        }
        CORE_APIC_IDS[count].store(apic_id, Ordering::SeqCst);
        count += 1;
    }

    CORE_COUNT.store(count, Ordering::SeqCst);
}

pub fn start_aps() {
    let count = CORE_COUNT.load(Ordering::SeqCst);
    if count <= 1 {
        crate::serial_println!("[CHRONO] smp: 1 core ready");
        SMP_READY.store(true, Ordering::SeqCst);
        return;
    }

    if !memory::identity_map_physical_range(
        memory::SMP_BOOT_DATA_PHYS,
        memory::SMP_TRAMPOLINE_PHYS + memory::SMP_TRAMPOLINE_SIZE,
    ) {
        crate::serial_println!("[CHRONO] smp: trampoline mapping failed; single core");
        CORE_COUNT.store(1, Ordering::SeqCst);
        SMP_READY.store(true, Ordering::SeqCst);
        return;
    }

    copy_trampoline();
    let (pml4, _) = memory::active_cr3();

    for core_id in 1..count {
        let apic_id = CORE_APIC_IDS[core_id].load(Ordering::SeqCst);
        write_boot_data(core_id, pml4.start_address().as_u64());
        send_init_sipi_sipi(apic_id);
        wait_for_core(core_id);
    }

    let ready = ONLINE_COUNT.load(Ordering::SeqCst);
    CORE_COUNT.store(ready, Ordering::SeqCst);
    crate::serial_println!("[CHRONO] smp: {} cores ready", ready);
    SMP_READY.store(true, Ordering::SeqCst);
}

pub fn core_count() -> usize {
    CORE_COUNT.load(Ordering::SeqCst)
}

pub fn current_core_id() -> usize {
    let lapic = LAPIC_VIRT.load(Ordering::Relaxed);
    if lapic == 0 {
        return 0;
    }

    let apic_id = unsafe { read_lapic(lapic, LAPIC_ID) >> 24 } as u8;
    for core_id in 0..CORE_COUNT.load(Ordering::SeqCst) {
        if CORE_APIC_IDS[core_id].load(Ordering::SeqCst) == apic_id {
            return core_id;
        }
    }

    0
}

pub fn mark_core_online(core_id: usize) {
    if core_id >= MAX_CORES {
        return;
    }

    if !CORE_ONLINE[core_id].swap(true, Ordering::SeqCst) {
        ONLINE_COUNT.fetch_add(1, Ordering::SeqCst);
    }
    crate::serial_println!("[CHRONO] smp: core {} online", core_id);
}

pub fn tasks_per_core() -> [usize; MAX_CORES] {
    crate::sched::tasks_per_core()
}

#[no_mangle]
pub extern "C" fn chrono_ap_main(core_id: usize) -> ! {
    crate::gdt::init_ap(core_id);
    crate::interrupts::init_ap(core_id);
    enable_local_apic();
    mark_core_online(core_id);
    x86_64::instructions::interrupts::enable();
    crate::sched::run_idle_loop()
}

pub fn eoi() {
    let lapic = LAPIC_VIRT.load(Ordering::Relaxed);
    if lapic != 0 {
        unsafe {
            write_lapic(lapic, LAPIC_EOI, 0);
        }
    }
}

fn copy_trampoline() {
    let start = unsafe { &__ap_trampoline_start as *const u8 as usize };
    let end = unsafe { &__ap_trampoline_end as *const u8 as usize };
    let len = end - start;
    assert!(len <= memory::SMP_TRAMPOLINE_SIZE as usize);

    let destination = memory::phys_to_mut(memory::SMP_TRAMPOLINE_PHYS)
        .expect("SMP trampoline destination mapped");
    unsafe {
        core::ptr::copy_nonoverlapping(start as *const u8, destination, len);
    }
}

fn write_boot_data(core_id: usize, pml4: u64) {
    let stack_top = unsafe {
        AP_STACKS[core_id]
            .bytes
            .as_ptr()
            .add(AP_BOOT_STACK_SIZE) as u64
    };
    let data = ApBootData {
        pml4,
        stack_top,
        entry: chrono_ap_main as usize as u64,
        core_id: core_id as u64,
    };
    let destination =
        memory::phys_to_mut(memory::SMP_BOOT_DATA_PHYS).expect("SMP boot data mapped");
    unsafe {
        core::ptr::write_volatile(destination as *mut ApBootData, data);
    }
}

fn wait_for_core(core_id: usize) {
    let start = crate::timer::ticks();
    let timeout_ticks = (AP_START_TIMEOUT_MS * crate::timer::PIT_HZ).div_ceil(1000);
    while crate::timer::ticks().saturating_sub(start) < timeout_ticks {
        if CORE_ONLINE[core_id].load(Ordering::SeqCst) {
            return;
        }
        core::hint::spin_loop();
    }
    crate::serial_println!("[CHRONO] smp: core {} startup timed out", core_id);
}

fn send_init_sipi_sipi(apic_id: u8) {
    let vector = (memory::SMP_TRAMPOLINE_PHYS / 0x1000) as u32;

    send_ipi(apic_id, ICR_INIT | ICR_LEVEL_ASSERT | ICR_TRIGGER_LEVEL);
    crate::timer::sleep_ms(10);
    send_ipi(apic_id, ICR_STARTUP | ICR_LEVEL_ASSERT | vector);
    crate::timer::sleep_ms(1);
    send_ipi(apic_id, ICR_STARTUP | ICR_LEVEL_ASSERT | vector);
    crate::timer::sleep_ms(1);
}

fn send_ipi(apic_id: u8, command: u32) {
    let lapic = LAPIC_VIRT.load(Ordering::SeqCst);
    if lapic == 0 {
        return;
    }

    unsafe {
        wait_icr_idle(lapic);
        write_lapic(lapic, LAPIC_ICR_HIGH, (apic_id as u32) << 24);
        write_lapic(lapic, LAPIC_ICR_LOW, command);
        wait_icr_idle(lapic);
    }
}

unsafe fn wait_icr_idle(lapic: u64) {
    while read_lapic(lapic, LAPIC_ICR_LOW) & LAPIC_ICR_DELIVERY_PENDING != 0 {
        core::hint::spin_loop();
    }
}

fn enable_local_apic() {
    unsafe {
        let value = rdmsr(IA32_APIC_BASE);
        wrmsr(IA32_APIC_BASE, value | APIC_BASE_ENABLE);
    }

    let lapic = LAPIC_VIRT.load(Ordering::SeqCst);
    if lapic != 0 {
        unsafe {
            write_lapic(lapic, LAPIC_SPURIOUS, read_lapic(lapic, LAPIC_SPURIOUS) | LAPIC_ENABLE | 0xFF);
        }
    }
}

fn initial_apic_id() -> u8 {
    unsafe {
        let cpuid = core::arch::x86_64::__cpuid(1);
        (cpuid.ebx >> 24) as u8
    }
}

unsafe fn read_lapic(lapic: u64, offset: u64) -> u32 {
    core::ptr::read_volatile((lapic + offset) as *const u32)
}

unsafe fn write_lapic(lapic: u64, offset: u64, value: u32) {
    core::ptr::write_volatile((lapic + offset) as *mut u32, value);
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
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") value as u32,
        in("edx") (value >> 32) as u32,
        options(nomem, nostack, preserves_flags)
    );
}

mod acpi {
    use super::MAX_CORES;
    use crate::boot::BootContext;
    const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";
    const SDT_HEADER_LEN: usize = 36;
    const MADT_ENTRY_START: usize = 44;

    pub struct MadtInfo {
        pub local_apic_address: u64,
        pub apic_ids: [u8; MAX_CORES],
        pub apic_id_count: usize,
    }

    pub fn discover_madt(
        boot_context: &'static BootContext,
        physical_memory_offset: u64,
    ) -> Option<MadtInfo> {
        let rsdp = boot_context.rsdp_addr?;
        let rsdp_ptr = phys_ptr(physical_memory_offset, rsdp);
        let rsdp_bytes = unsafe { core::slice::from_raw_parts(rsdp_ptr, 36) };
        if &rsdp_bytes[0..8] != RSDP_SIGNATURE || !checksum(&rsdp_bytes[..20]) {
            return None;
        }

        let revision = rsdp_bytes[15];
        if revision >= 2 {
            let length = read_u32(rsdp_bytes, 20) as usize;
            let bytes = unsafe { core::slice::from_raw_parts(rsdp_ptr, length) };
            if checksum(bytes) {
                let xsdt = read_u64(bytes, 24);
                if let Some(madt) = find_table(physical_memory_offset, xsdt, true, b"APIC") {
                    return parse_madt(physical_memory_offset, madt);
                }
            }
        }

        let rsdt = read_u32(rsdp_bytes, 16) as u64;
        find_table(physical_memory_offset, rsdt, false, b"APIC")
            .and_then(|madt| parse_madt(physical_memory_offset, madt))
    }

    fn find_table(
        physical_memory_offset: u64,
        table_phys: u64,
        xsdt: bool,
        signature: &[u8; 4],
    ) -> Option<u64> {
        let header = read_sdt(physical_memory_offset, table_phys)?;
        if !checksum(header) {
            return None;
        }

        let entry_size = if xsdt { 8 } else { 4 };
        let mut offset = SDT_HEADER_LEN;
        while offset + entry_size <= header.len() {
            let candidate = if xsdt {
                read_u64(header, offset)
            } else {
                read_u32(header, offset) as u64
            };
            let candidate_header = read_sdt(physical_memory_offset, candidate)?;
            if &candidate_header[0..4] == signature && checksum(candidate_header) {
                return Some(candidate);
            }
            offset += entry_size;
        }

        None
    }

    fn parse_madt(physical_memory_offset: u64, madt_phys: u64) -> Option<MadtInfo> {
        let bytes = read_sdt(physical_memory_offset, madt_phys)?;
        let mut info = MadtInfo {
            local_apic_address: read_u32(bytes, 36) as u64,
            apic_ids: [0; MAX_CORES],
            apic_id_count: 0,
        };

        let mut offset = MADT_ENTRY_START;
        while offset + 2 <= bytes.len() {
            let entry_type = bytes[offset];
            let entry_len = bytes[offset + 1] as usize;
            if entry_len < 2 || offset + entry_len > bytes.len() {
                break;
            }

            match entry_type {
                0 if entry_len >= 8 => {
                    let apic_id = bytes[offset + 3];
                    let flags = read_u32(bytes, offset + 4);
                    if flags & 0b11 != 0 {
                        push_apic_id(&mut info, apic_id);
                    }
                }
                5 if entry_len >= 12 => {
                    info.local_apic_address = read_u64(bytes, offset + 4);
                }
                9 if entry_len >= 16 => {
                    let flags = read_u32(bytes, offset + 4);
                    let apic_id = read_u32(bytes, offset + 8);
                    if flags & 0b11 != 0 && apic_id <= u8::MAX as u32 {
                        push_apic_id(&mut info, apic_id as u8);
                    }
                }
                _ => {}
            }

            offset += entry_len;
        }

        Some(info)
    }

    fn push_apic_id(info: &mut MadtInfo, apic_id: u8) {
        if info.apic_ids[..info.apic_id_count].contains(&apic_id)
            || info.apic_id_count >= MAX_CORES
        {
            return;
        }

        info.apic_ids[info.apic_id_count] = apic_id;
        info.apic_id_count += 1;
    }

    fn read_sdt(physical_memory_offset: u64, physical_address: u64) -> Option<&'static [u8]> {
        let header = unsafe {
            core::slice::from_raw_parts(phys_ptr(physical_memory_offset, physical_address), SDT_HEADER_LEN)
        };
        let length = read_u32(header, 4) as usize;
        Some(unsafe {
            core::slice::from_raw_parts(phys_ptr(physical_memory_offset, physical_address), length)
        })
    }

    fn phys_ptr(physical_memory_offset: u64, physical_address: u64) -> *const u8 {
        (physical_memory_offset + physical_address) as *const u8
    }

    fn checksum(bytes: &[u8]) -> bool {
        bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)) == 0
    }

    fn read_u32(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    fn read_u64(bytes: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
    }
}
