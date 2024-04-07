use core::arch::asm;

pub mod features;

pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
