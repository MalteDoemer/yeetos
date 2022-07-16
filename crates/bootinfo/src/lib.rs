#![no_std]

use memory::VirtAddr;

#[repr(C)]
pub struct BootInfo { 
    /// Virtual address of the initial ram disk
    pub initrd_addr: VirtAddr,
    /// Size of the initial ram disk in bytes
    pub initrd_size: usize,
    /// Virtual address of the memory map
    pub mmap_addr: VirtAddr,
    /// Number of memory map entries
    pub mmap_count: usize,
}