

#[cfg(target_arch = "x86_64")]
mod x86_64 {
    pub const PAGE_SIZE: usize = 4096;
    pub const PAGE_SHIFT: usize = 12;
}


#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

