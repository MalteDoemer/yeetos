#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]

mod arch;
mod ensure_image;
mod kresult;
mod panic_handler;

use boot_info::BootInfoHeader;

#[no_mangle]
pub extern "C" fn kernel_main(_boot_info: &BootInfoHeader, proc_id: usize) -> ! {
    arch::test();

    if proc_id == 0 {
        unsafe { ensure_image::test() };
    }

    arch::cpu::halt();
}
