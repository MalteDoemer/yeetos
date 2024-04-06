use core::sync::atomic::{AtomicUsize, Ordering};

use acpi::AcpiTables;
use log::info;
use memory::{to_higher_half, VirtAddr};
use spin::Once;

use crate::{acpi::IdentityMapAcpiHandler, arch::paging, boot_info, idt};

pub static AP_COUNT: AtomicUsize = AtomicUsize::new(0);

pub static KERNEL_ENTRY: Once<VirtAddr> = Once::new();

#[no_mangle]
static KERNEL_STACKS_VADDR: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
static KERNEL_STACK_SIZE: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    fn copy_ap_trampoline();

    fn startup_ap(lapic_base: usize, ap_id: usize);

    fn jmp_kernel_entry(
        boot_info_ptr: usize,
        processor_id: usize,
        entry_point: usize,
        stack_ptr: usize,
    ) -> !;
}

/// Initializes the kernel stack variables for the ap initialization code.
/// This function needs to be called before calling `startup_aps()`
///
/// # Panics
/// If kernel_stacks_start or kernel_stack_size is zero.
pub fn init_kernel_stack_vars(kernel_stacks_start: VirtAddr, kernel_stack_size: usize) {
    let addr: usize = kernel_stacks_start.into();

    if addr == 0 {
        panic!("init_kernel_stack_vars() called with kernel_stacks_start=0");
    }

    if kernel_stack_size == 0 {
        panic!("init_kernel_stack_vars() called with kernel_stack_size=0");
    }

    KERNEL_STACKS_VADDR.store(addr, Ordering::SeqCst);
    KERNEL_STACK_SIZE.store(kernel_stack_size, Ordering::SeqCst);
}

pub fn startup_aps(acpi_tables: &AcpiTables<IdentityMapAcpiHandler>) {
    if KERNEL_STACKS_VADDR.load(Ordering::SeqCst) == 0
        || KERNEL_STACK_SIZE.load(Ordering::SeqCst) == 0
    {
        panic!("startup_aps() called before init_kernel_stack_vars()");
    }

    let platform_info = acpi_tables
        .platform_info()
        .expect("unable to get acpi platform info");

    let processor_info = platform_info
        .processor_info
        .expect("unable to get acpi processor info");

    let im = platform_info.interrupt_model;

    let apic = match im {
        acpi::InterruptModel::Apic(apic) => apic,
        _ => panic!("acpi interrupt model unknown"),
    };

    // Safety: this function is implemented in boot.s and assumed to be safe.
    unsafe {
        copy_ap_trampoline();
    }

    let addr = apic
        .local_apic_address
        .try_into()
        .expect("local apic address to large");

    let num_cores = processor_info.application_processors.len() + 1;

    processor_info
        .application_processors
        .iter()
        .filter(|ap| matches!(ap.state, acpi::platform::ProcessorState::WaitingForSipi))
        .map(|ap| ap.local_apic_id.try_into().unwrap())
        .for_each(|apic_id| unsafe {
            // We only set up `num_cores` of kernel stacks and we use
            // the apic_id in boot.s to load a stack pointer.
            // Thus if the apic_id is bigger than num_cores (for whatever reason)
            // we will get memory corruption.
            assert!(apic_id < num_cores);

            // Safety: this function is implemented in boot.s and assumed to be safe.
            startup_ap(addr, apic_id);
        });
}

#[no_mangle]
pub extern "C" fn rust_entry_ap(ap_id: usize) -> ! {
    AP_COUNT.fetch_add(1, Ordering::SeqCst);

    info!("application processor #{} started", ap_id);

    // initialize paging for this AP
    paging::init_ap();

    // load IDT for this core
    idt::init_ap();

    // this waits until the BSP is finished initializing
    let entry_point = KERNEL_ENTRY.wait();

    make_jump_to_kernel(ap_id, *entry_point);
}

pub fn make_jump_to_kernel(processor_id: usize, entry_point_addr: VirtAddr) -> ! {
    let boot_info = boot_info::get_boot_info_addr();

    // calculate stack
    let stacks_base = to_higher_half(KERNEL_STACKS_VADDR.load(Ordering::SeqCst).into());
    let stack_size = KERNEL_STACK_SIZE.load(Ordering::SeqCst);
    let stack_addr = stacks_base + processor_id * stack_size;

    unsafe {
        jmp_kernel_entry(
            boot_info.to_inner(),
            processor_id,
            entry_point_addr.to_inner(),
            stack_addr.to_inner(),
        )
    };
}
