#![no_std]
// needed for memory reader align
#![feature(int_roundings)]
// needed for frame and page
#![feature(step_trait)]
// needed for checked ops in page and frame
#![feature(const_option)]

extern crate alloc;

mod arch;
mod misc;
mod mmap;

pub mod paging;
pub mod phys;
pub mod virt;

pub use arch::*;
pub use misc::*;
pub use mmap::*;
