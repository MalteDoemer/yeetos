use core::arch::global_asm;

pub mod paging;
pub mod time;

global_asm!(include_str!("ap_startup.s"), options(att_syntax));
