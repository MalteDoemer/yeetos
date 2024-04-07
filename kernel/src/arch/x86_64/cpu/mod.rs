pub mod features;

use core::arch::asm;

pub fn halt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}
