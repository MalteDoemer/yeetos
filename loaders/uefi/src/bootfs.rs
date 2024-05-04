use memory::PAGE_SIZE;
use uefi::proto::media::file::{FileInfo, RegularFile};
use uefi::{
    proto::media::{
        file::{Directory, File, FileAttribute, FileHandle, FileMode},
        fs::SimpleFileSystem,
    },
    table::boot::{BootServices, ScopedProtocol},
    CStr16, Error, Handle,
};

pub struct BootFs<'a> {
    _sfs: ScopedProtocol<'a, SimpleFileSystem>,
    vol: Directory,
}

impl<'a> BootFs<'a> {
    pub fn new(image_handle: Handle, boot_services: &'a BootServices) -> Self {
        let mut sfs = boot_services
            .get_image_file_system(image_handle)
            .expect("unable to get image file system");

        let vol = sfs
            .open_volume()
            .expect("unable to open volume on image file system");

        Self { _sfs: sfs, vol }
    }

    pub fn open_file_readonly(&mut self, name: &CStr16) -> Result<FileHandle, Error> {
        self.vol.open(name, FileMode::Read, FileAttribute::empty())
    }

    pub fn file_size_in_pages(&self, file: &mut RegularFile) -> usize {
        let info = file
            .get_boxed_info::<FileInfo>()
            .expect("unable to get file info");

        let total_size: usize = info.file_size().try_into().unwrap();

        total_size.next_multiple_of(PAGE_SIZE) / PAGE_SIZE
    }

    pub fn load_file(&self, file: &mut RegularFile, buffer: &'a mut [u8]) -> uefi::Result {
        let info = file.get_boxed_info::<FileInfo>()?;

        let total_size: usize = info.file_size().try_into().unwrap();
        assert!(buffer.len() >= total_size);

        let actual = file.read(buffer)?;

        if total_size != actual {
            panic!("number of bytes read differ from file size (this case is not yet handled)");
        }
        
        Ok(())
    }
}
