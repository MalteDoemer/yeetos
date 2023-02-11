#![no_std]

use memory::VirtAddr;

pub const BOOT_INFO_STRUCT_V1: usize = 1;
pub const BOOT_INFO_STRUCT_SIZE: usize = 56;

/// The yeetos boot-info struct.
///
/// It is used to pass information from the boot stage to the kernel.
/// The total boot information is made up of three pieces:
/// - The `BootInfo` struct
/// - The memory map / memory region info
/// - The initial ram disk (initrd) containing various other pieces.
///
/// All three pieces are guaranteed to be consecutive in memory (with some padding in between):
/// ```none
/// --------------   <-- boot_info_addr
/// | Boot Info  |
/// --------------   <-- mmap_addr
/// | Memory Map |
/// --------------   <-- initrd_addr
/// |   Initrd   |
/// --------------
/// ```
///
#[repr(C)]
pub struct BootInfo {
    /// Virtual address of this struct
    pub boot_info_addr: VirtAddr,
    /// Size of this struct in bytes.
    /// Must be exactly `BOOT_INFO_STRUCT_SIZE`
    pub boot_info_size: usize,
    /// Version of the boot information struct.
    /// Must be exactly `BOOT_INFO_STRUCT_V1`
    pub boot_info_version: usize,
    /// Virtual address of the memory map
    pub mmap_addr: VirtAddr,
    /// Number of memory map entries
    pub mmap_count: usize,
    /// Virtual address of the initial ram disk
    pub initrd_addr: VirtAddr,
    /// Size of the initial ram disk in bytes
    pub initrd_size: usize,
}
