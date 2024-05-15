use crate::mm::GlobalFrameAllocator;
use alloc::vec::Vec;
use boot_info::BootInfoHeader;
use kernel_image::KernelImageInfo;
use log::info;
use memory::paging::{Entry, EntryUsage, Level1, Level2, Level3, Level4, Table, TableLevel};
use memory::phys::{Frame, PageFrameAllocator, PhysAddr, PhysicalRange};
use memory::virt::{Page, VirtualRange};
use memory::{
    AccessFlags, MemoryMap, MemoryMapEntry, MemoryMapEntryKind, KERNEL_BASE, PAGE_TABLE_ENTRIES,
};
use spin::Once;

const P4_KERNEL_START_IDX: usize = (KERNEL_BASE >> 39) & 0x1FF;
const P4_KERNEL_END_IDX: usize = PAGE_TABLE_ENTRIES - 2;
const NUM_KERNEL_P3_TABLES: usize = P4_KERNEL_END_IDX - P4_KERNEL_START_IDX;

static INIT: Once<()> = Once::new();
static mut INITIAL_P4_ADDR: PhysAddr = PhysAddr::zero();

static mut KERNEL_P3_ADDRS: [PhysAddr; NUM_KERNEL_P3_TABLES] =
    [PhysAddr::zero(); NUM_KERNEL_P3_TABLES];

#[derive(Debug)]
struct InitialKernelRegion {
    virt_range: VirtualRange,
    phys_range: PhysicalRange,
    access_flags: AccessFlags,
}

pub fn init(boot_info: &BootInfoHeader) {
    INIT.call_once(|| init_once(boot_info).expect("failed to initialize paging"));

    init_all();
}

fn init_once(boot_info: &BootInfoHeader) -> Option<()> {
    init_p4_and_p3s()?;

    let regions: Vec<InitialKernelRegion> =
        get_memory_regions(&boot_info.memory_map, &boot_info.kernel_image_info).collect();

    for region in &regions {
        info!("Got a region: {:?}", region);
    }

    unsafe {
        map_initial_kernel_region(&regions);
    }

    Some(())
}

fn init_p4_and_p3s() -> Option<()> {
    let p4_addr = alloc_table()?;
    let p4 = unsafe { get_init_table::<Level4>(p4_addr) }?;

    for i in 0..NUM_KERNEL_P3_TABLES {
        let idx = i + P4_KERNEL_START_IDX;

        let p3_addr = alloc_table()?;

        unsafe {
            KERNEL_P3_ADDRS[i] = p3_addr;
        }

        p4[idx] = Entry::table_entry(p3_addr);
    }

    unsafe {
        INITIAL_P4_ADDR = p4_addr;
    };
    Some(())
}

fn alloc_table() -> Option<PhysAddr> {
    Some(GlobalFrameAllocator.alloc()?.to_addr())
}

unsafe fn get_init_table<'a, L: TableLevel>(addr: PhysAddr) -> Option<&'a mut Table<L>> {
    let virt_addr = addr.to_higher_half_checked()?;
    let table = unsafe { &mut *virt_addr.as_ptr_mut::<Table<L>>() };
    Some(table)
}

unsafe fn map_initial_kernel_region(regions: &[InitialKernelRegion]) -> Option<()> {
    for region in regions {
        for (page, frame) in region.virt_range.pages().zip(region.phys_range.frames()) {
            unsafe {
                map_initial_page(page, frame, region.access_flags)?;
            }
        }
    }

    Some(())
}

unsafe fn map_initial_page(page: Page, frame: Frame, access_flags: AccessFlags) -> Option<()> {
    let (p4_idx, p3_idx, p2_idx, p1_idx) = Table::<Level4>::get_table_indices(page);

    let p4 = unsafe { get_init_table::<Level4>(INITIAL_P4_ADDR)? };

    let p3_entry = &mut p4[p4_idx];
    let p3_addr = match p3_entry.usage() {
        EntryUsage::Table => PhysAddr::new(p3_entry.addr()),
        _ => return None,
    };
    let p3 = unsafe { get_init_table::<Level3>(p3_addr)? };

    let p2_entry = &mut p3[p3_idx];
    let p2_addr = match p2_entry.usage() {
        EntryUsage::Table => PhysAddr::new(p2_entry.addr()),
        EntryUsage::None => {
            let addr = GlobalFrameAllocator.alloc()?.to_addr();
            *p2_entry = Entry::table_entry(addr);
            addr
        }
        _ => return None,
    };
    let p2 = unsafe { get_init_table::<Level2>(p2_addr)? };

    let p1_entry = &mut p2[p2_idx];
    let p1_addr = match p1_entry.usage() {
        EntryUsage::Table => PhysAddr::new(p1_entry.addr()),
        EntryUsage::None => {
            let addr = GlobalFrameAllocator.alloc()?.to_addr();
            *p1_entry = Entry::table_entry(addr);
            addr
        }
        _ => return None,
    };
    let p1 = unsafe { get_init_table::<Level1>(p1_addr)? };

    p1[p1_idx] = Entry::page_entry(frame.to_addr(), access_flags);

    Some(())
}

fn get_memory_regions<'a>(
    map: &'a MemoryMap,
    kernel_image_info: &KernelImageInfo,
) -> impl Iterator<Item = InitialKernelRegion> + 'a {
    let map_entries = map.entries.iter().filter(|entry| match entry.kind {
        MemoryMapEntryKind::None => false,
        MemoryMapEntryKind::Free => false,
        MemoryMapEntryKind::Reserved => false,
        MemoryMapEntryKind::Unusable => false,
        MemoryMapEntryKind::RuntimeServiceCode => true,
        MemoryMapEntryKind::RuntimeServiceData => true,
        MemoryMapEntryKind::KernelPageTables => true,
        MemoryMapEntryKind::KernelLoader => true,
        MemoryMapEntryKind::KernelBootInfo => true,
        MemoryMapEntryKind::KernelImage => false, // handled separately
    });

    let kernel_regions = translate_kernel_image(kernel_image_info)
        .expect("paging::init(): failed to map kernel image");

    map_entries
        .map(|entry| translate_entry(entry).expect("paging::init() failed to map kernel regions"))
        .chain(kernel_regions)
}

fn translate_entry(entry: &MemoryMapEntry) -> Option<InitialKernelRegion> {
    
    if !entry.is_frame_aligned() {
        panic!("entry {:?} is not frame aligned", entry.kind);
    }
    
    // assert!(entry.is_frame_aligned());
    let phys = entry.range_enclose();
    let virt = translate_phys_range(phys)?;

    let flags = translate_access(entry.kind);

    Some(InitialKernelRegion {
        virt_range: virt,
        phys_range: phys,
        access_flags: flags,
    })
}

fn translate_kernel_image(
    kernel_image: &KernelImageInfo,
) -> Option<impl Iterator<Item = InitialKernelRegion>> {
    let stack = InitialKernelRegion {
        virt_range: kernel_image.stack,
        phys_range: translate_virt_range(kernel_image.stack)?,
        access_flags: AccessFlags::READ_WRITE,
    };

    let rodata = match kernel_image.rodata {
        None => None,
        Some(rodata) => Some(InitialKernelRegion {
            virt_range: rodata,
            phys_range: translate_virt_range(rodata)?,
            access_flags: AccessFlags::READ,
        }),
    };

    let code = InitialKernelRegion {
        virt_range: kernel_image.code,
        phys_range: translate_virt_range(kernel_image.code)?,
        access_flags: AccessFlags::READ_EXEC,
    };

    let relro = match kernel_image.relro {
        None => None,
        Some(relro) => Some(InitialKernelRegion {
            virt_range: relro,
            phys_range: translate_virt_range(relro)?,
            access_flags: AccessFlags::READ,
        }),
    };

    let data = match kernel_image.data {
        None => None,
        Some(data) => Some(InitialKernelRegion {
            virt_range: data,
            phys_range: translate_virt_range(data)?,
            access_flags: AccessFlags::READ_WRITE,
        }),
    };

    let heap = InitialKernelRegion {
        virt_range: kernel_image.heap,
        phys_range: translate_virt_range(kernel_image.heap)?,
        access_flags: AccessFlags::READ_WRITE,
    };

    let iter = core::iter::once(stack)
        .chain(rodata)
        .chain(core::iter::once(code))
        .chain(relro)
        .chain(data)
        .chain(core::iter::once(heap));

    Some(iter)
}

fn translate_access(kind: MemoryMapEntryKind) -> AccessFlags {
    match kind {
        MemoryMapEntryKind::RuntimeServiceCode => AccessFlags::READ_EXEC,
        MemoryMapEntryKind::RuntimeServiceData => AccessFlags::READ_WRITE,
        MemoryMapEntryKind::KernelPageTables => AccessFlags::READ,
        MemoryMapEntryKind::KernelLoader => AccessFlags::READ,
        MemoryMapEntryKind::KernelBootInfo => AccessFlags::READ,
        _ => AccessFlags::empty(),
    }
}

fn translate_phys_range(range: PhysicalRange) -> Option<VirtualRange> {
    let start = range.start_addr().to_higher_half_checked()?;
    let end = range.checked_end_addr()?.to_higher_half_checked()?;
    Some(VirtualRange::new_diff(Page::new(start), Page::new(end)))
}

fn translate_virt_range(range: VirtualRange) -> Option<PhysicalRange> {
    let start = memory::to_lower_half_virt(range.start_addr())?.to_phys_checked()?;
    let end = memory::to_lower_half_virt(range.checked_end_addr()?)?.to_phys_checked()?;
    Some(PhysicalRange::new_diff(Frame::new(start), Frame::new(end)))
}

fn init_all() {}
