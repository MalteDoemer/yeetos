use core::arch::asm;

pub mod paging;
pub mod time;

#[inline(always)]
pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
