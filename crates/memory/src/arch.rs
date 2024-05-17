#[cfg(target_arch = "x86_64")]
mod x86_64 {
    /// Base address of the kernel address space
    pub const KERNEL_BASE: usize = 0xfffff00000000000;

    /// The number of bytes that are identity mapped (with higher half addresses) when the kernel
    /// gains control.
    ///
    /// This means that when the kernel gains control the physical memory range of
    /// `0..IDENTITY_MAP_SIZE` is mapped to the virtual memory range
    /// `KERNEL_BASE..(KERNEL_BASE + IDENTITY_MAP_SIZE)`
    pub const IDENTITY_MAP_SIZE: usize = 4 * 1024 * 1024 * 1024; // 4 GiB

    /// The size in bytes of a logical page.
    pub const PAGE_SIZE: usize = 4096;
    pub const PAGE_SHIFT: usize = 12;

    /// The size in bytes of a physical page frame.
    pub const FRAME_SIZE: u64 = 4096;
    pub const FRAME_SHIFT: u64 = 12;

    pub const LAREGE_PAGE_SIZE: usize = 0x200000; // 2 MiB
    pub const LAREGE_PAGE_SHIFT: usize = 21;

    pub const LARGE_FRAME_SIZE: u64 = 0x200000; // 2 MiB
    pub const LARGE_FRAME_SHIFT: u64 = 21;

    /// The number of entries in a page table.
    pub const PAGE_TABLE_ENTRIES: usize = 512;
}

#[cfg(target_arch = "x86")]
mod x86 {
    /// Base address of the kernel address space
    pub const KERNEL_BASE: usize = 0xC0000000;

    /// The number of bytes that are identity mapped (with higher half addresses) when the kernel
    /// gains control.
    ///
    /// This means that when the kernel gains control the physical memory range of
    /// `0..IDENTITY_MAP_SIZE` is mapped to the virtual memory range
    /// `KERNEL_BASE..(KERNEL_BASE + IDENTITY_MAP_SIZE)`
    pub const IDENTITY_MAP_SIZE: usize = 1 * 1024 * 1024 * 1024; // 1 GiB

    /// The size in bytes of a logical page.
    pub const PAGE_SIZE: usize = 4096;
    pub const PAGE_SHIFT: usize = 12;

    /// The size in bytes of a physical page frame.
    pub const FRAME_SIZE: u32 = 4096;
    pub const FRAME_SHIFT: u32 = 12;

    pub const LAREGE_PAGE_SIZE: usize = 0x400000; // 2 MiB
    pub const LAREGE_PAGE_SHIFT: usize = 22;

    pub const LARGE_FRAME_SIZE: u32 = 0x400000; // 2 MiB
    pub const LARGE_FRAME_SHIFT: u32 = 22;

    /// The number of entries in a page table.
    pub const PAGE_TABLE_ENTRIES: usize = 1024;
}

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

use crate::phys::PhysAddr;
#[cfg(target_arch = "x86")]
pub use x86::*;

use crate::virt::VirtAddr;

/// Translates a lower-half virtual address to a higher-half virtual address.
pub const fn virt_to_higher_half_checked(addr: VirtAddr) -> Option<VirtAddr> {
    let inner = addr.to_inner();

    if inner >= IDENTITY_MAP_SIZE {
        return None;
    }

    match inner.checked_add(KERNEL_BASE) {
        Some(addr) => Some(VirtAddr::new(addr)),
        None => None,
    }
}

/// Translates a lower-half physical address to a higher-half virtual address.
pub fn phys_to_higher_half_checked(addr: PhysAddr) -> Option<VirtAddr> {
    // Note: lower-half physical and virtual addresses are identical.
    let virt = addr.try_into().ok()?;
    virt_to_higher_half_checked(virt)
}

/// This function checks whether a given address is in the identity mapped higher half
/// address range
pub const fn is_identity_mapped_higher_half_address(addr: VirtAddr) -> bool {
    let inner = addr.to_inner();
    inner >= KERNEL_BASE && inner - KERNEL_BASE <= IDENTITY_MAP_SIZE
}

/// Translates a higher-half virtual address to a lower-half virtual address.
pub const fn virt_to_lower_half_checked(addr: VirtAddr) -> Option<VirtAddr> {
    if is_identity_mapped_higher_half_address(addr) {
        let inner = addr.to_inner();
        // Note: we don't need checked_sub() here since we used  is_identity_mapped_higher_half_address()
        Some(VirtAddr::new(inner - KERNEL_BASE))
    } else {
        None
    }
}
