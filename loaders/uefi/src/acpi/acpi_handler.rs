use core::ptr::NonNull;

use acpi::{AcpiHandler, PhysicalMapping};

/// An implementation of the `AcpiHandler` trait needed for the acpi crate.
///
/// The job of the AcpiHandler is to map physical memory regions that are needed by the acpi crate.
/// In UEFI, as long as BootServices are still enabled all physical memory is identity mapped.
#[derive(Clone, Copy)]
pub struct IdentityMapAcpiHandler;

impl AcpiHandler for IdentityMapAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        let virtual_address = physical_address;

        let physical_start = physical_address;
        let virtual_start = NonNull::new(virtual_address as *mut T)
            .expect("YeetOSAcpiHandler: given address to map was zero");
        let region_length = size;
        let mapped_length = size;

        // Safety:
        // All memory is identity mapped.
        unsafe {
            PhysicalMapping::new(
                physical_start,
                virtual_start,
                region_length,
                mapped_length,
                IdentityMapAcpiHandler,
            )
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        // nothing to do here
    }
}
