use core::ptr::NonNull;

use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};

use crate::multiboot2::{Multiboot2Info, RSDPDescriptor, RSDPDescriptorV1, RSDPDescriptorV2};

const HIGHEST_MAPPED_ADDRESS: usize = 4 * 1024 * 1024 * 1024;

/// An implementation of the `AcpiHandler` trait needed for the acpi crate.
///
/// The job of the AcpiHandler is to map physical memory regions that are needed by the acpi crate.
/// Since this bootloader identity maps the first 4GiB of memory we don't need to do anything for addresses below 4GiB.
#[derive(Clone, Copy)]
pub struct YeetOSAcpiHandler;

impl AcpiHandler for YeetOSAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        if physical_address <= HIGHEST_MAPPED_ADDRESS {
            let virtual_address = physical_address;

            let physical_start = physical_address;
            let virtual_start = NonNull::new(virtual_address as *mut T)
                .expect("YeetOSAcpiHandler: given address to map was zero");
            let region_length = size;
            let mapped_length = size;

            // Safety:
            // All memory below 4 GiB is identity mapped.
            unsafe {
                PhysicalMapping::new(
                    physical_start,
                    virtual_start,
                    region_length,
                    mapped_length,
                    YeetOSAcpiHandler,
                )
            }
        } else {
            todo!("YeetOSAcpiHandler needed to map an address greater than 4 GiB")
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        // nothing to do here
    }
}

extern "C" {

    fn copy_ap_trampoline();
    fn startup_ap(lapic_base: u64, ap_id: u64);

}

pub fn acpi_init(mboot_info: &Multiboot2Info) {
    let acpi_tables = get_acpi_tables(
        &mboot_info
            .rsdp_descriptor
            .expect("no RSDP descriptor provided by multiboot2"),
    );

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

pub fn get_acpi_tables(rsdp: &RSDPDescriptor) -> AcpiTables<YeetOSAcpiHandler> {
    let addr = match rsdp {
        RSDPDescriptor::V1(ref rsdp) => rsdp as *const RSDPDescriptorV1 as usize,
        RSDPDescriptor::V2(ref rsdp) => rsdp as *const RSDPDescriptorV2 as usize,
    };

    // Safety:
    // All memory from the rsdp should be correctly mapped and safe to dereference.
    let tables = unsafe { AcpiTables::from_rsdp(YeetOSAcpiHandler, addr) }
        .expect("parsing acpi tables failed");

    tables
}
