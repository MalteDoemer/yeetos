use crate::arch::time::busy_sleep_us;
use crate::entry::{make_jump_to_kernel, KERNEL_ENTRY};
use acpi::AcpiTables;
use kernel_image::KernelImage;
use log::info;
use memory::phys::PhysAddr;
use memory::virt::VirtAddr;
use multi_core::handler::{IdentityMapMode, IdentityMappedAcpiHandler};
use spin::Once;
use uefi::table::cfg::{ACPI2_GUID, ACPI_GUID};
use uefi::table::{Boot, SystemTable};
use x86::controlregs::cr3;

pub fn startup_all_application_processors(
    acpi_tables: &AcpiTables<IdentityMappedAcpiHandler>,
    kernel_image: &KernelImage,
) {
    // Safety: we have CPL=0
    let cr3 = unsafe { cr3() };
    let pml4t_addr = PhysAddr::new(cr3);

    multi_core::ap_startup::startup_all_application_processors(
        acpi_tables,
        kernel_image,
        pml4t_addr,
        busy_sleep_us,
    )
    .expect("failed to bring up the application processors")
}

pub fn get_acpi_tables(system_table: &SystemTable<Boot>) -> AcpiTables<IdentityMappedAcpiHandler> {
    let acpi_entry = system_table
        .config_table()
        .iter()
        .find(|entry| entry.guid == ACPI_GUID || entry.guid == ACPI2_GUID)
        .expect("unable to find acpi tables");

    let rsdp_addr = VirtAddr::new(acpi_entry.address as usize);

    // Safety:
    // UEFI must provide correct pointers to the RSDP
    unsafe { get_acpi_tables_impl(rsdp_addr) }
}

/// Read the AcpiTables from system memory.
/// # Safety
/// `rsdp_addr` must point to either a valid RSDPV1 or RSDPV2 struct.
unsafe fn get_acpi_tables_impl(rsdp_addr: VirtAddr) -> AcpiTables<IdentityMappedAcpiHandler> {
    // UEFI identity maps all physical memory
    let handler = IdentityMappedAcpiHandler::new(IdentityMapMode::All);
    let tables = unsafe { AcpiTables::from_rsdp(handler, rsdp_addr.to_inner()) };
    tables.expect("parsing acpi tables failed")
}

#[no_mangle]
extern "C" fn rust_entry_ap(ap_id: usize) -> ! {
    info!("application processor #{} started", ap_id);

    // Load the new higher-half enabled page table
    crate::paging::activate();

    // this waits until the BSP is finished initializing
    let entry_point = KERNEL_ENTRY.wait();
    
    make_jump_to_kernel(ap_id, *entry_point);
}
