use alloc::vec::Vec;
use core::ptr::addr_of_mut;

use boot_info::{
    platform_info::{
        pc_x86::{self, PCx86Info},
        PlatformInfo,
    },
    BootInfoHeader, BOOT_INFO_STRUCT_V1,
};
use initrd::Initrd;
use kernel_image::KernelImageInfo;
use memory::{
    to_higher_half,
    virt::{Page, VirtAddr, VirtualRange},
    MemoryMap, MemoryMapEntry, MEMORY_MAP_ENTRIES,
};

use crate::multiboot2::{self, Multiboot2Info};

#[link_section = ".boot_info"]
static mut BOOT_INFO_HEADER: BootInfoHeader = BootInfoHeader::empty();

/// Get the higher-half address of the boot_info header.
pub fn get_boot_info_addr() -> VirtAddr {
    let boot_info_ptr = unsafe { addr_of_mut!(BOOT_INFO_HEADER) };
    to_higher_half((boot_info_ptr as usize).into())
}

pub fn init_boot_info<'a>(
    mboot: &Multiboot2Info,
    map: &Vec<MemoryMapEntry>,
    initrd: &Initrd<'a>,
    kernel_image_info: &KernelImageInfo,
) {
    // Safety: this function is only called in the BSP
    // and any further access only happens after this function finished.
    let boot_info = unsafe { &mut *addr_of_mut!(BOOT_INFO_HEADER) };

    let boot_info_start = get_boot_info_addr();
    let boot_info_end = to_higher_half(initrd.end_addr());

    boot_info.boot_info_addr = boot_info_start;
    boot_info.boot_info_size = boot_info_end - boot_info_start;
    boot_info.boot_info_version = BOOT_INFO_STRUCT_V1;

    boot_info.kernel_image_info = get_kernel_image_info(kernel_image_info);

    boot_info.frame_buffer_info = mboot.frame_buffer_info.clone().unwrap_or_default();

    boot_info.platform_info = get_platform_info(mboot);
    boot_info.memory_map = get_memory_map(map);

    boot_logger::get(|log| {
        boot_info.boot_logger = *log;
    });

    boot_info.initrd_addr = to_higher_half(initrd.start_addr());
    boot_info.initrd_size = initrd.size();
}

/// This function translates all addresses to higher half
fn get_kernel_image_info(kernel_image_info: &KernelImageInfo) -> KernelImageInfo {
    KernelImageInfo {
        stack: translate_range_to_higher_half(kernel_image_info.stack),
        rodata: translate_optional_range_to_higher_half(kernel_image_info.rodata),
        code: translate_range_to_higher_half(kernel_image_info.code),
        relro: translate_optional_range_to_higher_half(kernel_image_info.relro),
        data: translate_optional_range_to_higher_half(kernel_image_info.data),
        heap: translate_range_to_higher_half(kernel_image_info.heap),
    }
}

fn translate_range_to_higher_half(range: VirtualRange) -> VirtualRange {
    let page = Page::new(to_higher_half(range.start_addr()));
    VirtualRange::new(page, range.num_pages())
}

fn translate_optional_range_to_higher_half(range: Option<VirtualRange>) -> Option<VirtualRange> {
    range.map(|rng| translate_range_to_higher_half(rng))
}

fn get_platform_info(mboot: &Multiboot2Info) -> PlatformInfo {
    let info = PCx86Info {
        rsdp: convert_rsdp(mboot.rsdp_descriptor.expect("rsdp descriptor not present")),
    };

    PlatformInfo::PCX86(info)
}

fn convert_rsdp(rsdp: multiboot2::RSDPDescriptor) -> pc_x86::Rsdp {
    match rsdp {
        multiboot2::RSDPDescriptor::V1(rsdpv1) => pc_x86::Rsdp::V1(pc_x86::RsdpV1 {
            signature: rsdpv1.signature,
            checksum: rsdpv1.checksum,
            oem_id: rsdpv1.oemid,
            revision: rsdpv1.revision,
            rsdt_addr: rsdpv1.rsdt_physical_address,
        }),
        multiboot2::RSDPDescriptor::V2(rsdpv2) => pc_x86::Rsdp::V2(pc_x86::RsdpV2 {
            signature: rsdpv2.v1.signature,
            checksum: rsdpv2.v1.checksum,
            oem_id: rsdpv2.v1.oemid,
            revision: rsdpv2.v1.revision,
            rsdt_addr: rsdpv2.v1.rsdt_physical_address,
            length: rsdpv2.length,
            xsdt_addr: rsdpv2.xsdt_physical_address,
            extended_checksum: rsdpv2.extended_checksum,
            reserved: rsdpv2.reserved,
        }),
    }
}

fn get_memory_map(map: &Vec<MemoryMapEntry>) -> MemoryMap {
    assert!(map.len() <= MEMORY_MAP_ENTRIES);

    let mut result = MemoryMap::new();
    for (i, entry) in map.iter().enumerate() {
        result.entries[i] = *entry;
    }

    result
}
