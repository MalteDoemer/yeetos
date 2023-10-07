#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
// needed for the heap allocator
#![feature(alloc_error_handler)]
// enabled while still in early developement phase
#![allow(dead_code)]

extern crate alloc;

mod acpi;
mod heap;
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

#[no_mangle]
pub extern "C" fn rust_entry(mboot_ptr: usize) -> ! {
    // initialize heap
    heap::init();

    // initialize logger
    boot_logger::init();

    info!("intitialized boot logger...");

    // Safety:
    // mboot_ptr is passed by boot.s and assumend to be correct.
    let mboot_info = unsafe { Multiboot2Info::new(VirtAddr::new(mboot_ptr)) };

    let initrd_module = mboot_info
        .module_by_name("initrd")
        .expect("initrd module not found");

    // Safety:
    // The memory from the initrd module should be safe to access.
    let initrd = unsafe { Initrd::from_module(initrd_module) };

    let kernel_file = initrd
        .file_by_name("kernel")
        .expect("kernel file not found");
    let kernel_data = kernel_file.data();

    // The kernel image is loaded right after the initrd.
    // TODO: add ASLR for the kernel
    let kernel_load_addr = initrd.end_addr();

    let kernel_image = KernelImage::new(kernel_load_addr, kernel_data)
        .expect("unable to parse the kernel elf image");



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
