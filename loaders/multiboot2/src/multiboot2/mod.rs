use core::iter::Iterator;

use memory::{MemoryReader, VirtAddr};

mod taginfo;
mod tags;

use taginfo::*;
use tags::*;

pub(crate) enum Tag {
    CommandLine(CommandLineTag),
}

struct TagIterator {
    reader: MemoryReader,
    end_addr: VirtAddr,
}

impl TagIterator {
    /// Creates a new `TagIterator` for the given multiboot2 struct.
    ///
    /// ### Safety
    /// `mboot_addr` must point to a valid multiboot2 struct.
    pub unsafe fn new(mboot_addr: VirtAddr) -> Self {
        let mut reader = MemoryReader::new(mboot_addr);

        // Safety:
        // `mboot_addr` points to a valid multiboot2 struct
        unsafe {
            let size = reader.read::<u32>();
            reader.skip(4); // skip reserved field

            Self {
                reader,
                end_addr: mboot_addr + size as usize,
            }
        }
    }

    fn next_tag(&mut self) -> TagInfo {
        // Safety:
        // The contract in `new()` ensures that we can read
        // from the (valid) multiboot2 struct.
        unsafe {
            let addr = self.reader.addr();

            let tag_type = self.reader.read::<u32>();
            let tag_size = self.reader.read::<u32>();

            TagInfo::new(addr, tag_size as usize, tag_type)
        }
    }
}

impl Iterator for TagIterator {
    type Item = Tag;

    fn next(&mut self) -> Option<Self::Item> {
        let tag = self.next_tag();

        match tag.tag_type() {
            END_TAG => todo!(),
            BOOT_COMMAND_LINE_TAG => todo!(),
            BOOT_LOADER_NAME_TAG => todo!(),
            MODULE_INFO_TAG => todo!(),
            BIOS_BOOT_DEVICE_TAG => todo!(),
            BASIC_MEMORY_INFO_TAG => todo!(),
            MEMORY_MAP_TAG => todo!(),
            VBE_INFO_TAG => todo!(),
            FRAME_BUFFER_INFO_TAG => todo!(),
            ELF_SYMBOLS_TAG => todo!(),
            APM_TABLE_TAG => todo!(),
            EFI32_SYSTEM_TABLE_POINTER_TAG => todo!(),
            EFI64_SYSTEM_TABLE_POINTER_TAG => todo!(),
            SMBIOS_TABLES_TAG => todo!(),
            ACPI_RSDP_OLD_TAG => todo!(),
            ACPI_RSDP_NEW_TAG => todo!(),
            NETWORKING_INFO_TAG => todo!(),
            EFI_MEMORY_MAP_TAG => todo!(),
            EFI_BOOT_SERVICES_NOT_TERMINATED_TAG => todo!(),
            EFI32_IMAGE_HANDLE_POINTER_TAG => todo!(),
            EFI64_IMAGE_HANDLE_POINTER_TAG => todo!(),
            IMAGE_LOAD_BASE_PHYS_ADDR_TAG => todo!(),
            _ => todo!(),
        }
    }
}

pub struct Multiboot2Info {
    cmdline: Option<CommandLineTag>,
}

impl Multiboot2Info {
    /// Creates a new Multiboot2Info object by parsing
    /// the actual multiboot2 information structure passed
    /// by the bootloader.
    /// ### Safety
    /// `mboot_addr` must point to a valid multiboot2 struct
    pub unsafe fn new(mboot_addr: VirtAddr) -> Self {
        // Safety:
        // `mboot_addr` points to a valid multiboot2 struct
        let iter = unsafe { TagIterator::new(mboot_addr) };

        let mut cmdline = None;

        for tag in iter {
            match tag {
                Tag::CommandLine(tag) => cmdline = Some(tag),
            }
        }

        Multiboot2Info { cmdline }
    }

    pub(crate) fn command_line(&self) -> Option<&str> {
        self.cmdline.as_ref().map(|tag| tag.cmdline())
    }
}
