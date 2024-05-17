use core::ops::Range;

use crate::phys::{Frame, Inner};

use super::PhysAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRange {
    start: Frame,
    end: Frame,
}

impl PhysicalRange {
    /// Creates a new `PhysicalRange`.
    ///
    /// # Panics
    /// If `start + num_frames` overflows.
    pub const fn with_size(start: Frame, num_frames: Inner) -> Self {
        let end = start.add(num_frames);
        PhysicalRange { start, end }
    }

    /// Creates a new `PhysicalRange`.
    ///
    /// # Panics
    /// If `end < start`
    pub const fn new(start: Frame, end: Frame) -> Self {
        assert!(end.to_inner() >= start.to_inner());
        PhysicalRange { start, end }
    }

    pub const fn zero() -> Self {
        PhysicalRange {
            start: Frame::zero(),
            end: Frame::zero(),
        }
    }

    pub const fn start(&self) -> Frame {
        self.start
    }

    pub const fn start_addr(&self) -> PhysAddr {
        self.start.to_addr()
    }

    pub const fn num_frames(&self) -> Inner {
        self.end().diff(self.start())
    }

    pub const fn end(&self) -> Frame {
        self.end
    }

    pub const fn end_addr(&self) -> PhysAddr {
        self.end().to_addr()
    }

    pub const fn frames(&self) -> Range<Frame> {
        self.start()..self.end()
    }

    /// Checks if this physical range contains the given frame.
    pub fn contains_frame(&self, frame: Frame) -> bool {
        frame >= self.start() && frame < self.end()
    }

    /// Checks if this physical range completely contains the other range.
    pub fn contains_range(&self, range: PhysicalRange) -> bool {
        range.start() >= self.start() && range.end() <= self.end()
    }

    /// Checks if this physical range overlaps with another physical range.
    pub fn overlaps_with(&self, range: PhysicalRange) -> bool {
        self.contains_frame(range.start()) || range.contains_frame(self.start())
    }

    /// Creates the union of both ranges i.e. `min(self.start(), range.start())..max(self.end(), range.end())`
    pub fn union_with(&self, range: PhysicalRange) -> PhysicalRange {
        let start = core::cmp::min(self.start(), range.start());
        let end = core::cmp::max(self.end(), range.end());
        let num_frames = end.diff(start);

        PhysicalRange::with_size(start, num_frames)
    }
}
