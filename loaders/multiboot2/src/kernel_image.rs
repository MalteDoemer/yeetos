use elf::{endian::NativeEndian, segment::SegmentTable, ElfBytes, ParseError};
use memory::VirtAddr;

/// This struct represents the to be loaded kernel image.
/// It contains the file data of the kernel image and
/// the address at which the kernel should be loaded.
pub struct KernelImage<'a> {
    load_addr: VirtAddr,
    data: &'a [u8],
    elf_image: ElfBytes<'a, NativeEndian>,
}

impl<'a> KernelImage<'a> {
    /// Create a `KernelImage` from the raw data and the load address.
    pub fn new(load_addr: VirtAddr, data: &'a [u8]) -> Result<Self, ParseError> {
        let elf_image = ElfBytes::minimal_parse(data)?;

        Ok(Self {
            load_addr,
            data,
            elf_image,
        })
    }

    pub fn elf_image(&self) -> &ElfBytes<'a, NativeEndian> {
        &self.elf_image
    }

    pub fn segments(&self) -> Option<SegmentTable<'a, NativeEndian>> {
        self.elf_image.segments()
    }
}
