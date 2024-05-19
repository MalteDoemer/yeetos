use memory::phys::{Frame, Inner, PhysicalRange};

pub struct FrameBumpAllocator {
    range: PhysicalRange,
    index: Inner,
}

impl FrameBumpAllocator {
    pub fn new(range: PhysicalRange) -> Self {
        Self { range, index: 0 }
    }

    pub fn range(&self) -> PhysicalRange {
        self.range
    }

    pub fn contains(&self, frame: Frame) -> bool {
        self.range.contains_frame(frame)
    }

    pub fn alloc(&mut self) -> Option<Frame> {
        if self.index < self.range.num_frames() {
            let frame = self.range.start().add(self.index);
            self.index += 1;
            Some(frame)
        } else {
            None
        }
    }

    pub fn dealloc(&mut self, _frame: Frame) {
        // we simply don't deallocate
    }
}
