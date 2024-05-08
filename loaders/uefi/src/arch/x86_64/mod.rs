use core::arch::{asm, global_asm};

pub mod paging;
pub mod time;

global_asm!(include_str!("entry.asm"), options(att_syntax));

#[inline(always)]
pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
