use alloc::vec::Vec;
use boot_info::kernel_image_info::KernelImageInfo;
use elf::{abi::PT_LOAD, endian::LittleEndian, segment::ProgramHeader, ElfBytes, ParseError};
use memory::{to_lower_half, Page, VirtAddr, VirtualRange, KERNEL_BASE};

const PHDR_EXEC: u32 = 1;
const PHDR_WRITE: u32 = 2;
const PHDR_READ: u32 = 4;

const PHDR_RODATA: u32 = PHDR_READ;
const PHDR_CODE: u32 = PHDR_EXEC | PHDR_READ;
const PHDR_DATA: u32 = PHDR_READ | PHDR_WRITE;

#[derive(Debug)]
pub enum KernelImageError {
    ParseError(ParseError),
    ProgramHeadersMissing,
    ProgramHeadersInvalid,
    CodeSegmentMissing,
}

impl From<ParseError> for KernelImageError {
    fn from(value: ParseError) -> Self {
        KernelImageError::ParseError(value)
    }
}

pub struct KernelImageProgramHeaders {
    rodata: Option<ProgramHeader>,
    code: ProgramHeader,
    relro: Option<ProgramHeader>,
    data: Option<ProgramHeader>,
}

impl KernelImageProgramHeaders {
    pub fn image_base(&self) -> VirtAddr {
        // Note: casts from u64 to usize are still correct on 32-bit targets
        if let Some(rodata) = self.rodata {
            VirtAddr::new(rodata.p_vaddr as usize)
        } else {
            VirtAddr::new(self.code.p_vaddr as usize)
        }
    }
}

pub struct KernelImage<'a> {
    info: KernelImageInfo,
    phdrs: KernelImageProgramHeaders,
    kernel_stack_size: usize,
    entry_point: VirtAddr,
    elf_image: ElfBytes<'a, LittleEndian>,
}

impl<'a> KernelImage<'a> {
    /// Create a `KernelImage` struct using either `new_reloc()` or `new_fixed()` based on `use_reloc`.
    pub fn new(
        load_start_addr: VirtAddr,
        num_cores: usize,
        kernel_stack_size: usize,
        use_reloc: bool,
        data: &'a [u8],
    ) -> Result<Self, KernelImageError> {
        if use_reloc {
            Self::new_reloc(load_start_addr, num_cores, kernel_stack_size, data)
        } else {
            Self::new_fixed(num_cores, kernel_stack_size, data)
        }
    }

    /// Create a `KernelImage` struct that is going to be loaded at `load_start_addr`.
    /// Using this function allows for ASLR for the kernel if used with a random offset.
    pub fn new_reloc(
        load_start_addr: VirtAddr,
        num_cores: usize,
        kernel_stack_size: usize,
        data: &'a [u8],
    ) -> Result<Self, KernelImageError> {
        let elf_image = ElfBytes::minimal_parse(data)?;
        let phdrs = Self::get_phdrs(&elf_image)?;
        let info = Self::get_image_info(&phdrs, load_start_addr, num_cores, kernel_stack_size);
        let entry_point = Self::get_entry_point(&elf_image, &phdrs, &info);

        Ok(Self {
            info,
            phdrs,
            kernel_stack_size,
            entry_point,
            elf_image,
        })
    }

    /// Create a `KernelImage` struct that is going to be loaded into memory according to
    /// the image base it was compiled with. The image base is assumed to be a higher-half address.
    /// # Important
    /// The stacks for the cpu cores will be located before the first PT_LOAD segment.
    /// This means one has to make sure there is enough space inbetween the multiboot2 loader
    /// and the kernel.
    pub fn new_fixed(
        num_cores: usize,
        kernel_stack_size: usize,
        data: &'a [u8],
    ) -> Result<Self, KernelImageError> {
        let elf_image: ElfBytes<'a, LittleEndian> = ElfBytes::minimal_parse(data)?;
        let phdrs = Self::get_phdrs(&elf_image)?;
        let load_start_addr =
            Self::get_load_start_addr_for_fixed_image(&phdrs, num_cores, kernel_stack_size);
        let info = Self::get_image_info(&phdrs, load_start_addr, num_cores, kernel_stack_size);
        let entry_point = Self::get_entry_point(&elf_image, &phdrs, &info);

        Ok(Self {
            info,
            phdrs,
            kernel_stack_size,
            entry_point,
            elf_image,
        })
    }

    fn get_entry_point(
        elf_image: &ElfBytes<'a, LittleEndian>,
        phdrs: &KernelImageProgramHeaders,
        info: &KernelImageInfo,
    ) -> VirtAddr {
        let file_addr = VirtAddr::new(elf_image.ehdr.e_entry as usize);

        let image_base_file = phdrs.image_base();
        let image_base_mem = info.image_base();

        image_base_mem + (file_addr - image_base_file)
    }

    fn get_image_info(
        phdrs: &KernelImageProgramHeaders,
        load_start_addr: VirtAddr,
        num_cores: usize,
        kernel_stack_size: usize,
    ) -> KernelImageInfo {
        let image_base_file = phdrs.image_base();

        let stack = Self::get_stack_segment(load_start_addr, num_cores, kernel_stack_size);

        let image_base_mem = stack.end().to_addr();

        let rodata =
            Self::get_optional_segment_from_phdr(phdrs.rodata, image_base_file, image_base_mem);

        let code = Self::get_segment_from_phdr(phdrs.code, image_base_file, image_base_mem);

        let relro =
            Self::get_optional_segment_from_phdr(phdrs.relro, image_base_file, image_base_mem);

        let data =
            Self::get_optional_segment_from_phdr(phdrs.data, image_base_file, image_base_mem);

        KernelImageInfo {
            stack,
            rodata,
            code,
            relro,
            data,
        }
    }

    /// This function computes the range of virtual memory occupied by the stacks.
    fn get_stack_segment(
        load_start_addr: VirtAddr,
        num_cores: usize,
        kernel_stack_size: usize,
    ) -> VirtualRange {
        let stack_start = load_start_addr;
        let stack_size = num_cores * kernel_stack_size;
        let stack_end = stack_start + stack_size;

        let start_page = Page::new(stack_start);
        let end_page = Page::new(stack_end.page_align_up_checked().unwrap());

        VirtualRange::new_diff(start_page, end_page)
    }

    /// This function computes the range of virtual memory the given PHDR is going to occupy.
    ///
    /// - `image_base_file` is the image base this executable was compiled with (i.e. --image-base=... argument to the linker)
    /// - `image_base_mem` is the calculated address of the first non-stack segment in memory
    ///
    /// # Note
    /// When creating the image with `new_fixed()` then `image_base_file = image_base_mem`.
    /// Otherwise they might differ since the kernel is compiled with position independant code enabled.
    fn get_segment_from_phdr(
        phdr: ProgramHeader,
        image_base_file: VirtAddr,
        image_base_mem: VirtAddr,
    ) -> VirtualRange {
        // Note: casts from u64 to usize are still correct on 32-bit targets
        let addr_in_file = VirtAddr::new(phdr.p_vaddr as usize);
        let offset = addr_in_file - image_base_file;

        let segment_start = image_base_mem + offset;
        let segment_end = segment_start + phdr.p_memsz as usize;

        let start_page = Page::new(segment_start);
        let end_page = Page::new(segment_end.page_align_up_checked().unwrap());

        VirtualRange::new_diff(start_page, end_page)
    }

    /// See `get_segment_from_phdr()`
    fn get_optional_segment_from_phdr(
        phdr: Option<ProgramHeader>,
        image_base_file: VirtAddr,
        image_base_mem: VirtAddr,
    ) -> Option<VirtualRange> {
        phdr.map(|phdr| Self::get_segment_from_phdr(phdr, image_base_file, image_base_mem))
    }

    /// This function calculates the `load_start_addr`.
    ///
    /// # Note
    /// There are three diffrent but equally important addresses here.
    ///
    /// `image_base_file` referes to the address of the first section in the elf file - this is configured at compile time and a higher-half address.
    /// `image_base_mem` referes to the address where the first section will be loaded into memory
    /// `load_start_addr` referes to the start of the stacks which are before `image_base_mem`
    ///
    /// Since the stacks are located before the .rodata section
    /// we have `load_start_addr + total_stack_size = image_base_mem`
    fn get_load_start_addr_for_fixed_image(
        phdrs: &KernelImageProgramHeaders,
        num_cores: usize,
        kernel_stack_size: usize,
    ) -> VirtAddr {
        let image_base_file = phdrs.image_base();
        let image_base_mem = to_lower_half(image_base_file);
        let total_stack_size = num_cores * kernel_stack_size;

        image_base_mem - total_stack_size
    }

    fn get_phdrs(
        elf_image: &ElfBytes<'a, LittleEndian>,
    ) -> Result<KernelImageProgramHeaders, KernelImageError> {
        let phdrs: Vec<ProgramHeader> = elf_image
            .segments()
            .ok_or(KernelImageError::ProgramHeadersMissing)?
            .iter()
            .filter(|phdr| phdr.p_type == PT_LOAD)
            .collect();

        if phdrs.len() == 0 {
            return Err(KernelImageError::ProgramHeadersMissing);
        } else if phdrs.len() > 4 {
            // we expect at maximum 4 segments: .rodata, .code, .relro, .data
            // if there are more than 4 this function needs to be updated
            return Err(KernelImageError::ProgramHeadersInvalid);
        }

        // Normally if all segments are present the order (when using ld.lld as linker) is:
        // - .rodata
        // - .code
        // - .relro
        // - .data
        //
        // Now the only segment that is strictly required (by us) is the .code segment.
        // If there is only one segment with PHDR_DATA flags it is the .data segment, if
        // there are two, then the first is the .relro segment and the second is the .data segment

        let rodata = phdrs
            .iter()
            .find(|&&phdr| phdr.p_flags == PHDR_RODATA)
            .map(|phdr| *phdr);

        let code = *phdrs
            .iter()
            .find(|&&phdr| phdr.p_flags == PHDR_CODE)
            .ok_or(KernelImageError::CodeSegmentMissing)?;

        let mut data_phdrs = phdrs.iter().filter(|&&phdr| phdr.p_flags == PHDR_DATA);
        let first_data = data_phdrs.next();
        let second_data = data_phdrs.next();

        let (relro, data) = match (first_data, second_data) {
            (None, None) => (None, None),
            (Some(data), None) => (None, Some(*data)),
            (Some(relro), Some(data)) => (Some(*relro), Some(*data)),
            (None, Some(_)) => panic!("invalid state"),
        };

        Ok(KernelImageProgramHeaders {
            rodata,
            code,
            relro,
            data,
        })
    }
}

impl<'a> KernelImage<'a> {
    pub fn kernel_image_info(&self) -> &KernelImageInfo {
        &self.info
    }

    pub fn kernel_entry_point(&self) -> VirtAddr {
        self.entry_point
    }

    pub fn elf_image(&self) -> &ElfBytes<'a, LittleEndian> {
        &self.elf_image
    }

    pub fn kernel_stack_size(&self) -> usize {
        self.kernel_stack_size
    }

    pub fn load_kernel(&self) -> Result<(), KernelImageError> {
        let image_base_file = self.phdrs.image_base();
        let image_base_mem = self.info.image_base();

        if let (Some(phdr), Some(range)) = (self.phdrs.rodata, self.info.rodata) {
            self.load_segment(image_base_mem, image_base_file, phdr, range)?;
        }

        self.load_segment(
            image_base_mem,
            image_base_file,
            self.phdrs.code,
            self.info.code,
        )?;

        if let (Some(phdr), Some(range)) = (self.phdrs.relro, self.info.relro) {
            self.load_segment(image_base_mem, image_base_file, phdr, range)?;
            self.perform_relocations(image_base_mem, image_base_file, phdr, range)?;
        }

        if let (Some(phdr), Some(range)) = (self.phdrs.data, self.info.data) {
            self.load_segment(image_base_mem, image_base_file, phdr, range)?;
        }

        Ok(())
    }

    fn load_segment(
        &self,
        image_base_mem: VirtAddr,
        image_base_file: VirtAddr,
        program_header: ProgramHeader,
        segment: VirtualRange,
    ) -> Result<(), KernelImageError> {
        let zero_start = segment.start().to_addr();

        // Note: casts from u64 to usize are still correct on 32-bit targets
        let addr_in_file = VirtAddr::new(program_header.p_vaddr as usize);
        let load_start = image_base_mem + (addr_in_file - image_base_file);
        let load_end = load_start + program_header.p_filesz as usize;

        let zero_end = segment.end().to_addr();

        debug_assert!(zero_start <= load_start && load_start <= load_end && load_end <= zero_end);

        // clear out any bytes before
        unsafe {
            core::ptr::write_bytes(zero_start.as_ptr_mut::<u8>(), 0, load_start - zero_start);
        }

        // copy bytes from data to memory
        unsafe {
            let data_ptr = self.elf_image.segment_data(&program_header)?.as_ptr();

            // Note: cast from u64 to usize are still correct on 32-bit targets
            core::ptr::copy(
                data_ptr,
                load_start.as_ptr_mut(),
                program_header.p_filesz as usize,
            );
        }

        // clear out any bytes after
        unsafe {
            core::ptr::write_bytes(load_end.as_ptr_mut::<u8>(), 0, zero_end - load_end);
        }

        Ok(())
    }

    fn perform_relocations(
        &self,
        image_base_mem: VirtAddr,
        image_base_file: VirtAddr,
        relro_phdr: ProgramHeader,
        _relro_range: VirtualRange,
    ) -> Result<(), KernelImageError> {
        // only do relocations if it is necessary
        if image_base_mem == image_base_file {
            return Ok(());
        }

        let addr_in_file = VirtAddr::new(relro_phdr.p_vaddr as usize);
        let load_start = image_base_mem + (addr_in_file - image_base_file);
        let load_end = load_start + relro_phdr.p_filesz as usize;

        let mut ptr = load_start.as_ptr_mut::<usize>();
        let end = load_end.as_ptr_mut::<usize>();

        debug_assert!(ptr <= end);

        while ptr < end {
            unsafe {
                let mut data = core::ptr::read(ptr);

                if is_higher_half_address(data) {
                    let addr = VirtAddr::new(data);
                    let reloc = image_base_mem + (addr - image_base_file);
                    data = reloc.to_inner();
                }

                core::ptr::write(ptr, data);
                ptr = ptr.add(1);
            }
        }

        Ok(())
    }
}

fn is_higher_half_address(addr: usize) -> bool {
    addr >= KERNEL_BASE
}
