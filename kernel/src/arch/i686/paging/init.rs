use crate::arch::paging::{
    INITIAL_P2_ADDR, KERNEL_P1_ADDRS, KERNEL_P2_START_IDX, NUM_KERNEL_P1_TABLES,
};
use crate::mm::{
    get_initial_kernel_regions, GlobalFrameAllocator, InitPagingError, InitialKernelRegion,
};
use boot_info::BootInfoHeader;
use memory::{
    paging::{Entry, EntryUsage, Level1, Level2, Table, TableLevel},
    phys::{Frame, PageFrameAllocator, PhysAddr},
    virt::Page,
    AccessFlags, PAGE_TABLE_ENTRIES,
};
use spin::Once;
use x86::controlregs::cr3_write;
use zeroize::Zeroize;

const KERNEL_P2_RECURSIVE_IDX: usize = PAGE_TABLE_ENTRIES - 1;

static INIT: Once<()> = Once::new();

pub fn init(boot_info: &BootInfoHeader) {
    INIT.call_once(|| init_once(boot_info).expect("unable to initialize paging"));

    init_all();
}

fn init_once(boot_info: &BootInfoHeader) -> Result<(), InitPagingError> {
    init_p2_and_p1s()?;

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
    let (p2_idx, p1_idx) = Table::<Level2>::get_table_indices(page);

    unsafe {
        let p2 = get_init_table::<Level2>(INITIAL_P2_ADDR)?;
        let p1 = get_or_create_table(p2, p2_idx)?;

        assert_eq!(p1[p1_idx].usage(), EntryUsage::None);
        p1[p1_idx] = Entry::page_entry(frame.to_addr(), access_flags);
    }

    Ok(())
}

unsafe fn get_or_create_table(
    parent: &mut Table<Level2>,
    idx: usize,
) -> Result<&mut Table<Level1>, InitPagingError> {
    let entry = &mut parent[idx];

    let table = match entry.usage() {
        EntryUsage::None => {
            let (table, addr) = unsafe { alloc_table::<Level1>()? };
            *entry = Entry::table_entry(addr);
            table
        }
        EntryUsage::Table => {
            let addr = PhysAddr::new(entry.addr());
            let table = unsafe { get_init_table::<Level1>(addr)? };
            table
        }
        _ => return Err(InitPagingError::InvalidTableLayout),
    };

    Ok(table)
}

/// This function allocates and initializes the initial PD and the kernel PT's.
/// After this function has finished, INITIAL_P2_ADDR and KERNEL_P1_ADDRS contain valid values.
fn init_p2_and_p1s() -> Result<(), InitPagingError> {
    let (p2, p2_addr) = unsafe { alloc_table::<Level2>()? };

    // set the last entry of the PD to itself in order to enable recursive mapping
    p2[KERNEL_P2_RECURSIVE_IDX] = Entry::table_entry(p2_addr);

    for i in 0..NUM_KERNEL_P1_TABLES {
        let idx = i + KERNEL_P2_START_IDX;

        let (_p1, p1_addr) = unsafe { alloc_table::<Level1>()? };

        unsafe {
            KERNEL_P1_ADDRS[i] = p1_addr;
        }

        p2[idx] = Entry::table_entry(p1_addr);
    }

    unsafe {
        INITIAL_P2_ADDR = p2_addr;
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
    unsafe { cr3_write(INITIAL_P2_ADDR.to_inner() as u64) };
}
