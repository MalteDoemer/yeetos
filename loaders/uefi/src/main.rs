#![no_main]
#![no_std]
#![allow(dead_code)]
// needed for the heap allocator
#![feature(alloc_error_handler)]
#![feature(allocator_api)]

mod acpi;
mod arch;
mod boot_info;
mod bootfs;
mod heap;
mod paging;
mod panic_handler;

extern crate alloc;

use alloc::vec::Vec;
use bootfs::BootFs;
use initrd::Initrd;
use kernel_image::ParsedKernelImage;
use log::info;
use memory::{phys::PhysAddr, virt::VirtAddr, PAGE_SIZE};
use uefi::{
    prelude::*,
    table::boot::{AllocateType, MemoryType},
};

pub const MEMORY_TYPE_BOOT_INFO: u32 = 0x80000005;
pub const MEMORY_TYPE_KERNEL_IMAGE: u32 = 0x80000006;
pub const MEMORY_TYPE_KERNEL_PAGE_TABLES: u32 = 0x80000007;

#[entry]
fn main(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    // Initialize logger
    boot_logger::init();

    // Initialize the heap
    heap::init();

    // Initialize time helper functions
    arch::time::init(system_table.boot_services());

    // Parse the ACPI tables
    let acpi_tables = acpi::get_acpi_tables(&system_table);
    let num_cores = acpi::number_of_cores(&acpi_tables);

    info!("there are {} cores reported", num_cores);

    // Open a handle to the EFI System Partition
    let mut bootfs =
        BootFs::new(handle, system_table.boot_services()).expect("unable to open boot file system");

    // Find and open the initrd file
    let (mut initrd_file, initrd_pages) = bootfs.open_initrd().expect("unable to open initrd file");

    // Allocate memory for the boot info and INITRD
    let (_boot_info_header, initrd_buffer) =
        boot_info::allocate_boot_info(system_table.boot_services(), initrd_pages);

    // Load the INITRD into memory
    bootfs
        .load_file(&mut initrd_file, initrd_buffer)
        .expect("unable to load initrd file");

    let initrd = Initrd::new(initrd_buffer).expect("unable to parse initrd");

    // Search for the kernel command line file in the INITRD
    let cmdline_data = initrd
        .file_by_name("cmdline")
        .expect("kernel command line file not found")
        .data_as_str()
        .expect("kernel command line not valid utf-8");

    // Parse the kernel command line file
    let kernel_cmdline = kernel_cmdline::KernelCommandLineParser::new(cmdline_data).parse();

    // Search for the kernel file in the INITRD
    let kernel_file = initrd
        .file_by_name("kernel")
        .expect("kernel file not found");

    // Parse the kernel elf image
    let parsed_kernel_image = ParsedKernelImage::new(
        num_cores,
        kernel_cmdline.stack_size(),
        kernel_cmdline.initial_heap_size(),
        kernel_file.data(),
    )
    .expect("unable to parse kernel elf image");

    // Allocate the physical pages for the kernel image
    let kernel_base = allocate_kernel_pages(
        system_table.boot_services(),
        &parsed_kernel_image,
        kernel_cmdline.use_reloc(),
    )
    .expect("unable to allocate physical pages for the kernel image");

    // Now create a loadable kernel image based on the `use_reloc` command line option.
    let kernel_image = if kernel_cmdline.use_reloc() {
        parsed_kernel_image.to_reloc_image(kernel_base)
    } else {
        parsed_kernel_image.to_fixed_image()
    }
    .expect("unable to parse kernel elf image");

    let kernel_image_info = kernel_image.kernel_image_info();

    paging::prepare(system_table.boot_services());

    info!(
        "kernel image: {:p} - {:p}",
        kernel_image_info.start(),
        kernel_image_info.end()
    );

    info!("exiting boot services");

    // We cannot use the bootfs after exiting boot services
    drop(bootfs);

    // Call exit_boot_services to transition over to full control over the system
    let (system_table, _mmap) = system_table.exit_boot_services(MemoryType::LOADER_DATA);

    // Start all application processors
    acpi::ap_startup::startup_all_application_processors(&acpi_tables, &kernel_image);

    panic!("finished with main()");
}

fn dump_memory_map(boot_services: &BootServices) {
    let mmap_size = boot_services.memory_map_size();

    let buffer_size = mmap_size.map_size + mmap_size.entry_size * 8;

    let mut vec = Vec::<u8>::with_capacity(buffer_size);
    vec.resize(buffer_size, 0);

    let buffer = vec.as_mut_slice();

    let mut mmap = boot_services
        .memory_map(buffer)
        .expect("unable to get uefi memory map");

    mmap.sort();

    let mut i = 0;

    for entry in mmap.entries() {
        let start = PhysAddr::new(entry.phys_start);
        // let end = start + FRAME_SIZE * entry.page_count;

        info!(
            "start={:p} pages={:#x} type={:?}",
            start, entry.page_count, entry.ty
        );
        i += 1;

        if i % 8 == 0 {
            info!("---------------------------------------------");
            boot_services.stall(5_000_000);
        }
    }
}

fn allocate_kernel_pages(
    boot_services: &BootServices,
    parsed_kernel: &ParsedKernelImage,
    use_reloc: bool,
) -> uefi::Result<VirtAddr> {
    let size_in_bytes = parsed_kernel.total_size();

    // parsed_kernel.total_size() should be page aligned
    debug_assert!(size_in_bytes.next_multiple_of(PAGE_SIZE) == size_in_bytes);

    let num_pages = size_in_bytes / PAGE_SIZE;

    if use_reloc {
        // since we are using a relocatable kernel image we can
        // allocate the memory for the kernel anywhere

        let kernel_base: usize = boot_services
            .allocate_pages(
                AllocateType::AnyPages,
                MemoryType::custom(MEMORY_TYPE_KERNEL_IMAGE),
                num_pages,
            )?
            .try_into()
            .expect("physical address of to large");

        Ok(VirtAddr::new(kernel_base))
    } else {
        // since we are using a fixed address kernel image
        // we have to request memory from a specific address

        let kernel_base = parsed_kernel.fixed_load_addr();
        let kernel_base_phys = kernel_base.to_phys();
        let kernel_end_phys = (kernel_base + size_in_bytes).to_phys();

        info!(
            "requesting pages {:p} - {:p} from uefi",
            kernel_base_phys, kernel_end_phys
        );

        // dump_memory_map(boot_services);

        let ret = boot_services.allocate_pages(
            AllocateType::Address(kernel_base_phys.to_inner()),
            MemoryType::custom(MEMORY_TYPE_KERNEL_IMAGE),
            num_pages,
        )?;

        assert_eq!(ret, kernel_base_phys.to_inner());

        Ok(kernel_base)
    }
}
