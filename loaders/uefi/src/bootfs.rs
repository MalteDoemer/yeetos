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
}
