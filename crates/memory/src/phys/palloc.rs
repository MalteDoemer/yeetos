use crate::phys::{Frame, Inner, PhysicalRange};

pub trait PageFrameAllocator {
    /// This function returns the range of physical memory
    /// that is managed by this allocator.
    fn range(&self) -> PhysicalRange;

    /// Allocates a frame from this allocator.
    fn alloc(&mut self) -> Option<Frame>;

    /// Deallocates a frame previously obtained by this allocator.
    fn dealloc(&mut self, frame: Frame) -> Option<()>;

    /// Checks if a frame belongs to this allocator.
    fn contains(&self, frame: Frame) -> bool {
        self.range().contains_frame(frame)
    }
}

pub struct BumpPageFrameAllocator {
    range: PhysicalRange,
    index: Inner,
}

impl BumpPageFrameAllocator {
    pub fn new(range: PhysicalRange) -> Self {
        Self { range, index: 0 }
    }
}

impl PageFrameAllocator for BumpPageFrameAllocator {
    fn range(&self) -> PhysicalRange {
        self.range
    }

    fn alloc(&mut self) -> Option<Frame> {
        if self.index < self.range.num_frames() {
            let frame = self.range.start().add(self.index);
            self.index += 1;
            Some(frame)
        } else {
            None
        }
    }

    fn dealloc(&mut self, _frame: Frame) -> Option<()> {
        // we simply don't deallocate
        Some(())
    }
}
