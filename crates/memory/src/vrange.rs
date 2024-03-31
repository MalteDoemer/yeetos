use core::ops::Range;

use crate::Page;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualRange {
    start: Page,
    num_pages: usize,
}

impl VirtualRange {
    /// Creates a new `VirtualRange`.
    ///
    /// Returns `None` if `start + num_pages` overflows.
    pub const fn checked_new(start: Page, num_pages: usize) -> Option<Self> {
        let rng = VirtualRange { start, num_pages };

        if rng.overflows() {
            None
        } else {
            Some(rng)
        }
    }

    /// Creates a new `VirtualRange`.
    ///
    /// # Panics
    /// If `start + num_pages` overflows.
    pub const fn new(start: Page, num_pages: usize) -> Self {
        Self::checked_new(start, num_pages).unwrap()
    }

    /// Creates a new `VirtualRange`.
    ///
    /// # Panics
    /// If `end - start` underflows.
    pub const fn new_diff(start: Page, end: Page) -> Self {
        let num_pages = end.diff(start);
        VirtualRange { start, num_pages }
    }

    pub const fn zero() -> Self {
        VirtualRange {
            start: Page::zero(),
            num_pages: 0,
        }
    }

    pub const fn start(&self) -> Page {
        self.start
    }

    pub const fn num_pages(&self) -> usize {
        self.num_pages
    }

    /// # Panics
    /// If `start + num_pages` overflows.
    pub const fn end(&self) -> Page {
        self.checked_end().unwrap()
    }

    /// Returns `None` if `start + num_pages` overflows.
    pub const fn checked_end(&self) -> Option<Page> {
        self.start.checked_add(self.num_pages)
    }

    pub const fn overflows(&self) -> bool {
        self.checked_end().is_none()
    }

    /// # Panics
    /// If `start + num_pages` overflows.
    pub const fn pages(&self) -> Range<Page> {
        self.start()..self.end()
    }

    /// Checks if this virtual range contains the given page.
    ///
    /// # Panics
    /// If `start + num_pages` overflows.
    pub fn contains_page(&self, page: Page) -> bool {
        page >= self.start && page < self.end()
    }

    /// Checks if this virtual range completly contains the other range.
    ///
    /// # Panics
    /// If `start + num_pages` overflows.
    pub fn contains_range(&self, range: VirtualRange) -> bool {
        range.start() >= self.start() && range.end() <= self.end()
    }

    /// Checks if this virtual range overlaps with another virtual range.
    ///
    /// # Panics
    /// If `start + num_pages` overflows.
    pub fn overlaps_with(&self, range: VirtualRange) -> bool {
        self.contains_page(range.start()) || range.contains_page(self.start())
    }

    /// Creates the union of both ranges i.e. `min(self.start(), range.start())..max(self.end(), range.end())`
    ///
    /// # Panics
    /// If self.end() or range.end() would overflow.
    pub fn union_with(&self, range: VirtualRange) -> VirtualRange {
        let start = core::cmp::min(self.start(), range.start());
        let end = core::cmp::max(self.end(), range.end());
        let num_pages = end.diff(start);

        VirtualRange::new(start, num_pages)
    }
}
