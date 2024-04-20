use memory::phys::Frame;

use crate::{arch, kresult::KResult};

pub trait MemoryManager {
    /// Try to allocate a single page frame.
    fn alloc_frame(&self) -> KResult<Frame>;

    /// Try to deallocate a frame previously obtained by `self.alloc_frame()`.
    fn dealloc_frame(&self, frame: Frame) -> KResult<()>;
}

#[inline(always)]
pub fn get() -> &'static impl MemoryManager {
    arch::mm::get()
}

pub fn init() {}
