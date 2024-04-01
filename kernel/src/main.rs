#![no_std]
#![no_main]

mod ensure_image;

use core::{arch::asm, panic::PanicInfo};

use boot_info::BootInfoHeader;

#[no_mangle]
pub extern "C" fn kernel_main(_boot_info: &BootInfoHeader, proc_id: usize) -> ! {
    unsafe {
        asm!("mov dx, 0x3F8", "mov al, 0x64", "out dx, al",);
    }

    if proc_id == 0 {
        unsafe { ensure_image::test() };
    }
    
    loop {}
}

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
