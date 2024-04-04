use core::panic::PanicInfo;

use crate::arch;

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    arch::halt_cpu();
}
