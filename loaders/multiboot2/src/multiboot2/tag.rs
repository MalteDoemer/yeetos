use alloc::string::{String, ToString};

use super::tag_info::*;

pub(crate) enum Tag {
    End,
    CommandLine(String),
    Unknown(u32),
}

impl Tag {
    /// Creates a `Tag` by parsing a multiboot2 tag.
    ///
    /// ### Safety
    /// - `info` must point to a valid multiboot2 tag
    pub unsafe fn parse(info: TagInfo) -> Self {
        unsafe {
            match info.tag_type() {
                END_TAG => Tag::End,
                BOOT_COMMAND_LINE_TAG => Self::parse_cmdline(info),
                BOOT_LOADER_NAME_TAG => Tag::Unknown(info.tag_type()),
                MODULE_INFO_TAG => Tag::Unknown(info.tag_type()),
                BIOS_BOOT_DEVICE_TAG => Tag::Unknown(info.tag_type()),
                BASIC_MEMORY_INFO_TAG => Tag::Unknown(info.tag_type()),
                MEMORY_MAP_TAG => Tag::Unknown(info.tag_type()),
                VBE_INFO_TAG => Tag::Unknown(info.tag_type()),
                FRAME_BUFFER_INFO_TAG => Tag::Unknown(info.tag_type()),
                ELF_SYMBOLS_TAG => Tag::Unknown(info.tag_type()),
                APM_TABLE_TAG => Tag::Unknown(info.tag_type()),
                EFI32_SYSTEM_TABLE_POINTER_TAG => Tag::Unknown(info.tag_type()),
                EFI64_SYSTEM_TABLE_POINTER_TAG => Tag::Unknown(info.tag_type()),
                SMBIOS_TABLES_TAG => Tag::Unknown(info.tag_type()),
                ACPI_RSDP_OLD_TAG => Tag::Unknown(info.tag_type()),
                ACPI_RSDP_NEW_TAG => Tag::Unknown(info.tag_type()),
                NETWORKING_INFO_TAG => Tag::Unknown(info.tag_type()),
                EFI_MEMORY_MAP_TAG => Tag::Unknown(info.tag_type()),
                EFI_BOOT_SERVICES_NOT_TERMINATED_TAG => Tag::Unknown(info.tag_type()),
                EFI32_IMAGE_HANDLE_POINTER_TAG => Tag::Unknown(info.tag_type()),
                EFI64_IMAGE_HANDLE_POINTER_TAG => Tag::Unknown(info.tag_type()),
                IMAGE_LOAD_BASE_PHYS_ADDR_TAG => Tag::Unknown(info.tag_type()),
                _ => Tag::Unknown(info.tag_type()),
            }
        }
    }

    unsafe fn parse_cmdline(info: TagInfo) -> Tag {
        let size = info.data_size();
        let ptr = info.data_addr().as_ptr::<u8>();

        // Safety:
        // function contract assures a valid tags
        let data = unsafe { core::slice::from_raw_parts(ptr, size) };

        // The boot command line string is UTF-8 with a null byte at the end.
        // In order to make a `&str` the null byte has to be ignored.
        let str_data = &data[..data.len() - 1];

        let str_slice = core::str::from_utf8(str_data).expect("boot command line not valid utf-8");

        Tag::CommandLine(str_slice.to_string())
    }
}
