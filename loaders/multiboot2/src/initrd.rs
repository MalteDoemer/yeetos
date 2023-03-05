use memory::VirtAddr;
use tar_no_std::{ArchiveEntry, TarArchiveRef};

use crate::multiboot2::{ModuleDescriptor, Multiboot2Info};

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
    pub unsafe fn from_module(module: &ModuleDescriptor) -> Initrd {
        // Note:
        // Physical address == Virtual address

        let start_addr = module.mod_start as usize;
        let end_addr = module.mod_end as usize;
        let size = end_addr - start_addr;

        let slice = unsafe { core::slice::from_raw_parts(start_addr as *const u8, size) };

        Initrd {
            data: slice,
            tar_archive: TarArchiveRef::new(slice),
        }
    }

    /// Creates a `Initrd` from the multiboot2 info struct.
    ///
    /// ### Panics
    /// Panics if no initrd module has been provided by the bootloader.
    pub fn from_multiboot_info(mboot_info: &Multiboot2Info) -> Initrd {
        let initrd_mod = mboot_info
            .modules
            .iter()
            .find(|module| module.info == "initrd")
            .expect("initrd module not found");

        // Safety:
        // initrd_mod is assumed to be mapped and not used by anything else.
        unsafe { Initrd::from_module(initrd_mod) }
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
