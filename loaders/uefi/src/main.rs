#![no_main]
#![no_std]
#![allow(dead_code)]
// needed for the heap allocator
#![feature(alloc_error_handler)]
#![feature(allocator_api)]

// mod acpi_old;
mod acpi;
mod arch;
mod boot_info;
mod bootfs;
mod entry;
mod heap;
mod mmap;
mod paging;
mod panic_handler;

extern crate alloc;

use crate::entry::{make_jump_to_kernel, KernelEntryInfo, KERNEL_ENTRY};
use crate::mmap::MEMORY_TYPE_KERNEL_IMAGE;
use alloc::format;
use alloc::vec::Vec;
use bootfs::BootFs;
use core::fmt::Write;
use initrd::Initrd;
use kernel_image::ParsedKernelImage;
use log::info;
use memory::{phys::PhysAddr, virt::VirtAddr, PAGE_SIZE};
use uefi::table::boot::{MemoryAttribute, MemoryDescriptor};
use uefi::table::Runtime;
use uefi::{
    prelude::*,
    table::boot::{AllocateType, MemoryType},
};

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
    let num_cores =
        multi_core::number_of_cores(&acpi_tables).expect("acpi processor info not available");

    info!("there are {} cores reported", num_cores);

    // Open a handle to the EFI System Partition
    let mut bootfs =
        BootFs::new(handle, system_table.boot_services()).expect("unable to open boot file system");

    // Find and open the initrd file
    let (mut initrd_file, initrd_pages) = bootfs.open_initrd().expect("unable to open initrd file");

    // Allocate memory for the boot info and INITRD
    let (boot_info_header, initrd_buffer) =
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

    // Prepare paging structures. This function will allocate memory in order to identity and higher-half map the kernel
    // But we will only switch to those page tables once exit_boot_services() has been called.
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
    let (system_table, mmap) = system_table.exit_boot_services(MemoryType::LOADER_DATA);

    // Convert uefi memory map into our own representation
    // This function also validates the memory map to ensure that everything is still accessible
    // after enabling the higher half paging
    let memory_map = mmap::create_memory_map(&mmap).expect("failed to create memory map");

    // for entry in &memory_map {
    //     info!(
    //         "start={:p} size={:#x} type={:?}",
    //         entry.start,
    //         entry.size(),
    //         entry.kind
    //     );
    // }

    // Start all application processors
    acpi::startup_all_application_processors(&acpi_tables, &kernel_image);

    // Load kernel image into memory
    kernel_image.load_kernel().expect("failed to load kernel");

    // Switch to our onw identity/higher-half page tables
    paging::activate();

    let system_table = set_virtual_address_map(system_table, &mmap)
        .expect("unable to translate system table to higher half");

    // Initialize boot info
    boot_info::init_boot_info(
        &system_table,
        boot_info_header,
        &memory_map,
        &initrd,
        kernel_image_info,
    );

    // BootInfoHeader is now initialized
    let boot_info_header = unsafe { boot_info_header.assume_init_mut() };

    let boot_info_addr = boot_info_header.boot_info_addr;
    let entry_point = kernel_image.kernel_entry_point().to_higher_half();
    let stacks_start = kernel_image_info.stack.start_addr().to_higher_half();
    let stack_size = kernel_image.kernel_stack_size();

    let entry = KernelEntryInfo {
        boot_info_addr,
        entry_point,
        stacks_start,
        stack_size,
    };

    KERNEL_ENTRY.call_once(|| entry);

    make_jump_to_kernel(0, entry);
}

fn set_virtual_address_map(
    system_table: SystemTable<Runtime>,
    memory_map: &uefi::table::boot::MemoryMap,
) -> uefi::Result<SystemTable<Runtime>, ()> {
    let mut entries = Vec::<MemoryDescriptor>::new();

    for entry in memory_map.entries() {
        if entry.att.contains(MemoryAttribute::RUNTIME) {
            let phys_addr = PhysAddr::new(entry.phys_start.try_into().unwrap());
            let virt_addr = phys_addr.to_higher_half();

            let mut desc = entry.clone();
            desc.virt_start = virt_addr.to_inner().try_into().unwrap();
            entries.push(desc);
        }
    }

    let current_system_table_addr: usize = system_table
        .get_current_system_table_addr()
        .try_into()
        .unwrap();

    let new_system_table_addr: u64 = VirtAddr::new(current_system_table_addr)
        .to_higher_half()
        .to_inner()
        .try_into()
        .unwrap();

    unsafe { system_table.set_virtual_address_map(&mut entries, new_system_table_addr) }
}

fn dump_memory_map(system_table: &mut SystemTable<Boot>) {
    // let boot_services = system_table.boot_services();

    let mmap_size = system_table.boot_services().memory_map_size();

    let buffer_size = mmap_size.map_size + mmap_size.entry_size * 8;

    let mut vec = Vec::<u8>::with_capacity(buffer_size);
    vec.resize(buffer_size, 0);

    let buffer = vec.as_mut_slice();

    let mut mmap = system_table
        .boot_services()
        .memory_map(buffer)
        .expect("unable to get uefi memory map");

    mmap.sort();

    let mut i = 0;

    for entry in mmap.entries() {
        let start = PhysAddr::new(entry.phys_start);
        // let end = start + FRAME_SIZE * entry.page_count;

        let string = format!(
            "start={:p} pages={:#x} type={:?}\n",
            start, entry.page_count, entry.ty
        );

        let _ = system_table.stdout().write_str(&string);

        i += 1;

        if i % 8 == 0 {
            let _ = system_table
                .stdout()
                .write_str("---------------------------------------------");
            system_table.boot_services().stall(5_000_000);
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
