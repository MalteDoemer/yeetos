use core::arch::global_asm;

pub mod paging;

global_asm!(include_str!("asm/boot.s"), options(att_syntax));

global_asm!(include_str!("asm/ap_startup.s"), options(att_syntax));
