use crate::phys::Frame;
use alloc::boxed::Box;

pub trait PageFrameAllocator {
    /// Allocates a frame from this allocator.
    fn alloc(&mut self) -> Option<Frame>;

    /// Allocate multiple frames at once from this allocator.
    fn alloc_multiple(&mut self, num_frames: usize) -> Option<Box<[Frame]>>;

    /// Deallocates a frame previously obtained by this allocator.
    fn dealloc(&mut self, frame: Frame);

    /// Checks if a frame belongs to this allocator.
    /// # Note
    /// This function should not check whether this frame was actually allocated
    /// from this allocator using alloc(), just if the frame belongs to the physical range of
    /// memory managed by this allocator.
    fn contains(&self, frame: Frame) -> bool;
}
