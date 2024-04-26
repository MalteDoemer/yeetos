use core::sync::atomic::{AtomicUsize, Ordering};

use acpi::{platform::ProcessorState, AcpiTables};
use kernel_image::KernelImage;
use log::info;
use memory::{
    virt::{Page, VirtAddr, VirtualRange},
    PAGE_SHIFT, PAGE_SIZE,
};
use spin::Once;
use uefi::table::boot::{AllocateType, BootServices, MemoryType};

use super::acpi_handler::IdentityMapAcpiHandler;

pub static AP_COUNT: AtomicUsize = AtomicUsize::new(0);

pub static KERNEL_ENTRY: Once<VirtAddr> = Once::new();

#[no_mangle]
static KERNEL_STACKS_VADDR: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
static KERNEL_STACK_SIZE: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    fn startup_ap(local_apic_addr: usize, apic_id: usize, vector: usize);
}

pub fn startup_aps(
    boot_services: &BootServices,
    acpi_tables: &AcpiTables<IdentityMapAcpiHandler>,
    kernel_image: &KernelImage,
) {
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

    let local_apic_addr: usize = apic
        .local_apic_address
        .try_into()
        .expect("local apic address to large");

    let ap_tramp = init_ap_trampoline(boot_services);

    let vector = ap_tramp.to_inner() >> PAGE_SHIFT;

    processor_info
        .application_processors
        .iter()
        .filter(|ap| ap.state == ProcessorState::WaitingForSipi)
        .map(|ap| ap.local_apic_id.try_into().unwrap())
        .for_each(|apic_id: usize| unsafe {
            // We only set up `num_cores` of kernel stacks and we use
            // the apic_id in boot.s to load a stack pointer.
            // Thus if the apic_id is bigger than num_cores (for whatever reason)
            // we will get memory corruption.
            assert!(apic_id < num_cores);

            // Safety: this function is implemented in boot.s and assumed to be safe.
            // startup_ap(addr, apic_id);
            info!("starting ap #{} now ...", apic_id);
            startup_ap(local_apic_addr, apic_id, vector);
        });
}

fn init_ap_trampoline(boot_services: &BootServices) -> VirtAddr {
    let ap_dest = allocate_ap_trampoline(boot_services);

    let ap_source = ap_trampoline_range();

    // now copy the memory from `ap_source` to `ap_dest`
    unsafe {
        core::ptr::copy(
            ap_source.start_addr().as_ptr::<u8>(),
            ap_dest.as_ptr_mut::<u8>(),
            ap_source.num_pages() * PAGE_SIZE,
        );
    }

    ap_dest
}

fn ap_trampoline_range() -> VirtualRange {
    extern "C" {
        fn ap_trampoline();
        fn ap_trampoline_end();
    }

    let ap_trampoline_start_addr = ap_trampoline as usize;
    let ap_trampoline_end_addr = ap_trampoline_end as usize;

    let start = Page::new(ap_trampoline_start_addr.into());
    let end = Page::new(ap_trampoline_end_addr.into());

    VirtualRange::new_diff(start, end)
}

fn allocate_ap_trampoline(boot_services: &BootServices) -> VirtAddr {
    // Since the cpu is in real mode when executing the ap trampoline
    // and starts at address XX00:0000 where XX is the vector specified in the STARTUP IPI
    // we have to find free memory below 0xFF000 which is the highest addressable address
    let max_addr = 0xFF000;

    let num_pages = ap_trampoline_range().num_pages();

    let pages = boot_services.allocate_pages(
        AllocateType::MaxAddress(max_addr),
        MemoryType::LOADER_DATA,
        num_pages,
    );

    VirtAddr::new(pages.expect("unable to allocate pages for ap trampoline") as usize)
}
