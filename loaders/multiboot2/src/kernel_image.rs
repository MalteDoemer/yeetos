use alloc::vec::Vec;
use boot_info::kernel_image_info::KernelImageInfo;
use elf::{abi::PT_LOAD, endian::LittleEndian, segment::ProgramHeader, ElfBytes, ParseError};
use memory::{Page, VirtAddr, VirtualRange};

pub const PHDR_EXEC: u32 = 1;
pub const PHDR_WRITE: u32 = 2;
pub const PHDR_READ: u32 = 4;

#[derive(Debug)]
pub enum KernelImageError {
    ParseError(ParseError),
    ProgramHeadersMissing,
    ProgramHeadersInvalid,
}

impl From<ParseError> for KernelImageError {
    fn from(value: ParseError) -> Self {
        KernelImageError::ParseError(value)
    }
}

pub struct KernelImage<'a> {
    load_addr: VirtAddr,
    num_cores: usize,
    program_headers: [ProgramHeader; 4],
    entry_point: usize,
    elf_image: ElfBytes<'a, LittleEndian>,
}

impl<'a> KernelImage<'a> {
    pub fn new(
        load_addr: VirtAddr,
        num_cores: usize,
        data: &'a [u8],
    ) -> Result<Self, KernelImageError> {
        let elf_image = ElfBytes::minimal_parse(data)?;

        let program_headers = elf_image
            .segments()
            .ok_or(KernelImageError::ProgramHeadersMissing)?;

        let program_headers = Self::parse_program_headers(
            program_headers
                .into_iter()
                .filter(|phdr| phdr.p_type == PT_LOAD)
                .collect(),
        )?;

        let entry_point = elf_image.ehdr.e_entry as usize;

        Ok(KernelImage {
            load_addr,
            num_cores,
            program_headers,
            entry_point,
            elf_image,
        })
    }

    fn parse_program_headers(
        program_headers: Vec<ProgramHeader>,
    ) -> Result<[ProgramHeader; 4], KernelImageError> {
        // The kernel should always have exactly four program headers:
        // RODATA, CODE, DATA(RELRO), DATA(NON-RELRO)
        if program_headers.len() != 4 {
            return Err(KernelImageError::ProgramHeadersInvalid);
        }

        // RODATA segment is readonly
        if program_headers[0].p_flags != PHDR_READ {
            return Err(KernelImageError::ProgramHeadersInvalid);
        }

        // CODE segment is read/execute
        if program_headers[1].p_flags != PHDR_READ | PHDR_EXEC {
            return Err(KernelImageError::ProgramHeadersInvalid);
        }

        // RELRO is read/write
        if program_headers[2].p_flags != PHDR_READ | PHDR_WRITE {
            return Err(KernelImageError::ProgramHeadersInvalid);
        }

        // DATA is read/write
        if program_headers[3].p_flags != PHDR_READ | PHDR_WRITE {
            return Err(KernelImageError::ProgramHeadersInvalid);
        }

        let program_headers = [
            program_headers[0],
            program_headers[1],
            program_headers[2],
            program_headers[3],
        ];

        Ok(program_headers)
    }
}

impl<'a> KernelImage<'a> {
    pub fn get_kernel_image_info(&self) -> KernelImageInfo {
        let stack = self.get_stack_vrange();

        let load_base = stack.end().to_addr();

        let rodata = Self::get_kernel_segment(load_base, self.program_headers[0]);
        let code = Self::get_kernel_segment(load_base, self.program_headers[1]);
        let relro = Self::get_kernel_segment(load_base, self.program_headers[2]);
        let data = Self::get_kernel_segment(load_base, self.program_headers[3]);

        assert!(
            !stack.overlaps_with(rodata)
                && !rodata.overlaps_with(code)
                && !code.overlaps_with(relro)
        );

        KernelImageInfo {
            stack,
            rodata,
            code,
            relro,
            data,
        }
    }

    pub fn load_kernel(&self) -> Result<(), KernelImageError> {
        let info = self.get_kernel_image_info();

        let load_base = info.stack.end().to_addr();

        self.load_segment(load_base, self.program_headers[0], info.rodata)?;
        self.load_segment(load_base, self.program_headers[1], info.code)?;
        self.load_segment(load_base, self.program_headers[2], info.relro)?;
        self.load_segment(load_base, self.program_headers[3], info.data)?;

        Ok(())
    }

    fn load_segment(
        &self,
        load_base: VirtAddr,
        program_header: ProgramHeader,
        segment: VirtualRange,
    ) -> Result<(), KernelImageError> {
        let zero_start = segment.start().to_addr();
        let load_start = load_base + program_header.p_vaddr as usize;
        let load_end = load_start + program_header.p_filesz as usize;
        let zero_end = segment.end().to_addr();

        assert!(zero_start <= load_start && load_start <= load_end && load_end <= zero_end);

        // clear out any bytes before
        unsafe {
            core::ptr::write_bytes(zero_start.as_ptr_mut::<u8>(), 0, load_start - zero_start);
        }

        // copy bytes from data to memory
        unsafe {
            let data_ptr = self.elf_image.segment_data(&program_header)?.as_ptr();

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

    pub fn get_kernel_stack_size(&self) -> usize {
        64 * 1024
    }

    pub fn get_kernel_entry_point(&self) -> VirtAddr {
        let info = self.get_kernel_image_info();
        info.stack.end().to_addr() + self.entry_point
    }

    fn get_stack_vrange(&self) -> VirtualRange {
        let stack_start = self.load_addr;
        let stack_size = self.get_kernel_stack_size() * self.num_cores;
        let stack_end = stack_start + stack_size;

        let start_page = Page::new(stack_start);
        let end_page = Page::new(stack_end.page_align_up_checked().unwrap());

        VirtualRange::new_diff(start_page, end_page)
    }

    fn get_kernel_segment(load_base: VirtAddr, phdr: ProgramHeader) -> VirtualRange {
        // Note: p_vaddr is configured to be relative to address 0x00 at compile time
        let load_addr = load_base + phdr.p_vaddr as usize;
        let load_end = load_addr + phdr.p_memsz as usize;

        let start_page = Page::new(load_addr);
        let end_page = Page::new(load_end.page_align_up_checked().unwrap());

        VirtualRange::new_diff(start_page, end_page)
    }
}
