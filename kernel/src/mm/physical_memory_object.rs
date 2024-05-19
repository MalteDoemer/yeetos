use crate::mm::GlobalFrameAllocator;
use alloc::boxed::Box;
use alloc::sync::Arc;
use memory::phys::{Frame, PageFrameAllocator};

pub enum PhysicalMemoryObject<A: PageFrameAllocator + Clone = GlobalFrameAllocator> {
    Shared(Arc<SharedPhysicalMemoryObject<A>>),
    Anon(AnonymousPhysicalMemoryObject<A>),
}

impl PhysicalMemoryObject<GlobalFrameAllocator> {
    pub fn new_shared(num_frames: usize) -> Option<Self> {
        let arc = SharedPhysicalMemoryObject::new(num_frames)?;
        Some(Self::Shared(arc))
    }

    pub fn new_anon() -> Self {
        Self::Anon(AnonymousPhysicalMemoryObject::new())
    }
}

impl<A: PageFrameAllocator + Clone> PhysicalMemoryObject<A> {
    pub fn new_shared_in(num_frames: usize, alloc: A) -> Option<Self> {
        let arc = SharedPhysicalMemoryObject::new_in(num_frames, alloc)?;
        Some(Self::Shared(arc))
    }

    pub fn new_anon_in(alloc: A) -> Self {
        Self::Anon(AnonymousPhysicalMemoryObject::new_in(alloc))
    }
}

pub struct SharedPhysicalMemoryObject<A: PageFrameAllocator + Clone = GlobalFrameAllocator> {
    frames: Box<[Frame]>,
    alloc: A,
}

impl SharedPhysicalMemoryObject<GlobalFrameAllocator> {
    pub fn new(num_frames: usize) -> Option<Arc<SharedPhysicalMemoryObject<GlobalFrameAllocator>>> {
        Self::new_in(num_frames, GlobalFrameAllocator)
    }
}

impl<A: PageFrameAllocator + Clone> SharedPhysicalMemoryObject<A> {
    pub fn new_in(num_frames: usize, mut alloc: A) -> Option<Arc<SharedPhysicalMemoryObject<A>>> {
        let frames = alloc.alloc_multiple(num_frames)?;
        let pmo = SharedPhysicalMemoryObject { frames, alloc };
        Arc::try_new(pmo).ok()
    }

    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }

    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }
}

impl<A: PageFrameAllocator + Clone> Drop for SharedPhysicalMemoryObject<A> {
    fn drop(&mut self) {
        let mut alloc = self.alloc.clone();
        for frame in self.frames() {
            alloc.dealloc(*frame);
        }
    }
}

pub struct AnonymousPhysicalMemoryObject<A: PageFrameAllocator + Clone = GlobalFrameAllocator> {
    alloc: A,
}

impl AnonymousPhysicalMemoryObject<GlobalFrameAllocator> {
    pub fn new() -> Self {
        Self {
            alloc: GlobalFrameAllocator,
        }
    }
}

impl<A: PageFrameAllocator + Clone> AnonymousPhysicalMemoryObject<A> {
    pub fn new_in(alloc: A) -> Self {
        Self { alloc }
    }

    pub fn allocator(&self) -> A {
        self.alloc.clone()
    }
}
