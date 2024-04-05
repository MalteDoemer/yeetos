#[cfg(target_arch = "x86_64")]
mod x86_64 {
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
}

#[cfg(target_arch = "x86")]
mod x86 {
    /// Base address of the kernel address space
    pub const KERNEL_BASE: usize = 0xC0000000;

    /// The size in bytes of a logical page.
    pub const PAGE_SIZE: usize = 4096;
    pub const PAGE_SHIFT: usize = 12;

    /// The size in bytes of a physical page frame.
    pub const FRAME_SIZE: u32 = 4096;
    pub const FRAME_SHIFT: u32 = 12;

    /// The number of entries in a page table.
    pub const PAGE_TABLE_ENTRIES: usize = 1024;
}

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

#[cfg(target_arch = "x86")]
pub use x86::*;

use crate::VirtAddr;

/// Adds a fixed offset of `KERNEL_BASE` to the address.
pub const fn to_higher_half(addr: VirtAddr) -> VirtAddr {
    VirtAddr::new(addr.to_inner() + KERNEL_BASE)
}

/// Subtracts a fixed offset of `KERNEL_BASE` from the address.
pub const fn to_lower_half(addr: VirtAddr) -> VirtAddr {
    VirtAddr::new(addr.to_inner() - KERNEL_BASE)
}
