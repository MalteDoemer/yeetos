use memory::{virt::VirtAddr, PAGE_SIZE};
use tar_no_std::{ArchiveEntry, TarArchiveRef};
use uefi::proto::media::file::{File, FileInfo, RegularFile};

pub struct Initrd<'a> {
    data: &'a [u8],
    tar_archive: TarArchiveRef<'a>,
}

impl<'a> Initrd<'a> {
    pub fn get_number_of_pages(file: &mut RegularFile) -> usize {
        let info = file
            .get_boxed_info::<FileInfo>()
            .expect("unable to get file info");

        let total_size: usize = info.file_size().try_into().unwrap();

        total_size.next_multiple_of(PAGE_SIZE) / PAGE_SIZE
    }

    pub fn from_file(file: &mut RegularFile, buffer: &'a mut [u8]) -> Self {
        let info = file
            .get_boxed_info::<FileInfo>()
            .expect("unable to get file info");

        let total_size: usize = info.file_size().try_into().unwrap();

        assert!(buffer.len() >= total_size);

        let actual = file.read(buffer).expect("unable to read file");

        if total_size != actual {
            panic!("number of bytes read differ from file size");
        }

        Self::from_data(buffer)
    }

    pub fn from_data(data: &'a mut [u8]) -> Self {
        Self {
            data,
            tar_archive: TarArchiveRef::new(data),
        }
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
