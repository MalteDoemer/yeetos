use alloc::vec::Vec;
use memory::phys::PhysAddr;
use memory::{MemoryMapEntry, MemoryMapEntryKind, FRAME_SIZE};
use uefi::table::boot::{MemoryDescriptor, MemoryType};

pub const MEMORY_TYPE_BOOT_INFO: u32 = 0x80000005;
pub const MEMORY_TYPE_KERNEL_IMAGE: u32 = 0x80000006;

#[derive(Debug, Copy, Clone)]
pub enum MemoryMapError {
    MemoryMapEmpty,
    KernelNotMappable,
    RuntimeServicesNotMappable,
}

pub fn create_memory_map(
    uefi_map: &uefi::table::boot::MemoryMap,
) -> Result<Vec<MemoryMapEntry>, MemoryMapError> {
    let mut mmap = Vec::new();

    for entry in uefi_map.entries() {
        let translated = translate(entry);

        let merged = if let Some(last) = mmap.last() {
            if can_merge(last, &translated) {
                Some(merge(last, &translated))
            } else {
                None
            }
        } else {
            None
        };

        match merged {
            Some(merged) => {
                let last_idx = mmap.len() - 1;
                mmap[last_idx] = merged;
            }
            None => mmap.push(translated),
        }
    }

    verify_memory_map(&mmap).map(|_| mmap)
}

fn verify_memory_map(mmap: &Vec<MemoryMapEntry>) -> Result<(), MemoryMapError> {
    if mmap.is_empty() {
        return Err(MemoryMapError::MemoryMapEmpty);
    }

    for entry in mmap.iter() {
        if !is_higher_half(entry) {
            match entry.kind() {
                MemoryMapEntryKind::RuntimeServiceCode => {
                    return Err(MemoryMapError::RuntimeServicesNotMappable)
                }
                MemoryMapEntryKind::RuntimeServiceData => {
                    return Err(MemoryMapError::RuntimeServicesNotMappable)
                }
                MemoryMapEntryKind::Loader => {
                    return Err(MemoryMapError::KernelNotMappable);
                }
                MemoryMapEntryKind::BootInfo => return Err(MemoryMapError::KernelNotMappable),
                MemoryMapEntryKind::KernelImage => {
                    return Err(MemoryMapError::KernelNotMappable);
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn is_higher_half(entry: &MemoryMapEntry) -> bool {
    entry.start().to_higher_half_checked().is_some()
        && entry.end().to_higher_half_checked().is_some()
}

fn translate(uefi_entry: &MemoryDescriptor) -> MemoryMapEntry {
    let kind = map_memory_type(uefi_entry.ty);
    let start = PhysAddr::new(uefi_entry.phys_start.try_into().unwrap());
    let frames: memory::phys::Inner = uefi_entry.page_count.try_into().unwrap();
    let size = frames * FRAME_SIZE;
    MemoryMapEntry::new(start, start + size, kind)
}

fn can_merge(entry1: &MemoryMapEntry, entry2: &MemoryMapEntry) -> bool {
    entry1.end() == entry2.start() && entry1.kind() == entry2.kind()
}

fn merge(entry1: &MemoryMapEntry, entry2: &MemoryMapEntry) -> MemoryMapEntry {
    assert!(can_merge(entry1, entry2));

    MemoryMapEntry::new(entry1.start(), entry2.end(), entry1.kind())
}

fn map_memory_type(uefi_type: MemoryType) -> MemoryMapEntryKind {
    match uefi_type {
        MemoryType::LOADER_CODE => MemoryMapEntryKind::Loader,
        MemoryType::LOADER_DATA => MemoryMapEntryKind::Loader,
        MemoryType::RUNTIME_SERVICES_CODE => MemoryMapEntryKind::RuntimeServiceCode,
        MemoryType::RUNTIME_SERVICES_DATA => MemoryMapEntryKind::RuntimeServiceData,
        MemoryType::BOOT_SERVICES_CODE => MemoryMapEntryKind::Usable,
        MemoryType::BOOT_SERVICES_DATA => MemoryMapEntryKind::Usable,
        MemoryType::CONVENTIONAL => MemoryMapEntryKind::Usable,

        MemoryType::UNUSABLE => MemoryMapEntryKind::Defective,
        MemoryType::RESERVED => MemoryMapEntryKind::Reserved,
        MemoryType::ACPI_RECLAIM => MemoryMapEntryKind::Reserved,
        MemoryType::ACPI_NON_VOLATILE => MemoryMapEntryKind::Reserved,
        MemoryType::MMIO => MemoryMapEntryKind::Reserved,
        MemoryType::MMIO_PORT_SPACE => MemoryMapEntryKind::Reserved,
        MemoryType::PAL_CODE => MemoryMapEntryKind::Reserved,
        MemoryType::PERSISTENT_MEMORY => MemoryMapEntryKind::Reserved,

        MemoryType(custom) => match custom {
            MEMORY_TYPE_BOOT_INFO => MemoryMapEntryKind::BootInfo,
            MEMORY_TYPE_KERNEL_IMAGE => MemoryMapEntryKind::KernelImage,
            _ => MemoryMapEntryKind::Reserved,
        },
    }
}
