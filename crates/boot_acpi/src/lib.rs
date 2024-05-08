#![no_std]

extern crate alloc;

use core::arch::global_asm;

use acpi::{platform::ProcessorState, AcpiHandler, AcpiTables};
use memory::phys::PhysAddr;
use x86::{
    apic::{x2apic::X2APIC, xapic::XAPIC},
    cpuid::CpuId,
    msr::{rdmsr, IA32_APIC_BASE},
};

pub mod ap_startup;
pub mod handler;

#[cfg(target_arch = "x86_64")]
global_asm!(
    include_str!("asm/trampoline_x86_64.asm"),
    options(att_syntax)
);

#[cfg(target_arch = "x86")]
global_asm!(include_str!("asm/trampoline_i686.asm"), options(att_syntax));

pub enum ApicMode {
    X2Apic(X2APIC),
    Apic(XAPIC),
}

pub fn number_of_cores<H: AcpiHandler>(acpi_tables: &AcpiTables<H>) -> Option<usize> {
    let cnt = acpi_tables
        .platform_info()
        .ok()?
        .processor_info?
        .application_processors
        .iter()
        .filter(|&ap| ap.state == ProcessorState::WaitingForSipi)
        .count()
        + 1; // + 1 for the BSP

    Some(cnt)
}

pub fn get_local_apic(use_higher_half: bool) -> Option<ApicMode> {
    let cpuid = CpuId::new();

    let feature_info = cpuid.get_feature_info()?;

    let has_apic = feature_info.has_apic();
    let has_x2apic = feature_info.has_x2apic();

    if has_x2apic {
        let mut x2apic = X2APIC::new();

        // enable x2apic mode
        // Note: this means accessing the apic in memory mapped mode is no longer valid
        x2apic.attach();

        Some(ApicMode::X2Apic(x2apic))
    } else if has_apic {
        let base = unsafe { rdmsr(IA32_APIC_BASE) };
        let phys_addr = PhysAddr::new(base & 0xfffff000);

        let virt_addr = if use_higher_half {
            phys_addr.to_higher_half_checked()?
        } else {
            phys_addr.to_virt_checked()?
        };

        let ptr = virt_addr.as_ptr_mut::<u32>();
        // Note: I'm not 100% sure on the size of this slice, but since it is never really used as
        // one it shouldn't really matter
        let size = 0x400;

        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, size) };
        let mut xapic = XAPIC::new(slice);
        xapic.attach();
        Some(ApicMode::Apic(xapic))
    } else {
        None
    }
}
