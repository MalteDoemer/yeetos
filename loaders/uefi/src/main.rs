#![no_main]
#![no_std]
#![allow(dead_code)]

mod acpi;
mod boot_info;
mod bootfs;
mod initrd;
mod panic_handler;
mod time;

extern crate alloc;

// use core::arch::asm;

use bootfs::BootFs;
use initrd::Initrd;
use kernel_image::{KernelImage, KernelImageError};
use log::info;
use memory::PAGE_SIZE;
use tar_no_std::ArchiveEntry;
use uefi::{
    prelude::*,
    table::boot::{AllocateType, MemoryType},
};

pub const MEMORY_TYPE_BOOT_INFO: u32 = 0x80000005;
pub const MEMORY_TYPE_KERNEL_IMAGE: u32 = 0x80000006;

#[entry]
fn main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).expect("uefi::helpers::init() failed");

    let boot_services = system_table.boot_services();

    time::init(boot_services);

    let acpi_tables = acpi::get_acpi_tables(&system_table);
    let num_cores = acpi::number_of_cores(&acpi_tables);

    info!("there are {} cores reported", num_cores);

    let mut bootfs = BootFs::new(handle, boot_services);

    let mut initrd_file = bootfs
        .open_file_readonly(cstr16!("\\yeetos\\initrd"))
        .expect("unable to open \\yeetos\\initrd")
        .into_regular_file()
        .expect("\\yeetos\\initrd is not a file");

    let (_boot_info_header, initrd_buffer) =
        boot_info::allocate_boot_info(boot_services, Initrd::get_number_of_pages(&mut initrd_file));

    let initrd = Initrd::from_file(&mut initrd_file, initrd_buffer);

    // Search for the kernel command line file in the INITRD
    let cmdline_data = initrd
        .file_by_name("cmdline")
        .expect("kernel command line file not found")
        .data_as_str()
        .expect("kernel command line not valid utf-8");

    let kernel_cmdline = kernel_cmdline::KernelCommandLineParser::new(cmdline_data).parse();

    // Search for the kernel file in the INITRD
    let kernel_file = initrd
        .file_by_name("kernel")
        .expect("kernel file not found");

    let kernel_size = KernelImage::compute_total_size(
        acpi::number_of_cores(&acpi_tables),
        kernel_cmdline.stack_size(),
        kernel_cmdline.initial_heap_size(),
        kernel_file.data(),
    )
    .expect("unable to parse kernel elf");

    let kernel_pages = kernel_size.next_multiple_of(PAGE_SIZE) / PAGE_SIZE;

    // TODO: support fixed image using AllocateType::Address()

    let kernel_base: usize = boot_services
        .allocate_pages(
            AllocateType::AnyPages,
            MemoryType::custom(MEMORY_TYPE_KERNEL_IMAGE),
            kernel_pages,
        )
        .expect("unable to allocate pages for the kernel image")
        .try_into()
        .unwrap();

    let kernel_image = KernelImage::new(
        kernel_base.into(),
        acpi::number_of_cores(&acpi_tables),
        kernel_cmdline.stack_size(),
        kernel_cmdline.initial_heap_size(),
        true,
        kernel_file.data(),
    )
    .expect("unable to parse kernel elf");

    let kernel_image_info = kernel_image.kernel_image_info();

    info!(
        "kernel image: {:p} - {:p}",
        kernel_image_info.start(),
        kernel_image_info.end()
    );

    info!("done");

    boot_services.stall(10_000_000);

    Status::SUCCESS
}

// fn create_kernel_image<'a, 'b>(
//     boot_services: &'a BootServices,
//     kernel_file: ArchiveEntry<'b>,
//     num_cores: usize,
//     stack_size: usize,
//     heap_size: usize,
//     use_reloc: bool,
// ) -> KernelImage<'b> {
//     let kernel_size =
//         KernelImage::compute_total_size(num_cores, stack_size, heap_size, kernel_file.data())
//             .expect("unable to parse kernel elf file");

//     let kernel_pages = kernel_size.next_multiple_of(PAGE_SIZE) / PAGE_SIZE;

//     if use_reloc {
//         // since we are using a relocatable kernel image we can
//         // allocate the memory for the kernel anywhere

//         let kernel_base: usize = boot_services
//             .allocate_pages(
//                 AllocateType::AnyPages,
//                 MemoryType::custom(MEMORY_TYPE_KERNEL_IMAGE),
//                 kernel_pages,
//             )
//             .expect("unable to allocate pages for the kernel image")
//             .try_into()
//             .unwrap();
//     } else {
//     }

//     todo!()
// }
