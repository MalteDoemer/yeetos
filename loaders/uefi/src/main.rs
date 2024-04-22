#![no_main]
#![no_std]

mod acpi;
mod bootfs;
mod panic_handler;

extern crate alloc;

// use core::arch::asm;

use bootfs::BootFs;
use log::info;
use uefi::{
    prelude::*,
    proto::media::file::{File, FileInfo},
};

#[entry]
fn main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).expect("uefi::helpers::init() failed");

    let boot_services = system_table.boot_services();

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

    info!("name={} size={}", info.file_name(), info.file_size());

    boot_services.stall(10_000_000);

    Status::SUCCESS
}
