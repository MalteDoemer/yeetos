use acpi::{AcpiHandler, PhysicalMapping};
use core::ptr::NonNull;
use memory::phys::{Frame, PhysAddr, PhysicalRange};
use memory::virt::VirtAddr;
use memory::{FRAME_SIZE, IDENTITY_MAP_SIZE};

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
        let range = PhysicalRange::new(Frame::new(start_addr), Frame::new(end_addr));
        Self::new(IdentityMapMode::Range(range))
    }

    pub fn higher_half() -> Self {
        Self::new(IdentityMapMode::HigherHalf)
    }

    fn translate(&self, addr: PhysAddr, size: memory::phys::Inner) -> Option<VirtAddr> {
        match self.mode {
            IdentityMapMode::All => addr.to_virt_checked(),
            IdentityMapMode::Range(range) => {
                let range_to_map = PhysicalRange::with_size(
                    Frame::new(addr),
                    size.checked_next_multiple_of(FRAME_SIZE)? / FRAME_SIZE,
                );

                if range.contains_range(range_to_map) {
                    addr.to_virt_checked()
                } else {
                    None
                }
            }
            IdentityMapMode::HigherHalf => {
                let start_addr = addr.to_higher_half_checked()?;
                let _ =
                    PhysAddr::new(addr.to_inner().checked_add(size)?).to_higher_half_checked()?;
                Some(start_addr)
            }
        }
    }
}

impl AcpiHandler for IdentityMappedAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let phys_start_addr = PhysAddr::new(physical_address.try_into().unwrap());
        //let phys_end_addr = PhysAddr::new((physical_address + size).try_into().unwrap());

        // let range = PhysicalRange::new_diff(Frame::new(phys_start_addr), Frame::new(phys_end_addr));

        let translated = match self.translate(phys_start_addr, size.try_into().unwrap()) {
            None => panic!(
                "IdentityMappedAcpiHandler: unable to map physical region: start={:p}, size{:#x}",
                phys_start_addr, size,
            ),
            Some(addr) => addr,
        };

        let physical_start = physical_address;
        let virtual_start = NonNull::new(translated.as_ptr_mut::<T>())
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
