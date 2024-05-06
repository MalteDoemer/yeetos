use memory::PAGE_SIZE;
use uefi::proto::media::file::{FileInfo, RegularFile};
use uefi::{
    cstr16,
    proto::media::{
        file::{Directory, File, FileAttribute, FileMode},
        fs::SimpleFileSystem,
    },
    table::boot::{BootServices, ScopedProtocol},
    CStr16, Error, Handle, Status,
};

const INITRD_PATH: &CStr16 = cstr16!("\\yeetos\\initrd");

pub struct BootFs<'a> {
    _sfs: ScopedProtocol<'a, SimpleFileSystem>,
    vol: Directory,
}

impl<'a> BootFs<'a> {
    pub fn new(image_handle: Handle, boot_services: &'a BootServices) -> uefi::Result<Self> {
        let mut sfs = boot_services.get_image_file_system(image_handle)?;
        let vol = sfs.open_volume()?;

        Ok(Self { _sfs: sfs, vol })
    }

    pub fn open_initrd(&mut self) -> uefi::Result<(RegularFile, usize)> {
        let mut file = self
            .vol
            .open(INITRD_PATH, FileMode::Read, FileAttribute::empty())?
            .into_regular_file()
            .ok_or(Error::new(Status::NOT_FOUND, ()))?;

        let info = file.get_boxed_info::<FileInfo>()?;
        let total_size: usize = info.file_size().try_into().unwrap();
        let pages = total_size.next_multiple_of(PAGE_SIZE) / PAGE_SIZE;

        Ok((file, pages))
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
