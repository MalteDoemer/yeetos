use memory::virt::VirtAddr;

pub(crate) const END_TAG: u32 = 0;
pub(crate) const BOOT_COMMAND_LINE_TAG: u32 = 1;
pub(crate) const BOOT_LOADER_NAME_TAG: u32 = 2;
pub(crate) const MODULE_INFO_TAG: u32 = 3;
pub(crate) const BIOS_BOOT_DEVICE_TAG: u32 = 5;
pub(crate) const BASIC_MEMORY_INFO_TAG: u32 = 4;
pub(crate) const MEMORY_MAP_TAG: u32 = 6;
pub(crate) const VBE_INFO_TAG: u32 = 7;
pub(crate) const FRAME_BUFFER_INFO_TAG: u32 = 8;
pub(crate) const ELF_SYMBOLS_TAG: u32 = 9;
pub(crate) const APM_TABLE_TAG: u32 = 10;
pub(crate) const EFI32_SYSTEM_TABLE_POINTER_TAG: u32 = 11;
pub(crate) const EFI64_SYSTEM_TABLE_POINTER_TAG: u32 = 12;
pub(crate) const SMBIOS_TABLES_TAG: u32 = 13;
pub(crate) const ACPI_RSDP_OLD_TAG: u32 = 14;
pub(crate) const ACPI_RSDP_NEW_TAG: u32 = 15;
pub(crate) const NETWORKING_INFO_TAG: u32 = 16;
pub(crate) const EFI_MEMORY_MAP_TAG: u32 = 17;
pub(crate) const EFI_BOOT_SERVICES_NOT_TERMINATED_TAG: u32 = 18;
pub(crate) const EFI32_IMAGE_HANDLE_POINTER_TAG: u32 = 19;
pub(crate) const EFI64_IMAGE_HANDLE_POINTER_TAG: u32 = 20;
pub(crate) const IMAGE_LOAD_BASE_PHYS_ADDR_TAG: u32 = 21;

#[derive(Clone, Copy)]
pub(crate) struct TagInfo {
    addr: VirtAddr,
    size: usize,
    tag_type: u32,
}

impl TagInfo {
    pub fn new(addr: VirtAddr, size: usize, tag_type: u32) -> Self {
        Self {
            addr,
            size,
            tag_type,
        }
    }

    /// The address of the first data after the tag header
    pub fn data_addr(&self) -> VirtAddr {
        // type and size fields use 8 bytes
        self.start_addr() + 8
    }

    /// The size of the data after the tag header
    pub fn data_size(&self) -> usize {
        // type and size fields use 8 bytes
        self.total_size() - 8
    }

    pub fn start_addr(&self) -> VirtAddr {
        self.addr
    }

    pub fn end_addr(&self) -> VirtAddr {
        self.start_addr() + self.total_size()
    }

    pub fn total_size(&self) -> usize {
        self.size
    }

    pub fn tag_type(&self) -> u32 {
        self.tag_type
    }
}
