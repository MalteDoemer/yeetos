use crate::phys::{Frame, Inner, PhysAddr, PhysicalRange};
use arrayvec::ArrayVec;

/// Maximum number of memory map entries
pub const MEMORY_MAP_ENTRIES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMapEntryKind {
    /// Memory is free to use
    Usable,
    /// Memory is used by the firmware / hardware mapped
    Reserved,
    /// Memory cannot be used (defective)
    Defective,
    /// Code from the runtime environment (UEFI)
    RuntimeServiceCode,
    /// Code data the runtime environment (UEFI)
    RuntimeServiceData,
    /// Memory is used by the kernel loader (can be reused)
    Loader,
    /// Memory is used for the boot info structure (can be reused)
    BootInfo,
    /// Memory is used by the kernel image
    KernelImage,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    /// Start address of the region
    start: PhysAddr,
    /// End address of the region
    end: PhysAddr,
    /// Indicates the type of the memory in this entry
    kind: MemoryMapEntryKind,
}

impl MemoryMapEntry {
    pub fn new(start: PhysAddr, end: PhysAddr, kind: MemoryMapEntryKind) -> Self {
        Self { start, end, kind }
    }

    pub fn start(&self) -> PhysAddr {
        self.start
    }
    pub fn end(&self) -> PhysAddr {
        self.end
    }
    pub fn kind(&self) -> MemoryMapEntryKind {
        self.kind
    }

    pub fn size(&self) -> Inner {
        self.end - self.start
    }

    pub fn is_frame_aligned(&self) -> bool {
        self.start.is_frame_aligned() && self.end.is_frame_aligned()
    }

    pub fn range_truncate(&self) -> PhysicalRange {
        let start = self.start.frame_align_up();
        let end = self.end.frame_align_down();
        PhysicalRange::new(Frame::new(start), Frame::new(end))
    }

    pub fn range_enclose(&self) -> PhysicalRange {
        let start = self.start.frame_align_down();
        let end = self.end.frame_align_up();
        PhysicalRange::new(Frame::new(start), Frame::new(end))
    }
}

#[derive(Debug)]
pub struct MemoryMap {
    entries: ArrayVec<MemoryMapEntry, MEMORY_MAP_ENTRIES>,
}

impl MemoryMap {
    pub const fn new(vec: ArrayVec<MemoryMapEntry, MEMORY_MAP_ENTRIES>) -> Self {
        Self { entries: vec }
    }

    pub fn from_slice(map: &[MemoryMapEntry]) -> Self {
        assert!(map.len() <= MEMORY_MAP_ENTRIES);
        let iter = map.iter().map(|e| *e);
        let vec = ArrayVec::from_iter(iter);
        Self::new(vec)
    }

    pub fn entries(&self) -> impl Iterator<Item = &MemoryMapEntry> {
        self.entries.iter()
    }

    pub fn first(&self) -> &MemoryMapEntry {
        self.entries().nth(0).expect("memory map is empty")
    }

    pub fn last(&self) -> &MemoryMapEntry {
        let num_entries = self.entries().count();

        self.entries()
            .nth(num_entries - 1)
            .expect("memory map is empty")
    }

    pub fn start_addr(&self) -> PhysAddr {
        self.first().start
    }

    pub fn end_addr(&self) -> PhysAddr {
        self.last().end
    }

    pub fn is_covered(&self, range: PhysicalRange) -> bool {
        let start = self.start_addr();
        let end = self.end_addr();

        start <= range.start().to_addr() && end >= range.end().to_addr()
    }

    pub fn entries_for_range(
        &self,
        range: PhysicalRange,
    ) -> Option<impl Iterator<Item = &MemoryMapEntry>> {
        if self.is_covered(range) {
            let iter = self
                .entries()
                .skip_while(move |entry| entry.end <= range.start().to_addr())
                .take_while(move |entry| entry.start < range.end().to_addr());
            Some(iter)
        } else {
            None
        }
    }

    pub fn is_usable(&self, range: PhysicalRange) -> bool {
        if let Some(mut iter) = self.entries_for_range(range) {
            iter.all(|entry| entry.kind == MemoryMapEntryKind::Usable)
        } else {
            false
        }
    }
}
