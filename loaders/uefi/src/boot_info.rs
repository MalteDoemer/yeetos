use boot_info::BootInfoHeader;
use memory::PAGE_SIZE;
use uefi::table::boot::{AllocateType, BootServices, MemoryType};

const MEMORY_TYPE_BOOT_INFO: u32 = 0x80000005;

pub fn allocate_boot_info(
    boot_services: &BootServices,
    initrd_num_pages: usize,
) -> (&'static mut BootInfoHeader, &'static mut [u8]) {
    let header_size = core::mem::size_of::<BootInfoHeader>().next_multiple_of(PAGE_SIZE);
    let num_pages = (header_size / PAGE_SIZE) + initrd_num_pages;

    let base_addr: usize = boot_services
        .allocate_pages(
            AllocateType::AnyPages,
            MemoryType::custom(MEMORY_TYPE_BOOT_INFO),
            num_pages,
        )
        .expect("unable to allocate pages for boot info")
        .try_into()
        .unwrap();

    let header_ptr = base_addr as *mut BootInfoHeader;
    let buffer_ptr = (base_addr + header_size) as *mut u8;

    let buffer =
        unsafe { core::slice::from_raw_parts_mut(buffer_ptr, initrd_num_pages * PAGE_SIZE) };

    let header = unsafe { &mut *header_ptr };

    (header, buffer)
}
