use crate::phys::{Inner, PhysAddr, PhysicalRange};

/// Maximum number of memory map entries
pub const MEMORY_MAP_ENTRIES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMapEntryKind {
    /// This entry is unused
    None,
    /// Memory is free to use
    Free,
    /// Memory is used by the firmeware / hardware mapped
    Reserved,
    /// Memory cannot be used (defective)
    Unusable,
    /// Code from the runtime environment (UEFI)
    RuntimeServiceCode,
    /// Code data the runtime environment (UEFI)
    RuntimeServiceData,
    /// Memory is used for the initial kernel page tables (may be reused)
    KernelPageTables,
    /// Memory is used by the kernel loader (may be reused)
    KernelLoader,
    /// Memory is used for the boot info structure (may be reused)
    KernelBootInfo,
    /// Memory is used by the kernel image
    KernelImage,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    /// Start address of the region - page aligned
    pub start: PhysAddr,
    /// End address of the region - page aligned
    pub end: PhysAddr,
    /// Indicates the type of the memory in this entry
    pub kind: MemoryMapEntryKind,
}

impl MemoryMapEntry {
    pub const fn empty() -> Self {
        Self {
            start: PhysAddr::zero(),
            end: PhysAddr::zero(),
            kind: MemoryMapEntryKind::None,
        }
    }

    pub fn new(start: PhysAddr, end: PhysAddr, kind: MemoryMapEntryKind) -> Self {
        Self { start, end, kind }
    }

    pub fn size(&self) -> Inner {
        self.end - self.start
    }

    pub fn is_frame_aligned(&self) -> bool {
        self.start.is_frame_aligned() && self.end.is_frame_aligned()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    pub entries: [MemoryMapEntry; MEMORY_MAP_ENTRIES],
}

impl MemoryMap {
    pub const fn new() -> Self {
        Self {
            entries: [MemoryMapEntry::empty(); MEMORY_MAP_ENTRIES],
        }
    }

    pub fn from_slice(map: &[MemoryMapEntry]) -> Self {
        assert!(map.len() <= MEMORY_MAP_ENTRIES);

        let mut result = MemoryMap::new();

        for (i, entry) in map.iter().enumerate() {
            result.entries[i] = *entry;
        }

        result
    }

    pub fn entries(&self) -> impl Iterator<Item = &MemoryMapEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.kind != MemoryMapEntryKind::None)
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
            iter.all(|entry| entry.kind == MemoryMapEntryKind::Free)
        } else {
            false
        }
    }
}
