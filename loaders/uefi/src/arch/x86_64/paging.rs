use memory::{
    paging::{Entry, EntryFlags, Level2, Level3, Level4, Table, TableLevel},
    phys::{Frame, PhysAddr, PhysicalRange},
    virt::{Page, VirtAddr},
    AccessFlags, KERNEL_BASE,
};
use spin::Once;
use uefi::table::boot::{AllocateType, BootServices, MemoryType};
use x86::controlregs::cr3_write;
use zeroize::Zeroize;

use crate::mmap::MEMORY_TYPE_KERNEL_PAGE_TABLES;

static PAGE_TABLES_MEMORY: Once<PhysicalRange> = Once::new();

pub fn prepare(boot_services: &BootServices) {
    // this function should only be called once and never concurrently
    assert!(PAGE_TABLES_MEMORY.get().is_none());

    // We need to allocate memory for six page-tables:
    // The PLM4T, one PDPT and 4 PD's
    let num_frames = 6;

    let frames = boot_services.allocate_pages(
        AllocateType::AnyPages,
        MemoryType::custom(MEMORY_TYPE_KERNEL_PAGE_TABLES),
        num_frames as usize,
    );

    let start_addr = PhysAddr::new(frames.expect("unable to allocate physical pages"));
    let memory = PhysicalRange::new(Frame::new(start_addr), num_frames);

    let start_addr = start_addr.to_virt();

    let (pml4t, pdpt, pds) = unsafe { get_tables_mut(start_addr) };

    // clear out the memory to all zero's
    pml4t.zeroize();
    pdpt.zeroize();
    pds[0].zeroize();
    pds[1].zeroize();
    pds[2].zeroize();
    pds[3].zeroize();

    // the first PML4T entry points to the PDPT
    pml4t[0] = Entry::table_entry(table_addr(pdpt));

    // enable higher half mapping
    let pml4t_high_index = (KERNEL_BASE >> 39) & 0x1FF;
    pml4t[pml4t_high_index] = pml4t[0];

    // the last PML4T entry points to itself, this enables "recursive mapping"
    let pml4t_addr = table_addr(pml4t);
    pml4t[511] = Entry::table_entry(pml4t_addr);

    // write the PDPT entries to point to the PD
    for i in 0..4 {
        pdpt[i] = Entry::table_entry(table_addr(pds[i]));
    }

    // fill out the PD's with 2 MiB identity mapping pages
    let mut addr = 0x00usize;
    while addr < 0x100000000 {
        let vaddr = VirtAddr::new(addr);
        let page = Page::new(vaddr);

        let (plm4t_idx, pdpt_idx, pd_index, pt_idx) = Table::<Level4>::get_table_indices(page);

        debug_assert!(plm4t_idx == 0 && pt_idx == 0);

        let ref mut entry = pds[pdpt_idx][pd_index];

        *entry = Entry::page_entry(vaddr.to_phys(), AccessFlags::all());

        // Make this page a Huge Page
        let flags = entry.flags().union(EntryFlags::PAGE_SIZE);
        entry.set_flags(flags);

        addr += 0x200000; // 2 MiB
    }

    PAGE_TABLES_MEMORY.call_once(|| memory);
}

pub fn get_kernel_page_tables_range() -> PhysicalRange {
    *PAGE_TABLES_MEMORY
        .get()
        .expect("paging::get_kernel_page_tables_range() called before paging::prepare()")
}

pub fn activate() {
    let range = get_kernel_page_tables_range();
    let addr = range.start_addr();
    unsafe {
        cr3_write(addr.to_inner());
    }
}

fn table_addr<L: TableLevel>(table: &Table<L>) -> PhysAddr {
    let ptr = table as *const Table<L>;
    let addr = VirtAddr::new(ptr as usize);
    addr.to_phys()
}

unsafe fn get_tables_mut(
    start_addr: VirtAddr,
) -> (
    &'static mut Table<Level4>,
    &'static mut Table<Level3>,
    [&'static mut Table<Level2>; 4],
) {
    unsafe {
        let plm4t_ptr = start_addr.as_ptr_mut::<Table<Level4>>();
        let pdpt_ptr = start_addr.as_ptr_mut::<Table<Level3>>().add(1);
        let pd1_ptr = start_addr.as_ptr_mut::<Table<Level2>>().add(2);
        let pd2_ptr = start_addr.as_ptr_mut::<Table<Level2>>().add(3);
        let pd3_ptr = start_addr.as_ptr_mut::<Table<Level2>>().add(4);
        let pd4_ptr = start_addr.as_ptr_mut::<Table<Level2>>().add(5);

        let pds = [&mut *pd1_ptr, &mut *pd2_ptr, &mut *pd3_ptr, &mut *pd4_ptr];

        (&mut *plm4t_ptr, &mut *pdpt_ptr, pds)
    }
}
