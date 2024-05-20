use alloc::boxed::Box;
use alloc::vec::Vec;
use memory::phys::{Frame, PageFrameAllocator, PhysicalRange};

#[derive(Copy, Clone)]
pub struct FixedFrameAllocator {
    is_allocated: bool,
    range: PhysicalRange,
}

impl FixedFrameAllocator {
    pub fn new(range: PhysicalRange) -> Self {
        Self {
            is_allocated: false,
            range,
        }
    }

    pub fn num_frames(&self) -> usize {
        self.range.num_frames() as usize
    }
}

impl PageFrameAllocator for FixedFrameAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        None
    }

    fn alloc_multiple(&mut self, num_frames: usize) -> Option<Box<[Frame]>> {
        if self.is_allocated {
            return None;
        }

        if num_frames != self.num_frames() {
            return None;
        }

        let mut vec = Vec::new();

        vec.try_reserve(num_frames).ok()?;
        vec.extend(self.range.frames());

        self.is_allocated = true;

        Some(vec.into_boxed_slice())
    }

    fn alloc_specific(&mut self, _frame: Frame) -> Option<()> {
        None
    }

    fn dealloc(&mut self, frame: Frame) {
        debug_assert!(self.contains(frame));
        self.is_allocated = false;
    }

    fn contains(&self, frame: Frame) -> bool {
        self.range.contains_frame(frame)
    }
}
