use alloc::vec::Vec;
use memory::{MemoryMapEntry, PhysAddr};

use crate::multiboot2::Multiboot2Info;

pub fn create_memory_map(
    _mboot: &Multiboot2Info,
    _initrd_end_addr: PhysAddr,
    _kernel_end_addr: PhysAddr,
) -> Vec<MemoryMapEntry> {
    todo!()
}
