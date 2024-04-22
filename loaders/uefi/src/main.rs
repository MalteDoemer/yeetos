#![no_main]
#![no_std]

mod acpi;
mod boot_info;
mod bootfs;
mod panic_handler;
mod time;

extern crate alloc;

// use core::arch::asm;

use bootfs::BootFs;
use log::info;
use memory::PAGE_SIZE;
use uefi::{
    prelude::*,
    proto::media::file::{File, FileInfo},
};

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

    let info = initrd_file.get_boxed_info::<FileInfo>().unwrap();

    let initrd_size: usize = info.file_size().try_into().unwrap();
    let initrd_pages = initrd_size.next_multiple_of(PAGE_SIZE) / PAGE_SIZE;

    let (_boot_info, _initrd_buffer) = boot_info::allocate_boot_info(boot_services, initrd_pages);

    info!("name={} size={}", info.file_name(), info.file_size());

    boot_services.stall(10_000_000);

    Status::SUCCESS
}
