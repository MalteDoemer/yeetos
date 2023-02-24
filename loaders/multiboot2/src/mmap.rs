use memory::{MemoryMap, MemoryMapEntry, MemoryMapEntryKind, PhysAddr};

use crate::multiboot2::{ModuleDescriptor, Multiboot2Info};

pub fn create_memory_map(_mboot: &Multiboot2Info) -> MemoryMap {
    todo!()
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
    // symbols defined in link.ld
    extern "C" {
        pub fn __load_start();
        pub fn __boot_info_start(); // boot_info_start is the end of the loader
    }

    // Note:
    // physical address = virtual address
    let loader_start = __load_start as u64;
    let loader_end = __boot_info_start as u64;

    MemoryMapEntry {
        start: PhysAddr::new(loader_start),
        end: PhysAddr::new(loader_end),
        kind: MemoryMapEntryKind::KernelLoader,
    }
}

fn get_boot_info_entry(initrd_module: ModuleDescriptor) -> MemoryMapEntry {
    // symbols defined in link.ld
    extern "C" {
        pub fn __boot_info_start();
    }

    // Note:
    // physical address = virtual address
    let boot_info_start = __boot_info_start as u64;
    let boot_info_end = initrd_module.mod_end as u64;

    MemoryMapEntry {
        start: PhysAddr::new(boot_info_start),
        end: PhysAddr::new(boot_info_end),
        kind: MemoryMapEntryKind::KernelBootInfo,
    }
}
