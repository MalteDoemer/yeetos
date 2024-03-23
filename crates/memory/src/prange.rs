use core::ops::Range;

use crate::{frame, Frame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRange {
    start: Frame,
    num_frames: frame::Inner,
}

impl PhysicalRange {
    /// Creates a new `PhysicalRange`.
    ///
    /// # Panics
    /// If `start + num_frames` overflows.
    pub const fn new(start: Frame, num_frames: frame::Inner) -> Self {
        Self::checked_new(start, num_frames).unwrap()
    }

    /// Creates a new `PhysicalRange`.
    ///
    /// Returns `None` if `start + num_frames` overflows.
    pub const fn checked_new(start: Frame, num_frames: frame::Inner) -> Option<Self> {
        let rng = PhysicalRange { start, num_frames };

        if rng.overflows() {
            None
        } else {
            Some(rng)
        }
    }

    pub const fn start(&self) -> Frame {
        self.start
    }

    pub const fn num_frames(&self) -> frame::Inner {
        self.num_frames
    }

    /// # Panics
    /// If `start + num_frames` overflows.
    pub const fn end(&self) -> Frame {
        self.checked_end().unwrap()
    }

    /// Returns `None` if `start + num_frames` overflows.
    pub const fn checked_end(&self) -> Option<Frame> {
        self.start.checked_add(self.num_frames)
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
    pub fn contains_frame(&self, frame: Frame) -> bool  {
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
        range.start() >= self.start() && range.start() <= self.end()
            || range.end() >= self.start() && range.end() <= self.end()
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
