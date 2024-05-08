use acpi::{AcpiHandler, PhysicalMapping};
use core::ptr::NonNull;
use memory::phys::{Frame, PhysAddr, PhysicalRange};
use memory::virt::{Page, VirtualRange};
use memory::IDENTITY_MAP_SIZE;

#[derive(Debug, Copy, Clone)]
pub enum IdentityMapMode {
    All,
    Range(PhysicalRange),
    HigherHalf,
}

#[derive(Debug, Copy, Clone)]
pub struct IdentityMappedAcpiHandler {
    mode: IdentityMapMode,
}

impl IdentityMappedAcpiHandler {
    pub fn new(mode: IdentityMapMode) -> Self {
        Self { mode }
    }

    pub fn all_physical_memory() -> Self {
        Self::new(IdentityMapMode::All)
    }

    pub fn lower_half() -> Self {
        let start_addr = PhysAddr::zero();
        let end_addr = PhysAddr::new(IDENTITY_MAP_SIZE.try_into().unwrap());
        let range = PhysicalRange::new_diff(Frame::new(start_addr), Frame::new(end_addr));
        Self::new(IdentityMapMode::Range(range))
    }

    pub fn higher_half() -> Self {
        Self::new(IdentityMapMode::HigherHalf)
    }

    fn translate(&self, range_to_map: PhysicalRange) -> Option<VirtualRange> {
        match self.mode {
            IdentityMapMode::All => self.identity(range_to_map),
            IdentityMapMode::Range(range) => {
                if range.contains_range(range_to_map) {
                    self.identity(range_to_map)
                } else {
                    None
                }
            }
            IdentityMapMode::HigherHalf => {
                let start_addr = range_to_map.start_addr().to_higher_half_checked()?;
                let end_addr = range_to_map.checked_end_addr()?.to_higher_half_checked()?;

                Some(VirtualRange::new_diff(
                    Page::new(start_addr),
                    Page::new(end_addr),
                ))
            }
        }
    }

    fn identity(&self, range: PhysicalRange) -> Option<VirtualRange> {
        let start_addr = range.start_addr().to_virt_checked()?;
        let end_addr = range.checked_end_addr()?.to_virt_checked()?;

        Some(VirtualRange::new_diff(
            Page::new(start_addr),
            Page::new(end_addr),
        ))
    }
}

impl AcpiHandler for IdentityMappedAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let phys_start_addr = PhysAddr::new(physical_address.try_into().unwrap());
        let phys_end_addr = PhysAddr::new((physical_address + size).try_into().unwrap());

        let range = PhysicalRange::new_diff(Frame::new(phys_start_addr), Frame::new(phys_end_addr));

        let translated = match self.translate(range) {
            None => panic!(
                "IdentityMappedAcpiHandler: unable to map physical region: {:?}",
                range
            ),
            Some(range) => range,
        };

        let physical_start = physical_address;
        let virtual_start = NonNull::new(translated.start_addr().as_ptr_mut::<T>())
            .expect("IdentityMappedAcpiHandler: tried to map address zero!");

        let region_length = size;
        let mapped_length = size;

        unsafe {
            PhysicalMapping::new(
                physical_start,
                virtual_start,
                region_length,
                mapped_length,
                self.clone(),
            )
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        // nothing to do here
    }
}
