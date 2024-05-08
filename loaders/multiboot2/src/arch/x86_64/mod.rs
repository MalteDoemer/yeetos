pub mod paging;

use core::arch::global_asm;

global_asm!(include_str!("boot.s"), options(att_syntax));
