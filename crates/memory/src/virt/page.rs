use crate::{virt::VirtAddr, PAGE_SHIFT};

type Inner = usize;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Page(Inner);

impl Page {
    /// Returns the `Page` that contains `addr`.
    pub const fn new(addr: VirtAddr) -> Self {
        Page(addr.to_inner() >> PAGE_SHIFT)
    }

    #[cfg(target_arch = "x86_64")]
    pub const fn is_canonical(&self) -> bool {
        let addr = self.to_addr().to_inner();
        addr < 0x0000_8000_0000_0000 || addr >= 0xffff_8000_0000_0000
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

    /// Performs unsigned subtraction: `self.0 - other.0`
    pub const fn diff(self, other: Page) -> Inner {
        self.checked_diff(other).unwrap()
    }

    /// Performs unsigned subtraction: `self.0 - other.0`
    pub const fn checked_diff(self, other: Page) -> Option<Inner> {
        self.0.checked_sub(other.0)
    }

    pub const fn add(self, other: Inner) -> Page {
        self.checked_add(other).unwrap()
    }

    pub const fn checked_add(self, other: Inner) -> Option<Page> {
        let res = self.0.checked_add(other);

        match res {
            Some(res) => Some(Page::from_inner(res)),
            None => None,
        }
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
