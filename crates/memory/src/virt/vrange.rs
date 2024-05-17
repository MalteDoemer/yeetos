use core::ops::Range;

use crate::virt::Page;

use super::VirtAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualRange {
    start: Page,
    end: Page,
}

impl VirtualRange {
    /// Creates a new `VirtualRange`.
    ///
    /// # Panics
    /// If `start + num_pages` overflows.
    pub const fn with_size(start: Page, num_pages: usize) -> Self {
        let end = start.add(num_pages);
        VirtualRange { start, end }
    }

    /// Creates a new `VirtualRange`.
    ///
    /// # Panics
    /// If `end < start`
    pub const fn new(start: Page, end: Page) -> Self {
        assert!(end.to_inner() >= start.to_inner());
        VirtualRange { start, end }
    }

    pub const fn zero() -> Self {
        VirtualRange {
            start: Page::zero(),
            end: Page::zero(),
        }
    }

    pub const fn start(&self) -> Page {
        self.start
    }

    pub const fn start_addr(&self) -> VirtAddr {
        self.start.to_addr()
    }

    pub const fn num_pages(&self) -> usize {
        self.end().diff(self.start())
    }

    pub const fn end(&self) -> Page {
        self.end
    }

    pub const fn end_addr(&self) -> VirtAddr {
        self.end().to_addr()
    }

    pub const fn pages(&self) -> Range<Page> {
        self.start()..self.end()
    }

    /// Checks if this virtual range contains the given page.
    pub fn contains_page(&self, page: Page) -> bool {
        page >= self.start() && page < self.end()
    }

    /// Checks if this virtual range completely contains the other range.
    pub fn contains_range(&self, range: VirtualRange) -> bool {
        range.start() >= self.start() && range.end() <= self.end()
    }

    /// Checks if this virtual range overlaps with another virtual range.
    pub fn overlaps_with(&self, range: VirtualRange) -> bool {
        self.contains_page(range.start()) || range.contains_page(self.start())
    }

    /// Creates the union of both ranges i.e. `min(self.start(), range.start())..max(self.end(), range.end())`
    pub fn union_with(&self, range: VirtualRange) -> VirtualRange {
        let start = core::cmp::min(self.start(), range.start());
        let end = core::cmp::max(self.end(), range.end());

        VirtualRange::new(start, end)
    }
}
