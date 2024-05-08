use crate::arch::paging;
use crate::devices::tsc::busy_sleep_us;
use crate::entry::{make_jump_to_kernel, KERNEL_ENTRY};
use crate::idt;
use crate::multiboot2::{RSDPDescriptor, RSDPDescriptorV1, RSDPDescriptorV2};
use acpi::AcpiTables;
use kernel_image::KernelImage;
use log::info;
use memory::phys::PhysAddr;
use multi_core::handler::IdentityMappedAcpiHandler;

pub fn startup_all_application_processors(
    acpi_tables: &AcpiTables<IdentityMappedAcpiHandler>,
    kernel_image: &KernelImage,
) {
    // Defined in boot.s
    let page_table_addr = PhysAddr::new(0x1000);

    multi_core::ap_startup::startup_all_application_processors(
        acpi_tables,
        kernel_image,
        page_table_addr,
        busy_sleep_us,
    )
    .unwrap();
}

pub fn get_acpi_tables(rsdp: &RSDPDescriptor) -> AcpiTables<IdentityMappedAcpiHandler> {
    let addr = match rsdp {
        RSDPDescriptor::V1(ref rsdp) => rsdp as *const RSDPDescriptorV1 as usize,
        RSDPDescriptor::V2(ref rsdp) => rsdp as *const RSDPDescriptorV2 as usize,
    };

    let handler = get_handler();

    // Safety:
    // All memory from the rsdp should be correctly mapped and safe to dereference.
    let tables =
        unsafe { AcpiTables::from_rsdp(handler, addr) }.expect("parsing acpi tables failed");

    tables
}

fn get_handler() -> IdentityMappedAcpiHandler {
    #[cfg(target_arch = "x86_64")]
    {
        IdentityMappedAcpiHandler::lower_half()
    }

    #[cfg(target_arch = "x86")]
    {
        IdentityMappedAcpiHandler::all_physical_memory()
    }
}

#[no_mangle]
extern "C" fn rust_entry_ap(ap_id: usize) -> ! {
    info!("application processor #{} started", ap_id);

    // initialize paging for this AP
    paging::init_ap();

    // load IDT for this core
    idt::init_ap();

    // this waits until the BSP is finished initializing
    let entry_point = KERNEL_ENTRY.wait();

    make_jump_to_kernel(ap_id, *entry_point);
}
