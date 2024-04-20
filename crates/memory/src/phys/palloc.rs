use crate::phys::{Frame, PhysicalRange};

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
