use crate::{
    phys::{Inner, PhysAddr},
    FRAME_SHIFT,
};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frame(Inner);

impl Frame {
    /// Returns the frame which is containing `addr`.
    pub const fn new(paddr: PhysAddr) -> Self {
        Frame(paddr.to_inner() >> FRAME_SHIFT)
    }

    pub const fn from_inner(inner: Inner) -> Self {
        Frame(inner)
    }

    /// Returns the zero frame
    pub const fn zero() -> Self {
        Frame(0)
    }

    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Returns the base address of this frame.
    pub const fn to_addr(self) -> PhysAddr {
        PhysAddr::new(self.0 << FRAME_SHIFT)
    }

    /// Returns the frame number of this frame.
    pub const fn to_inner(self) -> Inner {
        self.0
    }

    /// Performs unsigned subtraction: `self.0 - other.0`
    pub const fn diff(self, other: Frame) -> Inner {
        self.checked_diff(other).unwrap()
    }

    /// Performs unsigned subtraction: `self.0 - other.0`
    pub const fn checked_diff(self, other: Frame) -> Option<Inner> {
        self.0.checked_sub(other.0)
    }

    pub const fn add(self, other: Inner) -> Self {
        self.checked_add(other).unwrap()
    }

    pub const fn checked_add(self, other: Inner) -> Option<Self> {
        let res = self.0.checked_add(other);

        match res {
            Some(res) => Some(Frame::from_inner(res)),
            None => None,
        }
    }
}

impl core::iter::Step for Frame {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        let diff = end.to_inner().checked_sub(start.to_inner())?;
        let diff = diff.try_into().ok()?;
        Some(diff)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let count = count.try_into().ok()?;
        let next = start.to_inner().checked_add(count)?;
        Some(Frame::from_inner(next))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let count = count.try_into().ok()?;
        let prev = start.to_inner().checked_sub(count)?;
        Some(Frame::from_inner(prev))
    }
}

impl core::fmt::Debug for Frame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Frame({:#x})", self.0)
    }
}
