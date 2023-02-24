use crate::PhysAddr;

/// Maximum number of memory map entries
pub const MEMORY_MAP_ENTRIES: usize = 64;

#[derive(Debug, Clone, Copy)]
pub enum MemoryMapEntryKind {
    /// This entry is unused
    None,
    /// Memory is free to use
    Free,
    /// Memory is used by the firmeware / hardware mapped
    Reserved,
    /// Memory cannot be used (defective)
    Unusable,

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
        Self {
            start: start.page_align_down(),
            end: end.page_align_up(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    pub entries: [MemoryMapEntry; MEMORY_MAP_ENTRIES],
}

impl MemoryMap {
    pub fn new() -> Self {
        Self {
            entries: [MemoryMapEntry::empty(); MEMORY_MAP_ENTRIES],
        }
    }
}
