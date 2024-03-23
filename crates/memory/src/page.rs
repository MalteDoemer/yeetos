use crate::{VirtAddr, PAGE_SHIFT};

type Inner = usize;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Page(Inner);

impl Page {
    /// Returns the `Page` that contains `addr`.
    /// # Note
    /// `addr` has to be in canonical form.
    #[cfg(target_arch = "x86_64")]
    pub const fn new(addr: VirtAddr) -> Self {
        let inner = addr.to_inner();
        assert!(inner <= 0x0000_8000_0000_0000 || inner >= 0xffff_8000_0000_0000);
        Page(inner >> PAGE_SHIFT)
    }

    pub const fn from_inner(inner: Inner) -> Self {
        Page(inner)
    }

    /// Returns the zero page.
    pub const fn zero() -> Self {
        Page(0)
    }

    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Returns the base address of this page.
    pub const fn to_addr(self) -> VirtAddr {
        VirtAddr::new(self.0 << PAGE_SHIFT)
    }

    /// Returns the page number of this page.
    pub const fn to_inner(self) -> Inner {
        self.0
    }

    /// Calculates the number of frames in `self..other`
    /// # Note
    /// `other` must be greater or equal than `self`
    pub const fn diff(self, other: Page) -> Inner {
        self.checked_diff(other).unwrap()
    }

    /// Calculates the number of frame in `self..other`
    pub const fn checked_diff(self, other: Page) -> Option<Inner> {
        other.0.checked_sub(self.0)
    }
}

impl core::iter::Step for Page {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        let diff = end.to_inner().checked_sub(start.to_inner())?;
        Some(diff)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let next = start.to_inner().checked_add(count)?;
        Some(Page::from_inner(next))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let prev = start.to_inner().checked_sub(count)?;
        Some(Page::from_inner(prev))
    }
}

impl core::fmt::Debug for Page {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Page({:#x})", self.0)
    }
}
