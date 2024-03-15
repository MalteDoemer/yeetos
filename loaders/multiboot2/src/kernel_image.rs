use elf::{abi::PT_LOAD, endian::NativeEndian, segment::SegmentTable, ElfBytes, ParseError};
use memory::{VirtAddr, PAGE_SIZE};

/// TODO: figure out (dynamically) how much stack should be allocated per core
const KERNEL_STACK_SIZE: usize = 64 * 1024;

/// This struct represents the to be loaded kernel image.
/// It contains the file data of the kernel image and
/// the address at which the kernel should be loaded.
pub struct KernelImage<'a> {
    load_addr: VirtAddr,
    num_cores: usize,
    data: &'a [u8],
    elf_image: ElfBytes<'a, NativeEndian>,
}

impl<'a> KernelImage<'a> {
    /// Create a `KernelImage` from the raw data and the load address.
    pub fn new(load_addr: VirtAddr, num_cores: usize, data: &'a [u8]) -> Result<Self, ParseError> {
        let elf_image = ElfBytes::minimal_parse(data)?;

        Ok(Self {
            load_addr,
            num_cores,
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

    pub fn kernel_stacks_start(&self) -> VirtAddr {
        self.load_addr
    }

    pub fn kernel_stack_size(&self) -> usize {
        KERNEL_STACK_SIZE
    }

    pub fn compute_in_memory_size(&self) -> usize {
        let image_size: usize = self
            .segments()
            .expect("could not find segment info in kernel executable")
            .iter()
            .filter(|seg| seg.p_type == PT_LOAD)
            .map(|seg| (seg.p_memsz as usize).next_multiple_of(PAGE_SIZE))
            .sum();

        image_size + self.compute_total_stack_size()
    }

    pub fn compute_total_stack_size(&self) -> usize {
        self.num_cores * KERNEL_STACK_SIZE
    }

    pub fn compute_load_end_address(&self) -> VirtAddr {
        self.load_addr + self.compute_in_memory_size()
    }
}
