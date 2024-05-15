use crate::mm::frame_bump_allocator::FrameBumpAllocator;
use alloc::vec::Vec;
use boot_info::BootInfoHeader;
use memory::phys::{Frame, PageFrameAllocator};
use memory::MemoryMapEntryKind;
use spin::Mutex;

static GLOBAL_ALLOC: AllocatorImpl = AllocatorImpl::new();

struct AllocatorImpl {
    allocators: Mutex<Vec<FrameBumpAllocator>>,
}

impl AllocatorImpl {
    pub const fn new() -> Self {
        AllocatorImpl {
            allocators: Mutex::new(Vec::new()),
        }
    }

    pub fn init(&self, boot_info: &BootInfoHeader) {
        let mut guard = self.allocators.lock();

        if guard.is_empty() {
            let vec: Vec<FrameBumpAllocator> = boot_info
                .memory_map
                .entries()
                .filter(|entry| entry.kind() == MemoryMapEntryKind::Usable)
                .map(|entry| FrameBumpAllocator::new(entry.range_truncate()))
                .collect();

            *guard = vec;
        } else {
            panic!("GLOBAL_ALLOC.init() called more than once!");
        }
    }

    pub fn alloc(&self) -> Option<Frame> {
        let mut guard = self.allocators.lock();

        debug_assert!(
            !guard.is_empty(),
            "GlobalFrameAllocator.alloc() called before it was initialized"
        );

        for alloc in guard.iter_mut() {
            let frame = alloc.alloc();

            if let Some(frame) = frame {
                return Some(frame);
            }
        }

        return None;
    }

    pub fn dealloc(&self, frame: Frame) -> Option<()> {
        let mut guard = self.allocators.lock();

        debug_assert!(
            !guard.is_empty(),
            "GlobalFrameAllocator.dealloc() called before it was initialized"
        );

        for alloc in guard.iter_mut() {
            if alloc.contains(frame) {
                return alloc.dealloc(frame);
            }
        }

        return None;
    }

    pub fn contains(&self, frame: Frame) -> bool {
        let guard = self.allocators.lock();

        for alloc in guard.iter() {
            if alloc.contains(frame) {
                return true;
            }
        }

        return false;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct GlobalFrameAllocator;

impl PageFrameAllocator for GlobalFrameAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        GLOBAL_ALLOC.alloc()
    }

    fn dealloc(&mut self, frame: Frame) -> Option<()> {
        GLOBAL_ALLOC.dealloc(frame)
    }

    fn contains(&self, frame: Frame) -> bool {
        GLOBAL_ALLOC.contains(frame)
    }
}

pub(super) fn init(boot_info: &BootInfoHeader) {
    GLOBAL_ALLOC.init(boot_info);
}
