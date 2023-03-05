use elf::{endian::NativeEndian, segment::SegmentTable, ElfBytes};

use crate::initrd::Initrd;

pub struct KernelImage<'a> {
    data: &'a [u8],
    elf_image: ElfBytes<'a, NativeEndian>,
}

impl<'a> KernelImage<'a> {
    /// Create a `KernelImage` from the raw data.
    ///
    /// ### Panics
    /// Panics if the kernel image contains invalid data.
    pub fn new(data: &'a [u8]) -> Self {
        let elf_image = ElfBytes::minimal_parse(data).expect("unable to parse kernel image");
        Self { data, elf_image }
    }

    /// Create a `KernelImage` from the initrd.
    ///
    /// ### Panics
    /// - Panics if the initrd doesn't contain the kernel image
    /// - Panics if the kernel image contains invalid data.
    pub fn from_initrd(initrd: &'a Initrd) -> Self {
        let kernel_file = initrd
            .file_by_name("kernel")
            .expect("unable to find kernel file");

        Self::new(kernel_file.data())
    }

    pub fn segments(&self) -> Option<SegmentTable<'a, NativeEndian>> {
        self.elf_image.segments()
    }

    pub fn memory_size(&self) -> Option<usize> {
        let mut res = 0;

        for segment in self.segments()?.iter() {
            if segment.p_type == elf::abi::PT_LOAD {
                res += segment.p_memsz;
            }
        }

        Some(res as usize)
    }
}
