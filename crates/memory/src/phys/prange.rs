use core::ops::Range;

use crate::phys::{Frame, Inner};

use super::PhysAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRange {
    start: Frame,
    num_frames: Inner,
}

impl PhysicalRange {
    /// Creates a new `PhysicalRange`.
    ///
    /// # Panics
    /// If `start + num_frames` overflows.
    pub const fn new(start: Frame, num_frames: Inner) -> Self {
        Self::checked_new(start, num_frames).unwrap()
    }

    /// Creates a new `PhysicalRange`.
    ///
    /// Returns `None` if `start + num_frames` overflows.
    pub const fn checked_new(start: Frame, num_frames: Inner) -> Option<Self> {
        let rng = PhysicalRange { start, num_frames };

        if rng.overflows() {
            None
        } else {
            Some(rng)
        }
    }

    /// Creates a new `PhysicalRange`.
    ///
    /// # Panics
    /// If `end - start` underflows.
    pub const fn new_diff(start: Frame, end: Frame) -> Self {
        let num_frames = end.diff(start);
        PhysicalRange { start, num_frames }
    }

    pub const fn zero() -> Self {
        PhysicalRange {
            start: Frame::zero(),
            num_frames: 0,
        }
    }

    pub const fn start(&self) -> Frame {
        self.start
    }

    pub const fn start_addr(&self) -> PhysAddr {
        self.start.to_addr()
    }

    pub const fn num_frames(&self) -> Inner {
        self.num_frames
    }

    /// # Panics
    /// If `start + num_frames` overflows.
    pub const fn end(&self) -> Frame {
        self.checked_end().unwrap()
    }

    /// # Panics
    /// If `start + num_frames` overflows.
    pub const fn end_addr(&self) -> PhysAddr {
        self.end().to_addr()
    }

    /// Returns `None` if `start + num_frames` overflows.
    pub const fn checked_end(&self) -> Option<Frame> {
        self.start.checked_add(self.num_frames)
    }

    /// Returns `None` if `start + num_frames` overflows.
    pub const fn checked_end_addr(&self) -> Option<PhysAddr> {
        if let Some(end) = self.checked_end() {
            Some(end.to_addr())
        } else {
            None
        }
    }

    pub const fn overflows(&self) -> bool {
        self.checked_end().is_none()
    }

    /// # Panics
    /// If `start + num_frames` overflows.
    pub const fn frames(&self) -> Range<Frame> {
        self.start()..self.end()
    }

    /// Checks if this physical range contains the given frame.
    ///
    /// # Panics
    /// If `start + num_frames` overflows.
    pub fn contains_frame(&self, frame: Frame) -> bool {
        frame >= self.start && frame < self.end()
    }

    /// Checks if this physical range completly contains the other range.
    ///
    /// # Panics
    /// If `start + num_frames` overflows.
    pub fn contains_range(&self, range: PhysicalRange) -> bool {
        range.start() >= self.start() && range.end() <= self.end()
    }

    /// Checks if this physical range overlaps with another physical range.
    ///
    /// # Panics
    /// If `start + num_frames` overflows.
    pub fn overlaps_with(&self, range: PhysicalRange) -> bool {
        self.contains_frame(range.start()) || range.contains_frame(self.start())
    }

    /// Creates the union of both ranges i.e. `min(self.start(), range.start())..max(self.end(), range.end())`
    ///
    /// # Panics
    /// If self.end() or range.end() would overflow.
    pub fn union_with(&self, range: PhysicalRange) -> PhysicalRange {
        let start = core::cmp::min(self.start(), range.start());
        let end = core::cmp::max(self.end(), range.end());
        let num_frames = end.diff(start);

        PhysicalRange::new(start, num_frames)
    }
}
