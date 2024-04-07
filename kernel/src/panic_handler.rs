use core::panic::PanicInfo;

use log::error;

use crate::arch;

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    error!("{}", info);
    arch::cpu::halt();
}
