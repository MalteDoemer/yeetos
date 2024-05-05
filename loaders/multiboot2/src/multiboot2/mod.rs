use alloc::{string::String, vec::Vec};

use kernel_graphics::FrameBufferInfo;
use log::info;

use memory::virt::VirtAddr;

use self::{tag::Tag, tag_iterator::TagIterator};

mod tag;
mod tag_info;
mod tag_iterator;

#[derive(Debug, Clone, Copy)]
pub struct BasicMemoryInfo {
    pub mem_lower: u32,
    pub mem_upper: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RSDPDescriptorV1 {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oemid: [u8; 6],
    pub revision: u8,
    pub rsdt_physical_address: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RSDPDescriptorV2 {
    pub v1: RSDPDescriptorV1,
    pub length: u32,
    pub xsdt_physical_address: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3],
}

#[derive(Debug, Clone, Copy)]
pub enum RSDPDescriptor {
    V1(RSDPDescriptorV1),
    V2(RSDPDescriptorV2),
}

#[derive(Debug, Clone, Copy)]
pub struct BiosBootDevice {
    pub bios_dev: u32,
    pub partition: u32,
    pub sub_partition: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub base_addr: u64,
    pub length: u64,
    pub region_type: u32,
}

#[derive(Debug)]
pub struct MultibootModule {
    pub mod_start: u32,
    pub mod_end: u32,
    pub info: String,
}

#[derive(Debug)]
pub struct Multiboot2Info {
    pub cmdline: Option<String>,
    pub loader_name: Option<String>,
    pub basic_memory_info: Option<BasicMemoryInfo>,
    pub bios_boot_device: Option<BiosBootDevice>,
    pub modules: Vec<MultibootModule>,
    pub memory_regions: Vec<MemoryRegion>,
    pub image_load_base_physical: Option<u32>,
    pub rsdp_descriptor: Option<RSDPDescriptor>,
    pub frame_buffer_info: Option<FrameBufferInfo>,
}

impl Multiboot2Info {
    /// Creates a new Multiboot2Info object by parsing
    /// the actual multiboot2 information structure passed
    /// by the bootloader.
    /// ### Safety
    /// `mboot_addr` must point to a valid multiboot2 struct
    pub unsafe fn new(mboot_addr: VirtAddr) -> Self {
        // Safety:
        // `mboot_addr` points to a valid multiboot2 struct
        let iter = unsafe { TagIterator::new(mboot_addr) };

        let mut cmdline = None;
        let mut loader_name = None;
        let mut basic_memory_info = None;
        let mut bios_boot_device = None;
        let mut modules = Vec::new();
        let mut memory_regions = Vec::new();
        let mut image_load_base_physical = None;
        let mut rsdp_descriptor = None;
        let mut frame_buffer_info = None;

        for tag in iter {
            match tag {
                Tag::CommandLine(value) => cmdline = Some(value),
                Tag::BootLoaderName(value) => loader_name = Some(value),
                Tag::BasicMemoryInfo(value) => basic_memory_info = Some(value),
                Tag::BiosBootDevice(value) => bios_boot_device = Some(value),
                Tag::ModuleDescriptor(value) => modules.push(value),
                Tag::MemoryRegions(value) => memory_regions = value,
                Tag::ImageLoadBasePhysical(value) => image_load_base_physical = Some(value),
                Tag::OldRSDP(value) => rsdp_descriptor = Some(RSDPDescriptor::V1(value)),
                Tag::NewRSDP(value) => rsdp_descriptor = Some(RSDPDescriptor::V2(value)),
                Tag::FrameBufferInfo(value) => frame_buffer_info = Some(value),
                Tag::Unknown(value) => info!("found unknown multiboot2 tag: {}", value),
                Tag::End => {}
            }
        }

        Multiboot2Info {
            cmdline,
            loader_name,
            basic_memory_info,
            bios_boot_device,
            modules,
            memory_regions,
            image_load_base_physical,
            rsdp_descriptor,
            frame_buffer_info,
        }
    }

    pub fn module_by_name(&self, name: &str) -> Option<&MultibootModule> {
        self.modules.iter().find(|module| module.info == name)
    }
}

impl MultibootModule {
    pub fn start_addr(&self) -> VirtAddr {
        VirtAddr::new(self.mod_start as usize)
    }

    pub fn size(&self) -> usize {
        (self.mod_end - self.mod_start) as usize
    }

    pub fn end_addr(&self) -> VirtAddr {
        self.start_addr() + self.size()
    }
}
