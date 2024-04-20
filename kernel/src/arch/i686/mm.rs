use crate::mm::MemoryManager;

struct MemoryManagerX86 {}

impl MemoryManager for MemoryManagerX86 {
    fn alloc_frame(&self) -> crate::kresult::KResult<memory::phys::Frame> {
        todo!()
    }

    fn dealloc_frame(&self, _frame: memory::phys::Frame) -> crate::kresult::KResult<()> {
        todo!()
    }
}

pub fn get() -> &'static impl MemoryManager {
    &MemoryManagerX86 {}
}

pub fn init() {}
