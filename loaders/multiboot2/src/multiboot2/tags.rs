//! This module defines structs used to parse and store the multiboot2 tags.
//! For more information about the memory layout of the multiboot2 struct visit
//! https://www.gnu.org/software/grub/manual/multiboot2/multiboot.pdf

use alloc::string::{String, ToString};

use super::taginfo::*;

pub(crate) struct CommandLineTag {
    cmdline: String,
}

impl CommandLineTag {
    /// Creates a `CommandLineTag` from a multiboot2 tag.
    ///
    /// ### Panics
    /// - `info.tag_type()` is not `BOOT_COMMAND_LINE_TAG`
    ///
    /// ### Safety:
    /// - `info` must point to a valid multiboot2 tag
    pub unsafe fn parse(info: TagInfo) -> Self {
        assert!(info.tag_type() == BOOT_COMMAND_LINE_TAG);

        let size = info.data_size();
        let ptr = info.data_addr().as_ptr::<u8>();

        // Safety:
        // function contract assures a valid tags
        let data = unsafe { core::slice::from_raw_parts(ptr, size) };

        // The boot command line string is UTF-8 with a null byte at the end.
        // In order to make a `&str` the null byte has to be ignored.
        let str_data = &data[..data.len() - 1];

        let str_slice = core::str::from_utf8(str_data).expect("boot command line not valid utf-8");

        CommandLineTag::new(str_slice.to_string())
    }

    pub fn new(cmdline: String) -> Self {
        Self { cmdline }
    }

    pub(crate) fn cmdline(&self) -> &str {
        self.cmdline.as_ref()
    }
}
