use alloc::vec::Vec;
use boot_info::BootInfoHeader;
use memory::phys::{BumpPageFrameAllocator, Frame, PageFrameAllocator};
use memory::MemoryMapEntryKind;
use spin::{Mutex, Once};

use crate::kresult::KError::DeallocError;
use crate::{
    kresult::{KError::OutOfMemory, KResult},
    mm::MemoryManager,
};

static MM: Once<MemoryManagerX86_64> = Once::new();

type PhysicalAllocator = BumpPageFrameAllocator;

struct MemoryManagerX86_64 {
    physical_allocators: Mutex<Vec<PhysicalAllocator>>,
}

impl MemoryManagerX86_64 {
    pub fn new(boot_info: &BootInfoHeader) -> Self {
        let physical_allocators = Self::get_physical_allocators(boot_info);

        Self {
            physical_allocators,
        }
    }

    fn get_physical_allocators(boot_info: &BootInfoHeader) -> Mutex<Vec<PhysicalAllocator>> {
        let vec = boot_info
            .memory_map
            .entries()
            .filter(|entry| entry.kind == MemoryMapEntryKind::Free)
            .map(|entry| BumpPageFrameAllocator::new(entry.range_truncate()))
            .collect();

        Mutex::new(vec)
    }
}

impl MemoryManager for MemoryManagerX86_64 {
    fn alloc_frame(&self) -> KResult<Frame> {
        let mut allocators = self.physical_allocators.lock();

        for alloc in allocators.as_mut_slice() {
            let frame = alloc.alloc();
            if let Some(frame) = frame {
                return Ok(frame);
            }
        }

        Err(OutOfMemory)
    }

    fn dealloc_frame(&self, frame: Frame) -> KResult<()> {
        let mut allocators = self.physical_allocators.lock();

        for alloc in allocators.as_mut_slice() {
            if alloc.contains(frame) {
                return match alloc.dealloc(frame) {
                    None => Err(DeallocError),
                    Some(_) => Ok(()),
                };
            }
        }

        Err(DeallocError)
    }
}

pub fn get() -> &'static impl MemoryManager {
    MM.get().unwrap()
}

pub fn init(boot_info: &BootInfoHeader) {
    MM.call_once(|| MemoryManagerX86_64::new(boot_info));
}
