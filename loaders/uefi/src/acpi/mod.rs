use acpi::{platform::ProcessorState, AcpiTables};

use acpi_handler::IdentityMapAcpiHandler;
use memory::virt::VirtAddr;
use uefi::table::{
    cfg::{ACPI2_GUID, ACPI_GUID},
    Boot, SystemTable,
};

pub mod acpi_handler;
pub mod ap_startup;

pub fn number_of_cores(acpi_tables: &AcpiTables<IdentityMapAcpiHandler>) -> usize {
    acpi_tables
        .platform_info()
        .expect("unable to get acpi platform info")
        .processor_info
        .expect("unable to get acpi processor info")
        .application_processors
        .iter()
        .filter(|ap| ap.state == ProcessorState::WaitingForSipi)
        .count()
        + 1 // + 1 for the BSP
}

pub fn get_acpi_tables(system_table: &SystemTable<Boot>) -> AcpiTables<IdentityMapAcpiHandler> {
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
unsafe fn get_acpi_tables_impl(rsdp_addr: VirtAddr) -> AcpiTables<IdentityMapAcpiHandler> {
    let tables = unsafe { AcpiTables::from_rsdp(IdentityMapAcpiHandler, rsdp_addr.to_inner()) };
    tables.expect("parsing acpi tables failed")
}
