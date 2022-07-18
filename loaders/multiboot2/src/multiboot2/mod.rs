use alloc::string::String;
use memory::VirtAddr;

use self::{tag::Tag, tag_iterator::TagIterator};

mod tag;
mod tag_info;
mod tag_iterator;

pub struct Multiboot2Info {
    cmdline: Option<String>,
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
                _ => {}
            }
        }

        Multiboot2Info { cmdline }
    }

    pub(crate) fn command_line(&self) -> Option<&String> {
        self.cmdline.as_ref()
    }
}
