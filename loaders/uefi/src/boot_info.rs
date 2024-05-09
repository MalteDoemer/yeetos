use alloc::vec::Vec;
use boot_info::platform_info::uefi::UefiInfo;
use boot_info::platform_info::PlatformInfo;
use boot_info::{BootInfoHeader, BOOT_INFO_STRUCT_V1};
use core::mem::MaybeUninit;
use initrd::Initrd;
use kernel_graphics::FrameBufferInfo;
use kernel_image::KernelImageInfo;
use memory::virt::VirtAddr;
use memory::{MemoryMap, MemoryMapEntry, PAGE_SIZE};
use uefi::table::boot::{AllocateType, BootServices, MemoryType};
use uefi::table::{Runtime, SystemTable};

use crate::mmap::MEMORY_TYPE_BOOT_INFO;

pub fn allocate_boot_info(
    boot_services: &BootServices,
    initrd_num_pages: usize,
) -> (&'static mut MaybeUninit<BootInfoHeader>, &'static mut [u8]) {
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

    let header_ptr = base_addr as *mut MaybeUninit<BootInfoHeader>;
    let buffer_ptr = (base_addr + header_size) as *mut u8;

    let buffer =
        unsafe { core::slice::from_raw_parts_mut(buffer_ptr, initrd_num_pages * PAGE_SIZE) };

    let header = unsafe { &mut *header_ptr };

    (header, buffer)
}

pub fn init_boot_info(
    system_table: &SystemTable<Runtime>,
    uninit_boot_info: &mut MaybeUninit<BootInfoHeader>,
    map: &Vec<MemoryMapEntry>,
    initrd: &Initrd,
    kernel_image_info: &KernelImageInfo,
) {
    let mut boot_info = BootInfoHeader::empty();

    let boot_info_start =
        VirtAddr::new(uninit_boot_info as *const MaybeUninit<BootInfoHeader> as usize);

    let boot_info_end = initrd.end_addr();

    boot_info.boot_info_addr = boot_info_start;
    boot_info.boot_info_size = boot_info_end - boot_info_start;
    boot_info.boot_info_version = BOOT_INFO_STRUCT_V1;

    boot_info.kernel_image_info = kernel_image_info.to_higher_half();
    boot_info.frame_buffer_info = FrameBufferInfo::empty();
    // panic!("here");
    boot_info.platform_info = get_platform_info(system_table);

    boot_info.memory_map = MemoryMap::from_slice(map);

    boot_logger::get(|log| {
        boot_info.boot_logger = *log;
    });

    boot_info.initrd_addr = initrd.start_addr().to_higher_half();
    boot_info.initrd_size = initrd.size();

    uninit_boot_info.write(boot_info);
}

fn get_platform_info(system_table: &SystemTable<Runtime>) -> PlatformInfo {
    let addr: usize = system_table
        .get_current_system_table_addr()
        .try_into()
        .unwrap();

    let info = UefiInfo {
        system_table_address: VirtAddr::new(addr),
    };

    PlatformInfo::UEFI(info)
}
