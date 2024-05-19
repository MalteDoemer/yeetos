use crate::arch;
use crate::mm::{frame_global_allocator, virtual_global_allocator};
use alloc::vec::Vec;
use boot_info::BootInfoHeader;
use core::iter::once;
use kernel_image::KernelImageInfo;
use memory::phys::{Frame, PhysicalRange};
use memory::virt::{Page, VirtualRange};
use memory::{AccessFlags, MemoryMap, MemoryMapEntry, MemoryMapEntryKind};
use spin::Once;

static INIT: Once<()> = Once::new();

/// This struct represents a region of virtual memory with corresponding physical memory
/// that needs to be mapped during paging::init(). This includes regions like the kernel stack,
/// code, data and many others.
#[derive(Copy, Clone)]
pub struct InitialKernelRegion {
    pub virt_range: VirtualRange,
    pub phys_range: PhysicalRange,
    pub access_flags: AccessFlags,
}

#[derive(Debug)]
pub enum InitPagingError {
    OutOfMemory,
    InvalidTableLayout,
    UnableToReadPageTable,
    UnableToMapKernelImage,
    UnableToMapEntry(MemoryMapEntryKind),
}

pub fn init(boot_info: &BootInfoHeader) {
    INIT.call_once(|| {
        frame_global_allocator::init(boot_info);

        virtual_global_allocator::init(boot_info);
    });

    arch::paging::init(boot_info);
}

pub fn get_initial_kernel_regions(
    map: &MemoryMap,
    kernel_image_info: &KernelImageInfo,
) -> Result<Vec<InitialKernelRegion>, InitPagingError> {
    let mut regions = get_kernel_image_regions(kernel_image_info)?;
    let entries_to_map = get_entries_to_map(map);
    for entry in entries_to_map {
        let region = translate_memory_map_entry(entry)?;
        regions.push(region);
    }

    Ok(regions)
}

/// Translates a `MemoryMapEntry` to an `InitialKernelRegion`
fn translate_memory_map_entry(
    entry: &MemoryMapEntry,
) -> Result<InitialKernelRegion, InitPagingError> {
    assert!(
        entry.is_frame_aligned(),
        "the entry {:?} is not page-aligned",
        entry.kind()
    );

    let phys_range = entry.range_enclose();
    let virt_range =
        translate_phys_range(phys_range).ok_or(InitPagingError::UnableToMapEntry(entry.kind()))?;

    let access_flags = translate_access_flags(entry.kind());

    Ok(InitialKernelRegion {
        virt_range,
        phys_range,
        access_flags,
    })
}

/// This function returns an iterator over memory regions that need to be mapped immediately.
///
/// Note that this function does not report an entry of kind `MemoryMapEntryKind::KernelImage` to
/// need a mapping as the kernel image is handled as a special case.
fn get_entries_to_map<'a>(map: &'a MemoryMap) -> impl Iterator<Item = &'a MemoryMapEntry> + 'a {
    map.entries().filter(|entry| match entry.kind() {
        MemoryMapEntryKind::BootInfo => true,
        MemoryMapEntryKind::RuntimeServiceCode => true,
        MemoryMapEntryKind::RuntimeServiceData => true,
        MemoryMapEntryKind::Usable => false,
        MemoryMapEntryKind::Reserved => false,
        MemoryMapEntryKind::Defective => false,
        MemoryMapEntryKind::Loader => false,
        MemoryMapEntryKind::KernelImage => false,
    })
}

fn get_kernel_image_regions(
    kernel_image: &KernelImageInfo,
) -> Result<Vec<InitialKernelRegion>, InitPagingError> {
    let stack = translate_kernel_image_region(kernel_image.stack, AccessFlags::READ_WRITE)?;
    let rodata = translate_optional_image_region(kernel_image.rodata, AccessFlags::READ)?;
    let code = translate_kernel_image_region(kernel_image.code, AccessFlags::READ_EXEC)?;
    let relro = translate_optional_image_region(kernel_image.relro, AccessFlags::READ)?;
    let data = translate_optional_image_region(kernel_image.data, AccessFlags::READ_WRITE)?;
    let heap = translate_kernel_image_region(kernel_image.heap, AccessFlags::READ_WRITE)?;

    let res = once(stack)
        .chain(rodata)
        .chain(once(code))
        .chain(relro)
        .chain(data)
        .chain(once(heap))
        .collect();

    Ok(res)
}

fn translate_kernel_image_region(
    region: VirtualRange,
    access_flags: AccessFlags,
) -> Result<InitialKernelRegion, InitPagingError> {
    let phys_range = translate_virt_range(region).ok_or(InitPagingError::UnableToMapKernelImage)?;
    Ok(InitialKernelRegion {
        virt_range: region,
        phys_range,
        access_flags,
    })
}

fn translate_optional_image_region(
    region: Option<VirtualRange>,
    access_flags: AccessFlags,
) -> Result<Option<InitialKernelRegion>, InitPagingError> {
    if let Some(region) = region {
        Ok(Some(translate_kernel_image_region(region, access_flags)?))
    } else {
        Ok(None)
    }
}

fn translate_access_flags(kind: MemoryMapEntryKind) -> AccessFlags {
    match kind {
        MemoryMapEntryKind::RuntimeServiceCode => AccessFlags::READ_EXEC,
        MemoryMapEntryKind::RuntimeServiceData => AccessFlags::READ_WRITE,
        MemoryMapEntryKind::Loader => AccessFlags::READ,
        MemoryMapEntryKind::BootInfo => AccessFlags::READ,
        _ => AccessFlags::empty(),
    }
}

fn translate_phys_range(physical_range: PhysicalRange) -> Option<VirtualRange> {
    let start = physical_range.start_addr().to_higher_half_checked()?;
    let end = physical_range.end_addr().to_higher_half_checked()?;
    Some(VirtualRange::new(Page::new(start), Page::new(end)))
}

fn translate_virt_range(virtual_range: VirtualRange) -> Option<PhysicalRange> {
    let start = virtual_range
        .start_addr()
        .to_lower_half_checked()?
        .to_phys_checked()?;

    let end = virtual_range
        .end_addr()
        .to_lower_half_checked()?
        .to_phys_checked()?;

    Some(PhysicalRange::new(Frame::new(start), Frame::new(end)))
}
