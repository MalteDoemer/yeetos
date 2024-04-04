#![no_std]
#![no_main]

mod arch;
mod ensure_image;
mod panic_handler;

use boot_info::BootInfoHeader;

#[no_mangle]
pub extern "C" fn kernel_main(_boot_info: &BootInfoHeader, proc_id: usize) -> ! {
    arch::test();

    if proc_id == 0 {
        unsafe { ensure_image::test() };
    }

    arch::halt_cpu();
}
