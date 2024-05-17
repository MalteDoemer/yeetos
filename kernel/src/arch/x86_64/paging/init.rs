use crate::arch::paging::{
    INITIAL_P4_ADDR, KERNEL_P3_ADDRS, KERNEL_P4_START_IDX, NUM_KERNEL_P3_TABLES,
};
use crate::mm::GlobalFrameAllocator;
use alloc::vec::Vec;
use boot_info::BootInfoHeader;
use core::iter::once;
use kernel_image::KernelImageInfo;
use memory::{
    paging::{Entry, EntryUsage, HierarchicalLevel, Level3, Level4, Table, TableLevel},
    phys::{Frame, PageFrameAllocator, PhysAddr, PhysicalRange},
    virt::{Page, VirtualRange},
    AccessFlags, MemoryMap, MemoryMapEntry, MemoryMapEntryKind, PAGE_TABLE_ENTRIES,
};
use spin::Once;
use x86::controlregs::cr3_write;
use zeroize::Zeroize;

const KERNEL_P4_RECURSIVE_IDX: usize = PAGE_TABLE_ENTRIES - 1;
static INIT: Once<()> = Once::new();

/// This struct represents a region of virtual memory with corresponding physical memory
/// that needs to be mapped during paging::init(). This includes regions like the kernel stack,
/// code, data and many others.
#[derive(Copy, Clone)]
struct InitialKernelRegion {
    virt_range: VirtualRange,
    phys_range: PhysicalRange,
    access_flags: AccessFlags,
}

#[derive(Debug)]
enum InitPagingError {
    OutOfMemory,
    InvalidTableLayout,
    UnableToReadPageTable,
    UnableToMapKernelImage,
    UnableToMapEntry(MemoryMapEntryKind),
}

pub fn init(boot_info: &BootInfoHeader) {
    INIT.call_once(|| init_once(boot_info).expect("unable to initialize paging"));

    init_all();
}

fn init_once(boot_info: &BootInfoHeader) -> Result<(), InitPagingError> {
    init_p4_and_p3s()?;

    let regions = get_memory_regions(&boot_info.memory_map, &boot_info.kernel_image_info)?;
    for region in regions {
        unsafe {
            map_initial_kernel_region(region)?;
        }
    }

    Ok(())
}

unsafe fn map_initial_kernel_region(region: InitialKernelRegion) -> Result<(), InitPagingError> {
    for (page, frame) in region.virt_range.pages().zip(region.phys_range.frames()) {
        unsafe {
            map_initial_page(page, frame, region.access_flags)?;
        }
    }

    Ok(())
}

unsafe fn map_initial_page(
    page: Page,
    frame: Frame,
    access_flags: AccessFlags,
) -> Result<(), InitPagingError> {
    let (p4_idx, p3_idx, p2_idx, p1_idx) = Table::<Level4>::get_table_indices(page);

    unsafe {
        let p4 = get_init_table::<Level4>(INITIAL_P4_ADDR)?;
        let p3 = get_or_create_table(p4, p4_idx)?;
        let p2 = get_or_create_table(p3, p3_idx)?;
        let p1 = get_or_create_table(p2, p2_idx)?;

        assert_eq!(p1[p1_idx].usage(), EntryUsage::None);
        p1[p1_idx] = Entry::page_entry(frame.to_addr(), access_flags);
    }

    Ok(())
}

unsafe fn get_or_create_table<L: HierarchicalLevel>(
    parent: &mut Table<L>,
    idx: usize,
) -> Result<&mut Table<L::NextLevel>, InitPagingError> {
    let entry = &mut parent[idx];

    let table = match entry.usage() {
        EntryUsage::None => {
            let (table, addr) = unsafe { alloc_table::<L::NextLevel>()? };
            *entry = Entry::table_entry(addr);
            table
        }
        EntryUsage::Table => {
            let addr = PhysAddr::new(entry.addr());
            let table = unsafe { get_init_table::<L::NextLevel>(addr)? };
            table
        }
        _ => return Err(InitPagingError::InvalidTableLayout),
    };

    Ok(table)
}

fn get_memory_regions(
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

/// This function allocates and initializes the initial PLM4T and the kernel PDPT's.
/// After this function has finished, INITIAL_P4_ADDR and KERNEL_P3_ADDRS contain valid values.
fn init_p4_and_p3s() -> Result<(), InitPagingError> {
    let (p4, p4_addr) = unsafe { alloc_table::<Level4>()? };

    // set the last entry of the PML4T to itself in order to enable recursive mapping
    p4[KERNEL_P4_RECURSIVE_IDX] = Entry::table_entry(p4_addr);

    for i in 0..NUM_KERNEL_P3_TABLES {
        let idx = i + KERNEL_P4_START_IDX;

        let (_p3, p3_addr) = unsafe { alloc_table::<Level3>()? };

        unsafe {
            KERNEL_P3_ADDRS[i] = p3_addr;
        }

        p4[idx] = Entry::table_entry(p3_addr);
    }

    unsafe {
        INITIAL_P4_ADDR = p4_addr;
    };

    Ok(())
}

/// This function allocates memory for a new page table and creates a mutable reference to it.
/// This function also calls `Zeroize:::zeroize()` on the newly created table in order to clear all
/// of its entries to zero.
unsafe fn alloc_table<'a, L: TableLevel>() -> Result<(&'a mut Table<L>, PhysAddr), InitPagingError>
{
    let addr = alloc_table_memory()?;
    let table = unsafe { get_init_table::<'a, L>(addr)? };
    table.zeroize();
    Ok((table, addr))
}

/// This function simply allocates a new frame using the global frame allocator.
fn alloc_table_memory() -> Result<PhysAddr, InitPagingError> {
    match GlobalFrameAllocator.alloc() {
        None => Err(InitPagingError::OutOfMemory),
        Some(frame) => Ok(frame.to_addr()),
    }
}

/// This function translates the given physical address to higher half and reinterprets it as a
/// paging table with the given level. This is highly unsafe and assumes that we are still in
/// "higher-half identity mapping" mode.
unsafe fn get_init_table<'a, L: TableLevel>(
    addr: PhysAddr,
) -> Result<&'a mut Table<L>, InitPagingError> {
    let virt_addr = addr
        .to_higher_half_checked()
        .ok_or(InitPagingError::UnableToReadPageTable)?;

    let table = unsafe { &mut *virt_addr.as_ptr_mut::<Table<L>>() };

    Ok(table)
}

fn init_all() {
    unsafe { cr3_write(INITIAL_P4_ADDR.to_inner()) };
}
