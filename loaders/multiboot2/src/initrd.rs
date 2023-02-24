use memory::VirtAddr;
use tar_no_std::TarArchiveRef;

use crate::multiboot2::ModuleDescriptor;

pub struct Initrd<'a> {
    data: &'a [u8],
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

        Initrd { data: slice }
    }

    pub fn tar_archive(&self) -> TarArchiveRef {
        TarArchiveRef::new(self.data)
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
