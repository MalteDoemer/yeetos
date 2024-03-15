#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
// needed for pit interrupt handler
#![feature(abi_x86_interrupt)]
// needed for acpi module
#![feature(allocator_api)]
// needed for the heap allocator
#![feature(alloc_error_handler)]
// enabled while still in early developement phase
#![allow(dead_code)]

extern crate alloc;

mod acpi;
mod devices;
mod heap;
mod idt;
mod initrd;
mod kernel_image;
mod mmap;
mod multiboot2;
mod vga;

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

use log::{error, info};
use memory::VirtAddr;
use multiboot2::Multiboot2Info;

use crate::{
    initrd::Initrd,
    kernel_image::KernelImage,
    vga::{Color, VGAWriter},
};

global_asm!(include_str!("boot.s"), options(att_syntax));

global_asm!(include_str!("ap_trampoline.s"), options(att_syntax));

#[no_mangle]
pub extern "C" fn rust_entry(mboot_ptr: usize) -> ! {
    // Initialize logger
    boot_logger::init();
    info!("intitialized boot logger...");

    // Initialize IDT
    idt::init();

    // Initialize heap
    heap::init();

    // Initialize PIC, PIT and TSC
    devices::init();

    // Safety:
    // mboot_ptr is passed by boot.s and assumend to be correct.
    let mboot_info = unsafe { Multiboot2Info::new(VirtAddr::new(mboot_ptr)) };

    // Parse the ACPI tables
    let acpi_tables = acpi::get_acpi_tables(
        &mboot_info
            .rsdp_descriptor
            .expect("rsdp descriptor not present"),
    );

    // Get the INITRD module loaded by the multiboot2 loader
    let initrd_module = mboot_info
        .module_by_name("initrd")
        .expect("initrd module not found");

    // Safety:
    // The memory from the initrd module should be safe to access
    let initrd = unsafe { Initrd::from_module(initrd_module) };

    // Search for the kernel file in the INITRD
    let kernel_file = initrd
        .file_by_name("kernel")
        .expect("kernel file not found");

    // The kernel image is loaded right after the initrd
    let kernel_load_addr = initrd.end_addr();

    // Partially parse the kernel elf image
    let kernel_image = KernelImage::new(kernel_load_addr, kernel_file.data())
        .expect("unable to parse the kernel elf image");

    // Calculate the end address of the kernel image
    let kernel_image_end_addr = kernel_image.compute_load_end_address();

    // Create the physical memory map
    let _memory_map = mmap::create_memory_map(
        &mboot_info,
        initrd.end_addr().to_phys(),
        kernel_image_end_addr.to_phys(),
    );

    // Startup the Application Processors
    acpi::acpi_startup_aps(&acpi_tables);

    // for entry in memory_map {
    //     info!("{:p}..{:p}: {:?}", entry.start, entry.end, entry.kind);
    // }

    panic!("finished with main()");
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    error!("{}\n", info);

    boot_logger::get(|log| {
        use core::fmt::Write;
        let mut writer = VGAWriter::new(Color::White, Color::Black);
        let _ = writer.write_str(log);
    });

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
