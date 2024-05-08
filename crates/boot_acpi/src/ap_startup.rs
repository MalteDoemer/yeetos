use crate::ApicMode;
use acpi::{platform::ProcessorState, AcpiHandler, AcpiTables};
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use kernel_image::KernelImage;
use memory::phys::PhysAddr;
use x86::apic::{ApicControl, ApicId};

#[no_mangle]
static KERNEL_STACKS_VADDR: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
static KERNEL_STACK_SIZE: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
static NUM_CORES: AtomicU32 = AtomicU32::new(0);

#[no_mangle]
static PAGE_TABLE_ADDRESS: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Copy, Clone)]
pub enum ApStartupError {
    BadPageTableAddress,
    NumCoresTooBig,
    AcpiPlatformInfoNotPresent,
    AcpiProcessorInfoNotAvailable,
    ApicNotPresent,
}

pub fn startup_all_application_processors<H: AcpiHandler, SleepMicroSecondsFn: Fn(u64) + Copy>(
    acpi_tables: &AcpiTables<H>,
    kernel_image: &KernelImage,
    page_table_address: PhysAddr,
    sleep_us: SleepMicroSecondsFn,
) -> Result<(), ApStartupError> {
    let num_cores = kernel_image.num_cores();
    let stack_size = kernel_image.kernel_stack_size();
    let stacks_addr = kernel_image
        .kernel_image_info()
        .stack
        .start_addr()
        .to_inner();

    let page_table_address = page_table_address
        .to_inner()
        .try_into()
        .map_err(|_| ApStartupError::BadPageTableAddress)?;

    if num_cores >= 256 {
        return Err(ApStartupError::NumCoresTooBig);
    }

    // Store some global information that the AP's can later access once they are started
    // Note: check that relaxed is okay here??
    KERNEL_STACKS_VADDR.store(stacks_addr, Ordering::Relaxed);
    KERNEL_STACK_SIZE.store(stack_size, Ordering::Relaxed);
    NUM_CORES.store(num_cores as u32, Ordering::Relaxed);
    PAGE_TABLE_ADDRESS.store(page_table_address, Ordering::Relaxed);

    let processor_info = acpi_tables
        .platform_info()
        .map_err(|_| ApStartupError::AcpiPlatformInfoNotPresent)?
        .processor_info
        .ok_or(ApStartupError::AcpiProcessorInfoNotAvailable)?;

    install_ap_trampoline();

    let mut local_apic = crate::get_local_apic(false).ok_or(ApStartupError::ApicNotPresent)?;

    let aps = processor_info
        .application_processors
        .iter()
        .filter(|&ap| ap.state == ProcessorState::WaitingForSipi);

    match local_apic {
        ApicMode::X2Apic(ref mut x2apic) => {
            for ap in aps {
                let id = ApicId::X2Apic(ap.local_apic_id);
                unsafe {
                    startup_ap(x2apic, id, sleep_us);
                }
            }
        }
        ApicMode::Apic(ref mut xapic) => {
            for ap in aps {
                let id = ApicId::XApic(ap.local_apic_id.try_into().unwrap());
                unsafe {
                    startup_ap(xapic, id, sleep_us);
                }
            }
        }
    }

    Ok(())
}

fn install_ap_trampoline() {
    extern "C" {
        pub fn ap_trampoline();
        pub fn ap_trampoline_end();
    }

    let ap_trampoline_addr = ap_trampoline as usize;
    let ap_trampoline_end_addr = ap_trampoline_end as usize;
    let ap_trampoline_dest = 0x8000usize;

    let src = ap_trampoline_addr as *const u8;
    let dst = ap_trampoline_dest as *mut u8;
    let size = ap_trampoline_end_addr - ap_trampoline_addr;

    unsafe {
        core::ptr::copy(src, dst, size);
    }
}

unsafe fn startup_ap<T: ApicControl, SleepMicroSecondsFn: Fn(u64)>(
    local_apic: &mut T,
    apic_id: ApicId,
    sleep_us: SleepMicroSecondsFn,
) {
    // This code follows the guidelines on https://wiki.osdev.org/Symmetric_Multiprocessing
    unsafe {
        local_apic.ipi_init(apic_id);
        local_apic.ipi_init_deassert();
        sleep_us(10_000); // sleep for 10 milliseconds

        for _ in 0..2 {
            local_apic.ipi_startup(apic_id, 0x08);
            sleep_us(200); // sleep for 200 microseconds
        }
    }
}
