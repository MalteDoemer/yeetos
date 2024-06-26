mod acpi_handler;
mod ap_startup;

use acpi::{platform::ProcessorState, AcpiTables};

use crate::multiboot2::{RSDPDescriptor, RSDPDescriptorV1, RSDPDescriptorV2};

use acpi_handler::IdentityMapAcpiHandler;

pub use ap_startup::*;

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

pub fn get_acpi_tables(rsdp: &RSDPDescriptor) -> AcpiTables<IdentityMapAcpiHandler> {
    let addr = match rsdp {
        RSDPDescriptor::V1(ref rsdp) => rsdp as *const RSDPDescriptorV1 as usize,
        RSDPDescriptor::V2(ref rsdp) => rsdp as *const RSDPDescriptorV2 as usize,
    };

    // Safety:
    // All memory from the rsdp should be correctly mapped and safe to dereference.
    let tables = unsafe { AcpiTables::from_rsdp(IdentityMapAcpiHandler, addr) }
        .expect("parsing acpi tables failed");

    tables
}
