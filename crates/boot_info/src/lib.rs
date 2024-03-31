#![no_std]

pub mod kernel_image_info;
pub mod platform_info;

use kernel_image_info::KernelImageInfo;
use memory::{MemoryMap, VirtAddr};
use platform_info::PlatformInfo;

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
pub struct BootInfoHeader {
    /// Virtual address of this struct
    pub boot_info_addr: VirtAddr,
    /// Total size of the boot info in bytes (header + data)
    pub boot_info_size: usize,
    /// Version of the boot information struct.
    /// Must be exactly `BOOT_INFO_STRUCT_V1`
    pub boot_info_version: usize,
    /// Information about the kernel image.
    pub kernel_image_info: KernelImageInfo,
    /// Information about the current platform
    pub platform_info: PlatformInfo,
    /// Physical Memory map
    pub memory_map: MemoryMap,
    /// The address of the initial ramdisk (initrd).
    pub initrd_addr: VirtAddr,
    /// The size in bytes of the initial ramdisk (initrd).
    pub initrd_size: usize,
}

impl BootInfoHeader {
    pub const fn empty() -> Self {
        BootInfoHeader {
            boot_info_addr: VirtAddr::zero(),
            boot_info_size: 0,
            boot_info_version: BOOT_INFO_STRUCT_V1,
            kernel_image_info: KernelImageInfo::empty(),
            platform_info: PlatformInfo::None,
            memory_map: MemoryMap::new(),
            initrd_addr: VirtAddr::zero(),
            initrd_size: 0,
        }
    }
}
