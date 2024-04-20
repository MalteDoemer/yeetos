use alloc::vec::Vec;
use memory::{phys::PhysAddr, MemoryMapEntry, MemoryMapEntryKind};

use crate::multiboot2::{MemoryRegion, Multiboot2Info};

#[cfg(target_arch = "x86")]
mod arch {
    use alloc::vec::Vec;
    use memory::{MemoryMapEntry, MemoryMapEntryKind};

    use crate::multiboot2;

    pub type AddrType = u32;

    pub fn translate_memory_regions(
        mem_regions: &[multiboot2::MemoryRegion],
    ) -> Vec<MemoryMapEntry> {
        // since we only support 32-bit physical addresses (no PAE)
        // we have to ignore all memory above 0xFFFFFFFF

        let mut memory_map = Vec::new();
        for region in mem_regions {
            if region.base_addr <= core::u32::MAX as u64 {
                memory_map.push(convert_region_to_entry(*region));
            } else {
                // ignored
            }
        }

        memory_map
    }

    fn convert_region_to_entry(region: multiboot2::MemoryRegion) -> MemoryMapEntry {
        let kind = match region.region_type {
            1 => MemoryMapEntryKind::Free,
            3 => MemoryMapEntryKind::Reserved, // Usable ACPI Information
            4 => MemoryMapEntryKind::Reserved, // Reserved but preserve on hibernation
            5 => MemoryMapEntryKind::Unusable,
            _ => MemoryMapEntryKind::Reserved,
        };

        if region.base_addr + region.length <= core::u32::MAX as u64 {
            let start = (region.base_addr as u32).into();
            let end = start + region.length as u32;

            MemoryMapEntry::new(start, end, kind)
        } else {
            // now we have a memory map entry that starts below u32::MAX
            // and ends above u32::MAX so we have to split it at u32::MAX

            let start = (region.base_addr as u32).into();
            let end = u32::MAX.into();

            MemoryMapEntry::new(start, end, kind)
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod arch {
    use alloc::vec::Vec;

    use crate::multiboot2;
    use memory::{MemoryMapEntry, MemoryMapEntryKind};

    pub type AddrType = u64;

    pub fn translate_memory_regions(
        mem_regions: &[multiboot2::MemoryRegion],
    ) -> Vec<MemoryMapEntry> {
        let mut memory_map = Vec::new();
        for region in mem_regions {
            memory_map.push(convert_region_to_entry(*region));
        }

        memory_map
    }

    fn convert_region_to_entry(region: multiboot2::MemoryRegion) -> MemoryMapEntry {
        let kind = match region.region_type {
            1 => MemoryMapEntryKind::Free,
            3 => MemoryMapEntryKind::Reserved, // Usable ACPI Information
            4 => MemoryMapEntryKind::Reserved, // Reserved but preserve on hibernation
            5 => MemoryMapEntryKind::Unusable,
            _ => MemoryMapEntryKind::Reserved,
        };

        let start = region.base_addr.into();
        let end = start + region.length;

        MemoryMapEntry::new(start, end, kind)
    }
}

pub struct MemoryMapAddresses {
    initrd_end_addr: PhysAddr,
    kernel_end_addr: PhysAddr,
}

pub fn create_memory_map(
    mboot: &Multiboot2Info,
    initrd_end_addr: PhysAddr,
    kernel_end_addr: PhysAddr,
) -> Vec<MemoryMapEntry> {
    let page_tables = get_page_tables_entry();
    let loader = get_loader_entry();
    let boot_info = get_boot_info_entry(initrd_end_addr);
    let kernel_image = get_kernel_image_entry(initrd_end_addr, kernel_end_addr);

    verify_hardcoded_mmap_entries(page_tables, loader, boot_info, kernel_image);

    let hardcoded_entries = [page_tables, loader, boot_info, kernel_image];

    verify_memory_regions(&mboot.memory_regions);
    let mut memory_map = arch::translate_memory_regions(&mboot.memory_regions);

    for entry in hardcoded_entries {
        let start_idx = find_mmap_entry_containing(entry.start, &memory_map)
            .expect("hardcoded memory map entry is not covered by any memory region");

        let end_idx = find_mmap_entry_containing(entry.end, &memory_map)
            .expect("hardcoded memory map entry is not covered by any memory region");

        if start_idx != end_idx {
            panic!("hardcoded memory map entry spans over multiple memory regions");
        }

        match memory_map[start_idx].kind {
            MemoryMapEntryKind::Free => {}
            _ => panic!("hardcoded memory map entry is not in a usable memory region"),
        }

        let (pre, entry, post) = split_mmap_entry(memory_map[start_idx], entry);

        memory_map.remove(start_idx);

        if post.size() != 0 {
            memory_map.insert(start_idx, post);
        }

        if entry.size() != 0 {
            memory_map.insert(start_idx, entry);
        }

        if pre.size() != 0 {
            memory_map.insert(start_idx, pre);
        }
    }

    memory_map
}

fn split_mmap_entry(
    big: MemoryMapEntry,
    small: MemoryMapEntry,
) -> (MemoryMapEntry, MemoryMapEntry, MemoryMapEntry) {
    debug_assert!(big.start <= small.start && big.end >= small.end);

    // Split region into three parts:
    // 1. big.start..small.start
    // 2. small.start..small.end
    // 3. small.end..big.end

    let pre = MemoryMapEntry::new(big.start, small.start, big.kind);
    let post = MemoryMapEntry::new(small.end, big.end, big.kind);

    (pre, small, post)
}

fn find_mmap_entry_containing(addr: PhysAddr, entries: &[MemoryMapEntry]) -> Option<usize> {
    // for now just a linear search since the mem_regions vector is probably quite small

    for (i, entry) in entries.iter().enumerate() {
        if addr >= entry.start && addr < entry.end {
            return Some(i);
        }
    }

    None
}

/// This function checks that implictly assumed properties
/// of the hardcoded memory map entries are true.
///
/// These include:
/// - The order is preserved: page_tables <= loader <= boot_info <= kernel_image
/// - There is no overlap
fn verify_hardcoded_mmap_entries(
    page_tables: MemoryMapEntry,
    loader: MemoryMapEntry,
    boot_info: MemoryMapEntry,
    kernel_image: MemoryMapEntry,
) {
    if page_tables.start > page_tables.end
        || loader.start > loader.end
        || boot_info.start > boot_info.end
        || kernel_image.start > kernel_image.end
    {
        panic!("some of the hardcoded memory map entries have negative size");
    }

    if page_tables.end > loader.start
        || loader.end > boot_info.start
        || boot_info.end > kernel_image.start
    {
        panic!("the order of the hardcoded memory map entries is not preserved");
    }
}

/// This function performs a few sanity checks on the memory regions
/// provided by the multiboot2 loader.
///
/// This includes:
/// - There is at least one memory region
/// - Memory regions are in ascending order
fn verify_memory_regions(mem_regions: &[MemoryRegion]) {
    // TODO: maybe propagate errors to rust_entry() instead of
    // panicking directly here?

    if mem_regions.is_empty() {
        panic!("no memory regions from multiboot2");
    }

    let mut prev_addr = 0;
    let mut prev_size = 0;

    for region in mem_regions {
        let base_addr = region.base_addr;

        if prev_addr + prev_size > base_addr {
            panic!("memory regions from multiboot2 are not in ascending order or overlapping");
        }

        prev_addr = base_addr;
        prev_size = region.length;
    }
}

fn get_page_tables_entry() -> MemoryMapEntry {
    // defined in boot.s
    let start_addr = 0;
    let end_addr = 0x7000;

    MemoryMapEntry {
        start: PhysAddr::new(start_addr),
        end: PhysAddr::new(end_addr),
        kind: MemoryMapEntryKind::KernelPageTables,
    }
}

fn get_loader_entry() -> MemoryMapEntry {
    // symbols defined in linkers/x86_64.ld
    extern "C" {
        pub fn __load_start();
        pub fn __boot_info_start(); // boot_info_start is the end of the loader
    }

    // Note:
    // physical address = virtual address
    let loader_start = __load_start as arch::AddrType;
    let loader_end = __boot_info_start as arch::AddrType;

    MemoryMapEntry {
        start: PhysAddr::new(loader_start),
        end: PhysAddr::new(loader_end),
        kind: MemoryMapEntryKind::KernelLoader,
    }
}

fn get_boot_info_entry(initrd_end_addr: PhysAddr) -> MemoryMapEntry {
    // symbols defined in linkers/x86_64.ld
    extern "C" {
        pub fn __boot_info_start();
    }

    // Note:
    // physical address = virtual address
    let boot_info_start = __boot_info_start as arch::AddrType;

    MemoryMapEntry {
        start: PhysAddr::new(boot_info_start),
        end: initrd_end_addr,
        kind: MemoryMapEntryKind::KernelBootInfo,
    }
}

fn get_kernel_image_entry(initrd_end_addr: PhysAddr, kernel_end_addr: PhysAddr) -> MemoryMapEntry {
    MemoryMapEntry {
        start: initrd_end_addr,
        end: kernel_end_addr,
        kind: MemoryMapEntryKind::KernelImage,
    }
}
