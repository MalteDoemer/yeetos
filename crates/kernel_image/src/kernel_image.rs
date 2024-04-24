use alloc::vec::Vec;
use elf::{abi::PT_LOAD, endian::LittleEndian, segment::ProgramHeader, ElfBytes, ParseError};
use memory::{
    to_lower_half,
    virt::{Page, VirtAddr, VirtualRange},
    KERNEL_BASE, PAGE_SHIFT, PAGE_SIZE,
};

use crate::KernelImageInfo;

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
    TooManyDataSegments,
    ImageBaseNotPageAligned,
    ImageBaseNotHigherHalf,
    SegmentsNotInOrder,
    InvalidNumCores,
    InvalidHeapSize,
    InvalidStackSize,
}

impl From<ParseError> for KernelImageError {
    fn from(value: ParseError) -> Self {
        KernelImageError::ParseError(value)
    }
}

pub type FileAddr = VirtAddr;

struct ProgramHeaders {
    rodata: Option<ProgramHeader>,
    code: ProgramHeader,
    relro: Option<ProgramHeader>,
    data: Option<ProgramHeader>,
}

pub struct ParsedKernelImage<'a> {
    elf_image: ElfBytes<'a, LittleEndian>,
    phdrs: ProgramHeaders,
    num_cores: usize,
    stack_size: usize,
    heap_size: usize,
}

pub struct KernelImage<'a> {
    parsed: ParsedKernelImage<'a>,
    info: KernelImageInfo,
    entry_point: VirtAddr,
    use_reloc: bool,
}

impl ProgramHeaders {
    pub fn first_segment_addr(&self) -> FileAddr {
        // Note: casts from u64 to usize are still correct on 32-bit targets
        FileAddr::new(self.first_segment().p_vaddr as usize)
    }

    fn first_segment(&self) -> ProgramHeader {
        if let Some(rodata) = self.rodata {
            rodata
        } else {
            self.code
        }
    }

    fn last_segment(&self) -> ProgramHeader {
        if let Some(data) = self.data {
            data
        } else if let Some(relro) = self.relro {
            relro
        } else {
            self.code
        }
    }

    /// Computes the total size the phdrs will use in memory
    pub fn total_size(&self) -> usize {
        // Note: casts from u64 to usize are still correct on 32-bit targets

        let first = self.first_segment();
        let last = self.last_segment();

        let fisrt_start = FileAddr::new(first.p_vaddr as usize).page_align_down();
        let last_end = FileAddr::new((last.p_vaddr + last.p_memsz) as usize)
            .page_align_up_checked()
            .unwrap();

        last_end - fisrt_start
    }
}

impl<'a> ParsedKernelImage<'a> {
    /// Create a new `ParsedKernelImage` based on the elf data provided in `data`.
    pub fn new(
        num_cores: usize,
        stack_size: usize,
        heap_size: usize,
        data: &'a [u8],
    ) -> Result<Self, KernelImageError> {
        let elf_image: ElfBytes<'a, LittleEndian> = ElfBytes::minimal_parse(data)?;
        let phdrs = Self::get_phdrs(&elf_image)?;

        Ok(Self {
            elf_image,
            phdrs,
            num_cores,
            stack_size,
            heap_size,
        })
    }

    fn get_phdrs(
        elf_image: &ElfBytes<'a, LittleEndian>,
    ) -> Result<ProgramHeaders, KernelImageError> {
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

        let data_phdrs: Vec<ProgramHeader> = phdrs
            .iter()
            .filter(|&&phdr| phdr.p_flags == PHDR_DATA)
            .map(|phdr| *phdr)
            .collect();

        if data_phdrs.len() > 2 {
            return Err(KernelImageError::TooManyDataSegments);
        }

        let mut data_phdrs_iter = data_phdrs.iter();

        let first_data = data_phdrs_iter.next();
        let second_data = data_phdrs_iter.next();

        let (relro, data) = match (first_data, second_data) {
            (None, None) => (None, None),
            (Some(data), None) => (None, Some(*data)),
            (Some(relro), Some(data)) => (Some(*relro), Some(*data)),
            (None, Some(_)) => panic!("invalid state"),
        };

        Ok(ProgramHeaders {
            rodata,
            code,
            relro,
            data,
        })
    }

    /// The total size of the stack area which contains one stack per core.
    pub fn total_stack_size(&self) -> usize {
        self.stack_size * self.num_cores
    }

    /// The size of the initially allocated heap for the kernel.
    pub fn heap_size(&self) -> usize {
        self.heap_size
    }

    /// The combined size of the .rodata, .code, .relro, .data segments
    pub fn segment_size(&self) -> usize {
        self.phdrs.total_size()
    }

    /// The total size of the kernel image including stack and heap measured in bytes.
    pub fn total_size(&self) -> usize {
        self.segment_size() + self.total_stack_size() + self.heap_size()
    }

    /// This calculates the lower half address of the start of the image when using fixed mode
    pub fn fixed_load_addr(&self) -> VirtAddr {
        let image_base = to_lower_half(self.phdrs.first_segment_addr());
        image_base - self.total_stack_size()
    }

    pub fn verify(&self) -> Result<(), KernelImageError> {
        // we assume there are <= 256 cores
        // Note: this could be dropped in the future

        if self.num_cores == 0 || self.num_cores > 256 {
            return Err(KernelImageError::InvalidNumCores);
        }

        // stacks must be page aligned
        if !is_page_aligned(self.stack_size) {
            return Err(KernelImageError::InvalidStackSize);
        }

        // heap size must be page aligned
        if !is_page_aligned(self.heap_size) {
            return Err(KernelImageError::InvalidHeapSize);
        }

        // the first segment (i.e. the image_base) must be page aligned
        if !is_page_aligned(self.phdrs.first_segment_addr().to_inner()) {
            return Err(KernelImageError::ImageBaseNotPageAligned);
        }

        // the image_base must (for now) be a higher half address
        if !is_higher_half_address(self.phdrs.first_segment_addr().to_inner()) {
            return Err(KernelImageError::ImageBaseNotHigherHalf);
        }

        // we assume that the segments are in the order: rodata code relro data
        // thus we should check that here

        if let Some(rodata) = self.phdrs.rodata {
            if rodata.p_vaddr > self.phdrs.code.p_vaddr {
                return Err(KernelImageError::SegmentsNotInOrder);
            }
        }

        if let Some(relro) = self.phdrs.relro {
            if self.phdrs.code.p_vaddr > relro.p_vaddr {
                return Err(KernelImageError::SegmentsNotInOrder);
            }
        }

        match (self.phdrs.relro, self.phdrs.data) {
            (Some(relro), Some(data)) => {
                if relro.p_vaddr > data.p_vaddr {
                    return Err(KernelImageError::SegmentsNotInOrder);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl<'a> ParsedKernelImage<'a> {
    pub fn to_reloc_image(self, base_addr: VirtAddr) -> Result<KernelImage<'a>, KernelImageError> {
        let base_addr_aligned = base_addr.page_align_up_checked().unwrap();
        let image_base_mem = base_addr_aligned + self.total_stack_size();

        self.to_image(image_base_mem, true)
    }

    pub fn to_fixed_image(self) -> Result<KernelImage<'a>, KernelImageError> {
        let image_base_mem = to_lower_half(self.phdrs.first_segment_addr());

        self.to_image(image_base_mem, false)
    }

    fn to_image(
        self,
        image_base_mem: VirtAddr,
        use_reloc: bool,
    ) -> Result<KernelImage<'a>, KernelImageError> {
        self.verify()?;

        // caller has to assure that image_base_mem is page aligned
        debug_assert!(image_base_mem.page_align_down() == image_base_mem);

        let image_base_file = self.phdrs.first_segment_addr();

        let info = self.get_image_info(image_base_mem);
        let entry_point = self.get_entry_point(image_base_file, image_base_mem);

        Ok(KernelImage {
            parsed: self,
            info,
            entry_point,
            use_reloc,
        })
    }

    fn get_entry_point(&self, image_base_file: FileAddr, image_base_mem: VirtAddr) -> VirtAddr {
        let file_addr = FileAddr::new(self.elf_image.ehdr.e_entry as usize);

        image_base_mem + (file_addr - image_base_file)
    }

    fn get_image_info(&self, image_base_mem: VirtAddr) -> KernelImageInfo {
        let image_base_file = self.phdrs.first_segment_addr();

        let stack = self.get_stack_segment(image_base_mem);

        let rodata =
            self.get_optional_segment_from_phdr(self.phdrs.rodata, image_base_file, image_base_mem);

        let code = self.get_segment_from_phdr(self.phdrs.code, image_base_file, image_base_mem);

        let relro =
            self.get_optional_segment_from_phdr(self.phdrs.relro, image_base_file, image_base_mem);

        let data =
            self.get_optional_segment_from_phdr(self.phdrs.data, image_base_file, image_base_mem);

        let heap = self.get_heap_segment(image_base_mem);

        KernelImageInfo {
            stack,
            rodata,
            code,
            relro,
            data,
            heap,
        }
    }

    /// This function computes the range of virtual memory occupied by the stacks.
    ///
    /// - `image_base_mem` is the calculated address of the first non-stack segment in memory
    fn get_stack_segment(&self, image_base_mem: VirtAddr) -> VirtualRange {
        let stack_start = image_base_mem - self.total_stack_size();
        let stack_end = image_base_mem;

        let start_page = Page::new(stack_start);
        let end_page = Page::new(stack_end.page_align_up_checked().unwrap());

        VirtualRange::new_diff(start_page, end_page)
    }

    /// This function computes the range of virtual memory occupied by the heap.
    ///
    /// - `image_base_mem` is the calculated address of the first non-stack segment in memory
    fn get_heap_segment(&self, image_base_mem: VirtAddr) -> VirtualRange {
        let heap_start = image_base_mem + self.segment_size();
        let heap_end = heap_start + self.heap_size();

        let start_page = Page::new(heap_start);
        let end_page = Page::new(heap_end.page_align_up_checked().unwrap());

        VirtualRange::new_diff(start_page, end_page)
    }

    /// This function computes the range of virtual memory the given PHDR is going to occupy.
    ///
    /// - `image_base_file` is the image base this executable was compiled with (i.e. --image-base=... argument to the linker)
    /// - `image_base_mem` is the calculated address of the first non-stack segment in memory
    ///
    fn get_segment_from_phdr(
        &self,
        phdr: ProgramHeader,
        image_base_file: FileAddr,
        image_base_mem: VirtAddr,
    ) -> VirtualRange {
        // Note: casts from u64 to usize are still correct on 32-bit targets
        let addr_in_file = FileAddr::new(phdr.p_vaddr as usize);
        let offset = addr_in_file - image_base_file;

        let segment_start = image_base_mem + offset;
        let segment_end = segment_start + phdr.p_memsz as usize;

        let start_page = Page::new(segment_start);
        let end_page = Page::new(segment_end.page_align_up_checked().unwrap());

        VirtualRange::new_diff(start_page, end_page)
    }

    /// See `get_segment_from_phdr()`
    fn get_optional_segment_from_phdr(
        &self,
        phdr: Option<ProgramHeader>,
        image_base_file: VirtAddr,
        image_base_mem: VirtAddr,
    ) -> Option<VirtualRange> {
        phdr.map(|phdr| self.get_segment_from_phdr(phdr, image_base_file, image_base_mem))
    }
}

impl<'a> KernelImage<'a> {
    /// This is a helper function that directly uses ParsedKernelImage
    pub fn new(
        base_addr: Option<VirtAddr>,
        num_cores: usize,
        stack_size: usize,
        heap_size: usize,
        data: &'a [u8],
    ) -> Result<KernelImage, KernelImageError> {
        let parsed = ParsedKernelImage::new(num_cores, stack_size, heap_size, data)?;

        if let Some(base_addr) = base_addr {
            parsed.to_reloc_image(base_addr)
        } else {
            parsed.to_fixed_image()
        }
    }

    pub fn kernel_image_info(&self) -> &KernelImageInfo {
        &self.info
    }

    pub fn kernel_entry_point(&self) -> VirtAddr {
        self.entry_point
    }

    pub fn elf_image(&self) -> &ElfBytes<'a, LittleEndian> {
        &self.parsed.elf_image
    }

    /// The size in bytes of a single kernel stack
    pub fn kernel_stack_size(&self) -> usize {
        self.parsed.stack_size
    }

    /// The number of cores this KernelImage was initialized for.
    ///
    /// Note: this corresponds to the number of stacks.
    pub fn num_cores(&self) -> usize {
        self.parsed.num_cores
    }

    pub fn load_kernel(&self) -> Result<(), KernelImageError> {
        let image_base_file = self.parsed.phdrs.first_segment_addr();
        let image_base_mem = self.info.image_base();

        // Note: we cannot clear out the stack memory here
        // as the AP's may have already started using the stack area.

        // load rodata segment if necessary
        if let (Some(phdr), Some(range)) = (self.parsed.phdrs.rodata, self.info.rodata) {
            self.load_segment(image_base_mem, image_base_file, phdr, range)?;
        }

        // load code segment
        self.load_segment(
            image_base_mem,
            image_base_file,
            self.parsed.phdrs.code,
            self.info.code,
        )?;

        // load relro segment if necessary
        if let (Some(phdr), Some(range)) = (self.parsed.phdrs.relro, self.info.relro) {
            self.load_segment(image_base_mem, image_base_file, phdr, range)?;
            self.perform_relocations(image_base_mem, image_base_file, phdr, range)?;
        }

        // load data segment if necessary
        if let (Some(phdr), Some(range)) = (self.parsed.phdrs.data, self.info.data) {
            self.load_segment(image_base_mem, image_base_file, phdr, range)?;
        }

        // clear out heap memory
        self.clear_segment(self.info.heap);

        Ok(())
    }

    fn clear_segment(&self, segment: VirtualRange) {
        let zero_start = segment.start_addr().as_ptr_mut::<u8>();
        let size_in_bytes = segment.num_pages() * PAGE_SIZE;

        unsafe {
            core::ptr::write_bytes(zero_start, 0, size_in_bytes);
        }
    }

    fn load_segment(
        &self,
        image_base_mem: VirtAddr,
        image_base_file: VirtAddr,
        program_header: ProgramHeader,
        segment: VirtualRange,
    ) -> Result<(), KernelImageError> {
        let zero_start = segment.start_addr();

        // Note: casts from u64 to usize are still correct on 32-bit targets
        let addr_in_file = VirtAddr::new(program_header.p_vaddr as usize);
        let load_start = image_base_mem + (addr_in_file - image_base_file);
        let load_end = load_start + program_header.p_filesz as usize;

        let zero_end = segment.end_addr();

        debug_assert!(zero_start <= load_start && load_start <= load_end && load_end <= zero_end);

        // clear out any bytes before
        unsafe {
            core::ptr::write_bytes(zero_start.as_ptr_mut::<u8>(), 0, load_start - zero_start);
        }

        // copy bytes from data to memory
        unsafe {
            let data_ptr = self.elf_image().segment_data(&program_header)?.as_ptr();

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
        if !self.use_reloc {
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
/*
    fn new_reloc(base_addr: VirtAddr, parsed: ParsedKernelImage) -> KernelImage {
        let image_base_file = parsed.phdrs.first_segment_addr();
        let image_base_mem = base_addr + parsed.total_stack_size();

        let info = Self::get_image_info(
            image_base_mem,
            &parsed.phdrs,
            parsed.num_cores,
            parsed.stack_size,
            parsed.heap_size,
        );

        let entry_point = Self::get_entry_point(&parsed.elf_image, image_base_file, image_base_mem);

        KernelImage {
            parsed,
            info,
            entry_point,
        }
    }

    fn new_fixed(parsed: ParsedKernelImage) -> KernelImage {
        let image_base_file = parsed.phdrs.first_segment_addr();
        let image_base_mem = image_base_file;

        let info = Self::get_image_info(
            image_base_mem,
            &parsed.phdrs,
            parsed.num_cores,
            parsed.stack_size,
            parsed.heap_size,
        );

        let entry_point = Self::get_entry_point(&parsed.elf_image, image_base_file, image_base_mem);

        KernelImage {
            parsed,
            info,
            entry_point,
        }
    }

    fn get_entry_point(
        elf_image: &'a ElfBytes<'a, LittleEndian>,
        image_base_file: FileAddr,
        image_base_mem: VirtAddr,
    ) -> VirtAddr {
        let file_addr = FileAddr::new(elf_image.ehdr.e_entry as usize);

        image_base_mem + (file_addr - image_base_file)
    }

    fn get_image_info(
        image_base_mem: VirtAddr,
        phdrs: &ProgramHeaders,
        num_cores: usize,
        stack_size: usize,
        heap_size: usize,
    ) -> KernelImageInfo {
        let image_base_file = phdrs.first_segment_addr();

        let stack = Self::get_stack_segment(image_base_mem, num_cores, stack_size);

        let rodata =
            Self::get_optional_segment_from_phdr(phdrs.rodata, image_base_file, image_base_mem);

        let code = Self::get_segment_from_phdr(phdrs.code, image_base_file, image_base_mem);

        let relro =
            Self::get_optional_segment_from_phdr(phdrs.relro, image_base_file, image_base_mem);

        let data =
            Self::get_optional_segment_from_phdr(phdrs.data, image_base_file, image_base_mem);

        let heap = Self::get_heap_segment(image_base_mem, phdrs.total_size(), heap_size);

        KernelImageInfo {
            stack,
            rodata,
            code,
            relro,
            data,
            heap,
        }
    }

    /// This function computes the range of virtual memory occupied by the stacks.
    /// - `image_base_mem` is the calculated address of the first non-stack segment in memory
    fn get_stack_segment(
        image_base_mem: VirtAddr,
        num_cores: usize,
        stack_size: usize,
    ) -> VirtualRange {
        let stack_size = num_cores * stack_size;
        let stack_start = image_base_mem - stack_size;
        let stack_end = image_base_mem;

        let start_page = Page::new(stack_start);
        let end_page = Page::new(stack_end.page_align_up_checked().unwrap());

        VirtualRange::new_diff(start_page, end_page)
    }

    /// This function computes the range of virtual memory occupied by the heap.
    ///
    /// - `image_base_mem` is the calculated address of the first non-stack segment in memory
    /// - `total_segments_size` is the size of all .rodata, .code, .relro, .data segments.
    ///
    fn get_heap_segment(
        image_base_mem: VirtAddr,
        total_segments_size: usize,
        heap_size: usize,
    ) -> VirtualRange {
        let heap_start = image_base_mem + total_segments_size;

        let heap_end = heap_start + kernel_heap_size;

        let start_page = Page::new(heap_start);
        let end_page = Page::new(heap_end.page_align_up_checked().unwrap());

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
        image_base_file: FileAddr,
        image_base_mem: VirtAddr,
    ) -> VirtualRange {
        // Note: casts from u64 to usize are still correct on 32-bit targets
        let addr_in_file = FileAddr::new(phdr.p_vaddr as usize);
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
}
*/

fn is_page_aligned(val: usize) -> bool {
    (val >> PAGE_SHIFT) << PAGE_SHIFT == val
}

fn is_higher_half_address(addr: usize) -> bool {
    addr >= KERNEL_BASE
}
