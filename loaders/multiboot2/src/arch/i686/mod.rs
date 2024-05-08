use core::arch::global_asm;

pub mod paging;

global_asm!(include_str!("boot.s"), options(att_syntax));
