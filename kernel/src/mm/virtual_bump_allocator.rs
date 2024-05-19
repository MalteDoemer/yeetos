use memory::virt::{Page, VirtualRange};

pub struct VirtualBumpAllocator {
    range: VirtualRange,
    current: Page,
}

impl VirtualBumpAllocator {
    pub const fn new(range: VirtualRange) -> Self {
        let current = range.start();
        Self { range, current }
    }

    pub fn range(&self) -> VirtualRange {
        self.range
    }

    pub fn alloc(&mut self, num_pages: usize, align: usize) -> Option<VirtualRange> {
        let start_page = self.current.to_inner().checked_next_multiple_of(align)?;
        let end_page = start_page.checked_add(num_pages)?;

        let rng = VirtualRange::new(Page::from_inner(start_page), Page::from_inner(end_page));

        if self.range.contains_range(rng) {
            self.current = rng.end();
            Some(rng)
        } else {
            None
        }
    }

    pub fn alloc_specific(&mut self, range_to_allocate: VirtualRange) -> Option<()> {
        if self.current <= range_to_allocate.start() {
            self.current = range_to_allocate.end();
            Some(())
        } else {
            None
        }
    }

    pub fn dealloc(&mut self, range: VirtualRange) -> Option<()> {
        debug_assert!(self.range.contains_range(range));
        // No de-allocation for now
        Some(())
    }
}
