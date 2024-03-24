
#[cfg(target_arch = "x86_64")]
mod paging_x64;

#[cfg(target_arch = "x86_64")]
pub use paging_x64::*;

