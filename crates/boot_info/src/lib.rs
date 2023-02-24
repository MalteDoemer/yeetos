#![no_std]

use memory::{MemoryMap, VirtAddr};

pub const BOOT_INFO_STRUCT_V1: usize = 1;

/// The yeetos boot info struct.
///
/// It is used to pass information from the boot stage to the kernel.
///
/// It consists of two parts:
/// - A fixed sized header
/// - A variable sized body
///
/// The boot info struct is guaranteed to be consecutive in memory
/// and all pointers in the header point into the data body.
///
///
/// ```none
/// ----------------
/// |    Header    |
/// ----------------
/// |     Data     |
/// |   (initrd)   |
/// ----------------
/// ```
///
#[repr(C)]
pub struct BootInfoHeader {
    /// Virtual address of this struct
    pub boot_info_addr: VirtAddr,
    /// Total size of the boot info (header + data)
    pub boot_info_size: usize,
    /// Version of the boot information struct.
    /// Must be exactly `BOOT_INFO_STRUCT_V1`
    pub boot_info_version: usize,
    /// Physical Memory map
    pub memory_map: MemoryMap,
    /// The data of the initial ramdisk (initrd).
    pub initrd: &mut [u8],
}
