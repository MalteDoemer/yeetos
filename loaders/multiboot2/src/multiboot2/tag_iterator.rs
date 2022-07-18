use memory::{MemoryReader, VirtAddr};

use super::{tag::Tag, tag_info::TagInfo};

pub(crate) struct TagIterator {
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
        let info = self.next_tag();

        // Safety:
        // Contract in new assures a valid multiboot2 structure.
        let tag = unsafe { Tag::parse(info) };

        if let Tag::End = tag {
            None
        } else {
            Some(tag)
        }
    }
}
