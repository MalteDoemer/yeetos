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
use log::info;
use uefi::prelude::*;

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

    let _kernel_cmdline = kernel_cmdline::KernelCommandLineParser::new(cmdline_data).parse();

    // Search for the kernel file in the INITRD
    let _kernel_file = initrd
        .file_by_name("kernel")
        .expect("kernel file not found");

    info!("done");

    boot_services.stall(10_000_000);

    Status::SUCCESS
}
