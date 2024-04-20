use memory::virt::{MemoryReader, VirtAddr};

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

    fn current_tag(&self) -> TagInfo {
        // Safety:
        // The contract in `new()` ensures that we can read
        // from the (valid) multiboot2 struct.
        unsafe {
            let addr = self.reader.addr();

            let ptr = self.reader.as_ptr::<u32>();

            let tag_type = *ptr;
            let tag_size = *ptr.add(1);

            TagInfo::new(addr, tag_size as usize, tag_type)
        }
    }

    fn skip_to_next_tag(&mut self) {
        let current_tag = self.current_tag();

        self.reader.skip(current_tag.total_size()); // skip the over the tag
        self.reader.align_up(8); // tags are 8-byte aligned
    }
}

impl Iterator for TagIterator {
    type Item = Tag;

    fn next(&mut self) -> Option<Self::Item> {
        let info = self.current_tag();

        // Safety:
        // Contract in new assures a valid multiboot2 structure.
        let tag = unsafe { Tag::parse(info) };

        self.skip_to_next_tag();

        if let Tag::End = tag {
            None
        } else {
            Some(tag)
        }
    }
}
