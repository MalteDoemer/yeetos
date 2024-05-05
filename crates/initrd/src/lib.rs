#![no_std]

use memory::virt::VirtAddr;
use tar_no_std::{ArchiveEntry, CorruptDataError, TarArchiveRef};

/// A wrapper object around the INITRD tar archive. The INITRD is always read-only.
#[derive(Debug, Clone)]
pub struct Initrd<'a> {
    data: &'a [u8],
    tar_archive: TarArchiveRef<'a>,
}

impl<'a> Initrd<'a> {
    /// Creates a new `Initrd` instance by parsing the provided data as tar archive.
    ///
    /// # Safety
    /// The memory at address `addr` with a size of `size` must be valid for reads for the provided
    /// lifetime `'a`
    pub unsafe fn from_addr_size(addr: VirtAddr, size: usize) -> Result<Self, CorruptDataError> {
        let ptr = addr.as_ptr::<u8>();

        // Safety:
        // Function contract ensures valid memory
        let slice = unsafe { core::slice::from_raw_parts(ptr, size) };

        Self::new(slice)
    }

    /// Creates a new `Initrd` instance by parsing the provided data as tar archive.
    pub fn new(data: &'a [u8]) -> Result<Self, CorruptDataError> {
        let tar_archive = TarArchiveRef::new(data)?;
        Ok(Self { data, tar_archive })
    }

    pub fn data(&self) -> &[u8] {
        self.data
    }

    pub fn tar_archive(&self) -> TarArchiveRef<'a> {
        self.tar_archive.clone()
    }

    pub fn file_by_name(&self, name: &str) -> Option<ArchiveEntry> {
        self.tar_archive
            .entries()
            .find(|entry| match entry.filename().as_str() {
                Ok(str) => str == name,
                Err(_) => false,
            })
    }

    pub fn start_addr(&self) -> VirtAddr {
        VirtAddr::new(self.data.as_ptr() as usize)
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn end_addr(&self) -> VirtAddr {
        self.start_addr() + self.size()
    }
}
