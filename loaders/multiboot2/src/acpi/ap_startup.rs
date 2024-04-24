use core::sync::atomic::{AtomicUsize, Ordering};

use acpi::{platform::ProcessorState, AcpiTables};
use kernel_image::KernelImage;
use log::info;
use memory::{to_higher_half, virt::VirtAddr};
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

pub fn startup_aps(acpi_tables: &AcpiTables<IdentityMapAcpiHandler>, kernel_image: &KernelImage) {
    let stacks_addr = kernel_image.kernel_image_info().stack.start_addr();
    KERNEL_STACKS_VADDR.store(stacks_addr.to_inner(), Ordering::SeqCst);
    KERNEL_STACK_SIZE.store(kernel_image.kernel_stack_size(), Ordering::SeqCst);

    let num_cores = kernel_image.num_cores();

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

    processor_info
        .application_processors
        .iter()
        .filter(|ap| ap.state == ProcessorState::WaitingForSipi)
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
