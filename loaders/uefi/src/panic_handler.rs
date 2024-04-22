use core::{arch::asm, panic::PanicInfo};

use log::error;

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    error!("{}", info);

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
