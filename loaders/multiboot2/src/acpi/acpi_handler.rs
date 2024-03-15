use core::ptr::NonNull;

use acpi::{AcpiHandler, PhysicalMapping};

const HIGHEST_MAPPED_ADDRESS: usize = 4 * 1024 * 1024 * 1024;

/// An implementation of the `AcpiHandler` trait needed for the acpi crate.
///
/// The job of the AcpiHandler is to map physical memory regions that are needed by the acpi crate.
/// Since this bootloader identity maps the first 4GiB of memory we don't need to do anything for addresses below 4GiB.
#[derive(Clone, Copy)]
pub struct IdentityMapAcpiHandler;

impl AcpiHandler for IdentityMapAcpiHandler {
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
                    IdentityMapAcpiHandler,
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
