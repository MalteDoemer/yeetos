use core::sync::atomic::{AtomicUsize, Ordering};

use acpi::AcpiTables;
use memory::VirtAddr;

use super::acpi_handler::IdentityMapAcpiHandler;

#[no_mangle]
pub static AP_COUNT: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
pub static KERNEL_STACKS_VADDR: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
pub static KERNEL_STACK_SIZE: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    fn copy_ap_trampoline();
    fn startup_ap(lapic_base: u64, ap_id: u64);
}

// Initializes the kernel stack variables for the ap initialization code.
// This function needs to be called before calling `startup_aps()`
pub fn init_kernel_stack_vars(kernel_stacks_start: VirtAddr, kernel_stack_size: usize) {
    let addr: usize = kernel_stacks_start.into();
    KERNEL_STACKS_VADDR.store(addr, Ordering::SeqCst);
    KERNEL_STACK_SIZE.store(kernel_stack_size, Ordering::SeqCst);
}

pub fn startup_aps(acpi_tables: &AcpiTables<IdentityMapAcpiHandler>) {
    let platform_info = acpi_tables
        .platform_info()
        .expect("unable to get acpi platform info");

    let processor_info = platform_info
        .processor_info
        .expect("unable to get acpi processor info");

    let im = platform_info.interrupt_model;

    let apic = match im {
        acpi::InterruptModel::Apic(apic) => apic,
        _ => panic!("acpi interrupt model unkown"),
    };

    unsafe {
        copy_ap_trampoline();
    }

    for proc in processor_info.application_processors.iter() {
        unsafe {
            startup_ap(apic.local_apic_address, proc.local_apic_id.into());
        }
    }
}
