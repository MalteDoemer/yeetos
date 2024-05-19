use crate::arch::paging::{
    INITIAL_P4_ADDR, KERNEL_P3_ADDRS, KERNEL_P4_START_IDX, NUM_KERNEL_P3_TABLES,
};
use crate::mm::{
    get_initial_kernel_regions, GlobalFrameAllocator, InitPagingError, InitialKernelRegion,
};
use boot_info::BootInfoHeader;
use memory::{
    paging::{Entry, EntryUsage, HierarchicalLevel, Level3, Level4, Table, TableLevel},
    phys::{Frame, PageFrameAllocator, PhysAddr},
    virt::Page,
    AccessFlags, PAGE_TABLE_ENTRIES,
};
use spin::Once;
use x86::controlregs::cr3_write;
use zeroize::Zeroize;

const KERNEL_P4_RECURSIVE_IDX: usize = PAGE_TABLE_ENTRIES - 1;
static INIT: Once<()> = Once::new();

pub fn init(boot_info: &BootInfoHeader) {
    INIT.call_once(|| init_once(boot_info).expect("unable to initialize paging"));

    init_all();
}

fn init_once(boot_info: &BootInfoHeader) -> Result<(), InitPagingError> {
    init_p4_and_p3s()?;

    let regions = get_initial_kernel_regions(&boot_info.memory_map, &boot_info.kernel_image_info)?;
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
