#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
// needed for the heap allocator
#![feature(alloc_error_handler)]
// enabled while still in early developement phase
#![allow(dead_code)]

extern crate alloc;

mod heap;
mod multiboot2;
mod vga;
mod acpi;

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

use log::{error, info};
use memory::VirtAddr;
use multiboot2::Multiboot2Info;

use crate::vga::{Color, VGAWriter};

global_asm!(include_str!("boot.s"), options(att_syntax));

#[no_mangle]
pub extern "C" fn rust_entry(mboot_ptr: usize) -> ! {
    heap::init();

    // initialize logger
    boot_logger::init();

    // Safety:
    // mboot_ptr is passed by boot.s and assumend to be correct.
    let mboot_info = unsafe { Multiboot2Info::new(VirtAddr::new(mboot_ptr)) };

    info!("{:?}", mboot_info);

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
