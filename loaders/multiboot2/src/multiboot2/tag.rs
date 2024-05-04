use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use kernel_graphics::{EgaPixelFormat, IndexedPixelFormat, PixelFormat, RgbColor, RgbPixelFormat};
use memory::{phys::PhysAddr, virt::MemoryReader};

use super::{
    tag_info::*, BasicMemoryInfo, BiosBootDevice, FrameBufferInfo, MemoryRegion, MultibootModule,
    RSDPDescriptorV1, RSDPDescriptorV2,
};

pub(crate) enum Tag {
    End,
    CommandLine(String),
    BootLoaderName(String),
    BasicMemoryInfo(BasicMemoryInfo),
    BiosBootDevice(BiosBootDevice),
    ModuleDescriptor(MultibootModule),
    MemoryRegions(Vec<MemoryRegion>),
    FrameBufferInfo(FrameBufferInfo),
    OldRSDP(RSDPDescriptorV1),
    NewRSDP(RSDPDescriptorV2),
    ImageLoadBasePhysical(u32),
    Unknown(u32),
}

// Note: all parse_* functions assume to receive a valid `TagInfo` with their respective type.
impl Tag {
    /// Creates a `Tag` by parsing a multiboot2 tag.
    ///
    /// ### Safety
    /// - `info` must point to a valid multiboot2 tag
    pub unsafe fn parse(info: TagInfo) -> Self {
        unsafe {
            match info.tag_type() {
                END_TAG => Tag::End,
                BOOT_COMMAND_LINE_TAG => Self::parse_cmdline(info),
                BOOT_LOADER_NAME_TAG => Self::parse_boot_loader_name(info),
                BASIC_MEMORY_INFO_TAG => Self::parse_basic_memory_info(info),
                BIOS_BOOT_DEVICE_TAG => Self::parse_bios_boot_device(info),
                MODULE_INFO_TAG => Self::parse_module_info(info),
                MEMORY_MAP_TAG => Self::parse_memory_regions(info),
                IMAGE_LOAD_BASE_PHYS_ADDR_TAG => Self::parse_image_load_base(info),
                ACPI_RSDP_OLD_TAG => Self::parse_old_rsdp(info),
                ACPI_RSDP_NEW_TAG => Self::parse_new_rsdp(info),
                FRAME_BUFFER_INFO_TAG => Self::parse_frame_buffer_info(info),

                VBE_INFO_TAG => Tag::Unknown(info.tag_type()),
                ELF_SYMBOLS_TAG => Tag::Unknown(info.tag_type()),
                APM_TABLE_TAG => Tag::Unknown(info.tag_type()),
                EFI32_SYSTEM_TABLE_POINTER_TAG => Tag::Unknown(info.tag_type()),
                EFI64_SYSTEM_TABLE_POINTER_TAG => Tag::Unknown(info.tag_type()),
                SMBIOS_TABLES_TAG => Tag::Unknown(info.tag_type()),
                NETWORKING_INFO_TAG => Tag::Unknown(info.tag_type()),
                EFI_MEMORY_MAP_TAG => Tag::Unknown(info.tag_type()),
                EFI_BOOT_SERVICES_NOT_TERMINATED_TAG => Tag::Unknown(info.tag_type()),
                EFI32_IMAGE_HANDLE_POINTER_TAG => Tag::Unknown(info.tag_type()),
                EFI64_IMAGE_HANDLE_POINTER_TAG => Tag::Unknown(info.tag_type()),
                _ => Tag::Unknown(info.tag_type()),
            }
        }
    }

    unsafe fn parse_cmdline(info: TagInfo) -> Tag {
        let size = info.data_size();
        let ptr = info.data_addr().as_ptr::<u8>();

        // Safety:
        // function contract assures a valid tag
        let data = unsafe { core::slice::from_raw_parts(ptr, size) };

        // The boot command line string is UTF-8 with a null byte at the end.
        // In order to make a `&str` the null byte has to be ignored.
        let str_data = &data[..data.len() - 1];

        let str_slice = core::str::from_utf8(str_data).expect("boot command line not valid utf-8");

        Tag::CommandLine(str_slice.to_string())
    }

    unsafe fn parse_boot_loader_name(info: TagInfo) -> Tag {
        let size = info.data_size();
        let ptr = info.data_addr().as_ptr::<u8>();

        // Safety:
        // function contract assures a valid tag
        let data = unsafe { core::slice::from_raw_parts(ptr, size) };

        // The boot loader name string is UTF-8 with a null byte at the end.
        // In order to make a `&str` the null byte has to be ignored.
        let str_data = &data[..data.len() - 1];

        let str_slice = core::str::from_utf8(str_data).expect("boot loader name not valid utf-8");

        Tag::BootLoaderName(str_slice.to_string())
    }

    unsafe fn parse_basic_memory_info(info: TagInfo) -> Tag {
        let ptr = info.data_addr().as_ptr::<u32>();

        // Safety:
        // function contract assures a valid tag
        unsafe {
            let mem_lower = *ptr;
            let mem_upper = *ptr.add(1);

            Tag::BasicMemoryInfo(BasicMemoryInfo {
                mem_lower,
                mem_upper,
            })
        }
    }

    unsafe fn parse_bios_boot_device(info: TagInfo) -> Tag {
        let ptr = info.data_addr().as_ptr::<u32>();

        // Safety:
        // function contract assures a valid tag
        unsafe {
            let bios_dev = *ptr;
            let partition = *ptr.add(1);
            let sub_partition = *ptr.add(2);

            Tag::BiosBootDevice(BiosBootDevice {
                bios_dev,
                partition,
                sub_partition,
            })
        }
    }

    unsafe fn parse_module_info(info: TagInfo) -> Tag {
        let ptr = info.data_addr().as_ptr::<u32>();

        // Safety:
        // function contract assures a valid tag
        unsafe {
            let mod_start = *ptr;
            let mod_end = *ptr.add(1);

            let str_size = info.data_size() - 8;
            let str_ptr = info.data_addr().as_ptr::<u8>().add(8);

            let data = core::slice::from_raw_parts(str_ptr, str_size);
            let str_data = &data[..data.len() - 1]; // skip the last null byte

            let str_slice = core::str::from_utf8(str_data).expect("module string not valid utf-8");

            Tag::ModuleDescriptor(MultibootModule {
                mod_start,
                mod_end,
                info: str_slice.to_string(),
            })
        }
    }

    unsafe fn parse_memory_regions(info: TagInfo) -> Tag {
        let mut reader = MemoryReader::new(info.data_addr());

        let end_addr = info.end_addr();

        let mut regions = Vec::new();

        // Safety:
        // function contract assures a valid tag
        unsafe {
            let entry_size = reader.read::<u32>();
            let _entry_version = reader.read::<u32>();

            while reader.addr() < end_addr {
                let base_addr = reader.read::<u64>();
                let length = reader.read::<u64>();
                let region_type = reader.read::<u32>();

                // 20 bytes are already skipped by the read calls.
                // Note: an entry is always at least 24 bytes so there is no overflow here.
                reader.skip(entry_size as usize - 20);

                regions.push(MemoryRegion {
                    base_addr,
                    length,
                    region_type,
                });
            }

            debug_assert!(reader.addr() == end_addr);

            Tag::MemoryRegions(regions)
        }
    }

    unsafe fn parse_image_load_base(info: TagInfo) -> Tag {
        let ptr = info.data_addr().as_ptr::<u32>();

        // Safety:
        // function contract assures a valid tag
        unsafe {
            let addr = *ptr;

            Tag::ImageLoadBasePhysical(addr)
        }
    }

    unsafe fn parse_old_rsdp(info: TagInfo) -> Tag {
        let ptr = info.data_addr().as_ptr::<RSDPDescriptorV1>();

        // Safety:
        // function contract assures a valid tag
        unsafe {
            let rsdp = *ptr;

            Tag::OldRSDP(rsdp)
        }
    }

    unsafe fn parse_new_rsdp(info: TagInfo) -> Tag {
        let ptr = info.data_addr().as_ptr::<RSDPDescriptorV2>();

        // Safety:
        // function contract assures a valid tag
        unsafe {
            let rsdp = *ptr;

            Tag::NewRSDP(rsdp)
        }
    }

    unsafe fn parse_frame_buffer_info(info: TagInfo) -> Tag {
        let mut reader = MemoryReader::new(info.data_addr());

        // Safety:
        // function contract assures a valid tag
        unsafe {
            let frame_buffer_addr = reader.read::<u64>();
            let frame_buffer_pitch = reader.read::<u32>();
            let frame_buffer_width = reader.read::<u32>();
            let frame_buffer_height = reader.read::<u32>();
            let frame_buffer_bpp = reader.read::<u8>();
            let frame_buffer_type = reader.read::<u8>();

            // Note: the multiboot2 spec says the reserved field is only an u8, but for alignment
            // to be correct it should be an u16 and in the grub multiboot2 header this field is
            // defined as an u16.
            // see https://github.com/rhboot/grub2/blob/fedora-39/include/multiboot2.h#L288
            let _reserved = reader.read::<u16>();

            let pixel_format = Self::parse_color_info(frame_buffer_type, &mut reader);

            Tag::FrameBufferInfo(FrameBufferInfo {
                frame_buffer_addr: PhysAddr::new(frame_buffer_addr),
                frame_buffer_pitch,
                frame_buffer_width,
                frame_buffer_height,
                frame_buffer_bpp,
                pixel_format,
            })
        }
    }
    unsafe fn parse_color_info(frame_buffer_type: u8, reader: &mut MemoryReader) -> PixelFormat {
        unsafe {
            match frame_buffer_type {
                0 => {
                    let mut palette = Vec::new();
                    let frame_buffer_palette_num_colors = reader.read::<u32>();

                    for _ in 0..frame_buffer_palette_num_colors {
                        let red = reader.read::<u8>();
                        let green = reader.read::<u8>();
                        let blue = reader.read::<u8>();

                        palette.push(RgbColor { red, green, blue });
                    }

                    PixelFormat::Indexed(IndexedPixelFormat::new(palette))
                }
                1 => {
                    let frame_buffer_red_field_position = reader.read::<u8>();
                    let frame_buffer_red_mask_size = reader.read::<u8>();
                    let frame_buffer_green_field_position = reader.read::<u8>();
                    let frame_buffer_green_mask_size = reader.read::<u8>();
                    let frame_buffer_blue_field_position = reader.read::<u8>();
                    let frame_buffer_blue_mask_size = reader.read::<u8>();

                    PixelFormat::RGB(RgbPixelFormat::new(
                        frame_buffer_red_field_position,
                        frame_buffer_red_mask_size,
                        frame_buffer_green_field_position,
                        frame_buffer_green_mask_size,
                        frame_buffer_blue_field_position,
                        frame_buffer_blue_mask_size,
                    ))
                }
                2 => PixelFormat::EGA(EgaPixelFormat::new()),
                _ => panic!("invalid frame buffer type: {}", frame_buffer_type),
            }
        }
    }
}
