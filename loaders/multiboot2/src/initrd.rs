use memory::virt::VirtAddr;
use tar_no_std::{ArchiveEntry, TarArchiveRef};

use crate::multiboot2::{Multiboot2Info, MultibootModule};

pub struct Initrd<'a> {
    data: &'a [u8],
    tar_archive: TarArchiveRef<'a>,
}

impl<'a> Initrd<'a> {
    /// Creates a `Initrd` from a multiboot2 module.
    ///
    /// ### Safety
    /// - `module.mod_start` until `module.mod_end` must be accessible memory.
    /// - Rust's aliasing rules must be respected.
    pub unsafe fn from_module(module: &MultibootModule) -> Initrd {
        // Note:
        // Physical address == Virtual address

        let start_addr: usize = module.mod_start.try_into().unwrap();
        let end_addr: usize = module.mod_end.try_into().unwrap();
        let size = end_addr - start_addr;

        let slice = unsafe { core::slice::from_raw_parts(start_addr as *const u8, size) };

        Initrd {
            data: slice,
            tar_archive: TarArchiveRef::new(slice),
        }
    }

    /// Creates a `Initrd` by searching for the correct module from the multiboot2 struct.
    pub fn from_multiboot_info(mboot_info: &Multiboot2Info) -> Option<Initrd> {
        let module = mboot_info.module_by_name("initrd")?;

        // Safety: This is the only function accessing the "initrd" module.
        // Thus there will be no other references to the memory other than those obtained by this function.
        unsafe { Some(Self::from_module(module)) }
    }

    pub fn tar_archive(&self) -> &TarArchiveRef {
        &self.tar_archive
    }

    pub fn file_by_name(&self, name: &str) -> Option<ArchiveEntry> {
        self.tar_archive()
            .entries()
            .find(|entry| entry.filename().as_str() == name)
    }

    pub fn start_addr(&self) -> VirtAddr {
        let addr = self.data.as_ptr() as usize;
        VirtAddr::new(addr)
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn end_addr(&self) -> VirtAddr {
        self.start_addr() + self.size()
    }
}
