#![no_std]

use arrayvec::ArrayVec;
use boot_logger_info::BootLoggerInfo;
use kernel_graphics::FrameBufferInfo;
use kernel_image::KernelImageInfo;
use memory::{virt::VirtAddr, MemoryMap};
use platform_info::PlatformInfo;

pub mod boot_logger_info;
pub mod platform_info;

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
    /// Information about the frame buffer provided by the boot loader.
    pub frame_buffer_info: FrameBufferInfo,
    /// Information about the current platform
    pub platform_info: PlatformInfo,
    /// Physical Memory map
    pub memory_map: MemoryMap,
    /// A fixed size string containing to logging output during the boot loader
    pub boot_logger: BootLoggerInfo,
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
            frame_buffer_info: FrameBufferInfo::empty(),
            platform_info: PlatformInfo::None,
            memory_map: MemoryMap::new(ArrayVec::new_const()),
            boot_logger: BootLoggerInfo::new_const(),
            initrd_addr: VirtAddr::zero(),
            initrd_size: 0,
        }
    }
}
