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

    // Safety:
    // mboot_ptr is passed by boot.s and assumend to be correct.
    let mboot_info = unsafe { Multiboot2Info::new(VirtAddr::new(mboot_ptr)) };

    let initrd = Initrd::from_multiboot_info(&mboot_info);

    let kernel_image = KernelImage::from_initrd(&initrd);

    let kernel_image_size = kernel_image.memory_size().unwrap();

    info!("kernel needs {} bytes of memory.", kernel_image_size);

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
