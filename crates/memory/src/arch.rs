#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use crate::VirtAddr;

    /// Base address of the kernel address space
    pub const KERNEL_BASE: usize = 0xfffff00000000000;

    /// The size in bytes of a logical page.
    pub const PAGE_SIZE: usize = 4096;
    pub const PAGE_SHIFT: usize = 12;

    /// The size in bytes of a physical page frame.
    pub const FRAME_SIZE: usize = 4096;
    pub const FRAME_SHIFT: usize = 12;

    /// The number of entries in a page table.
    pub const PAGE_TABLE_ENTRIES: usize = 512;

    /// Adds a fixed offset of `KERNEL_BASE` to the address.
    pub fn to_higher_half(addr: VirtAddr) -> VirtAddr {
        addr + KERNEL_BASE
    }
}

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;
