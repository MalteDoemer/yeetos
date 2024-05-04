#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
// needed for pit interrupt handler
#![feature(abi_x86_interrupt)]
// needed for acpi module
#![feature(allocator_api)]
// needed for the heap allocator
#![feature(alloc_error_handler)]
// enabled while still in early development phase
#![allow(dead_code)]

extern crate alloc;

use initrd::Initrd;
use kernel_image::KernelImage;
use log::info;
use memory::{to_higher_half, virt::VirtAddr};
use multiboot2::Multiboot2Info;

use crate::acpi::{make_jump_to_kernel, KERNEL_ENTRY};

mod acpi;
mod arch;
mod boot_info;
mod devices;
mod heap;
mod idt;
mod mmap;
mod multiboot2;
mod panic_handler;

#[no_mangle]
pub extern "C" fn rust_entry(mboot_ptr: usize) -> ! {
    // Initialize logger
    boot_logger::init();

    // Initialize IDT
    idt::init();

    // Initialize heap
    heap::init();

    // Initialize PIC, PIT and TSC
    devices::init();

    // Safety:
    // mboot_ptr is passed by boot.s and assumed to be correct.
    let mboot_info = unsafe { Multiboot2Info::new(VirtAddr::new(mboot_ptr)) };

    // Parse the ACPI tables
    let acpi_tables = acpi::get_acpi_tables(
        &mboot_info
            .rsdp_descriptor
            .expect("rsdp descriptor not present"),
    );

    let num_cores = acpi::number_of_cores(&acpi_tables);

    // Get the INITRD module loaded by the multiboot2 loader
    let initrd_module = mboot_info
        .module_by_name("initrd")
        .expect("initrd module not found");

    // Safety:
    // The memory from a multiboot2 module is assumed to be safely accessible.
    let initrd = unsafe {
        Initrd::from_addr_size(initrd_module.start_addr(), initrd_module.size())
            .expect("unable to parse initrd")
    };

    let cmdline_data = initrd
        .file_by_name("cmdline")
        .expect("cmdline file not found")
        .data_as_str()
        .expect("kernel command line not valid utf-8");

    let kernel_cmdline = kernel_cmdline::KernelCommandLineParser::new(cmdline_data).parse();

    // Search for the kernel file in the INITRD
    let kernel_file = initrd
        .file_by_name("kernel")
        .expect("kernel file not found");

    // Create the KernelImage struct.
    // Relocation will be used based on the kernel command line argument `use_reloc` which defaults to true.
    // Relocation can be disabled in order to facilitate debugging with gdb.
    // Note: this does not yet load the kernel.

    let kernel_image_base_addr = if kernel_cmdline.use_reloc() {
        Some(initrd.end_addr())
    } else {
        None
    };

    let kernel_image = KernelImage::new(
        kernel_image_base_addr,
        num_cores,
        kernel_cmdline.stack_size(),
        kernel_cmdline.initial_heap_size(),
        kernel_file.data(),
    )
    .expect("unable to parse the kernel elf image");

    let kernel_image_info = kernel_image.kernel_image_info();

    // Create the physical memory map
    let memory_map = mmap::create_memory_map(
        &mboot_info,
        initrd.end_addr().to_phys(),
        kernel_image_info.end().to_phys(),
    );

    for entry in &memory_map {
        info!(
            "start={:p} end={:p} type={:?}",
            entry.start, entry.end, entry.kind
        );
    }

    // Startup the Application Processors
    acpi::startup_aps(&acpi_tables, &kernel_image);

    // Parse elf structure and load the kernel into memory
    kernel_image.load_kernel().expect("failed to load kernel");

    // Initialize paging and enable the higher half mapping.
    //
    // Note: after this access to some addresses is no longer possible
    // This means functions such as startup_aps() and get_acpi_tables() must be called before
    //
    // Also: most of the initialization of the paging structs is done in boot.s
    // on x86_64 paging is already enabled in boot.s
    arch::paging::init();

    // Get the entry point address from the kernel image and translate it into
    // a higher-half address.
    let entry_point = to_higher_half(kernel_image.kernel_entry_point());

    // Initialize the boot_info header
    boot_info::init_boot_info(&mboot_info, &memory_map, &initrd, &kernel_image_info);

    // This releases all started AP's to enter the kernel
    KERNEL_ENTRY.call_once(|| entry_point);

    // Note: BSP has id of 0
    make_jump_to_kernel(0, entry_point);
}
