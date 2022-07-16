#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![deny(unsafe_op_in_unsafe_fn)]

// enabled while still in early developement phase
#![allow(dead_code)]

extern crate alloc;

mod heap;
mod multiboot2;

use core::{arch::global_asm, panic::PanicInfo};

use memory::VirtAddr;
use multiboot2::Multiboot2Info;

global_asm!(include_str!("boot.s"), options(att_syntax));

#[no_mangle]
pub extern "C" fn rust_entry(mboot_ptr: usize) -> ! {
    heap::init();

    // Safety:
    // mboot_ptr is passed by boot.s and assumend to be correct.
    let mboot_info = unsafe { Multiboot2Info::new(VirtAddr::new(mboot_ptr)) };


    let _cmdline = mboot_info.command_line();
    
    loop {}
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
