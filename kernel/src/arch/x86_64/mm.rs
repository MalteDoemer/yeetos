use memory::phys::Frame;
use spin::Once;

use crate::{
    kresult::{KError::OutOfMemory, KResult},
    mm::MemoryManager,
};

static MM: Once<MemoryManagerX86_64> = Once::new();

struct MemoryManagerX86_64 {}

impl MemoryManagerX86_64 {
    pub fn new() -> Self {
        Self {}
    }
}

impl MemoryManager for MemoryManagerX86_64 {
    fn alloc_frame(&self) -> KResult<Frame> {
        Err(OutOfMemory)
    }

    fn dealloc_frame(&self, _frame: Frame) -> KResult<()> {
        todo!()
    }
}

pub fn get() -> &'static impl MemoryManager {
    MM.get().unwrap()
}

pub fn init() {
    MM.call_once(|| MemoryManagerX86_64::new());
}
