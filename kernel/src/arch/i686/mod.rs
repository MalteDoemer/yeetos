use core::arch::global_asm;

pub mod cpu;
pub mod interrupts;
pub mod paging;

global_asm!(include_str!("asm.s"), options(att_syntax));
