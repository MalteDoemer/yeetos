#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
// needed for pit interrupt handler
#![feature(abi_x86_interrupt)]
// needed for acpi module
#![feature(allocator_api)]
// needed for the heap allocator
#![feature(alloc_error_handler)]
// enabled while still in early developement phase
#![allow(dead_code)]

extern crate alloc;

mod acpi;
mod arch;
mod boot_info;
mod devices;
mod heap;
mod idt;
mod initrd;
mod kernel_image;
mod mmap;
mod multiboot2;
mod panic_handling;
mod vga;

use log::info;
use memory::{to_higher_half, VirtAddr};
use multiboot2::Multiboot2Info;

use crate::{
    acpi::{make_jump_to_kernel, KERNEL_ENTRY},
    initrd::Initrd,
    kernel_image::KernelImage,
};

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
    // mboot_ptr is passed by boot.s and assumend to be correct.
    let mboot_info = unsafe { Multiboot2Info::new(VirtAddr::new(mboot_ptr)) };

    // Parse the ACPI tables
    let acpi_tables = acpi::get_acpi_tables(
        &mboot_info
            .rsdp_descriptor
            .expect("rsdp descriptor not present"),
    );

    // Get the INITRD module loaded by the multiboot2 loader
    let initrd = Initrd::from_multiboot_info(&mboot_info).expect("initrd module not found");

    // Search for the kernel file in the INITRD
    let kernel_file = initrd
        .file_by_name("kernel")
        .expect("kernel file not found");

    // Create the KernelImage struct:
    // Normally new_reloc is used to create the kernel image.
    // However when debugging new_fixed is used so that gdb works correctly.
    // Note: this function does not yet load the kernel.
    #[cfg(not(debug_assertions))]
    let kernel_image = KernelImage::new_reloc(
        initrd.end_addr(),
        acpi::number_of_cores(&acpi_tables),
        kernel_file.data(),
    )
    .expect("unable to parse the kernel elf image");
    #[cfg(debug_assertions)]
    let kernel_image =
        KernelImage::new_fixed(acpi::number_of_cores(&acpi_tables), kernel_file.data())
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

    // Initialize some global variables that the ap initialization
    // code will use to set up the stacks for each core.
    acpi::init_kernel_stack_vars(
        kernel_image_info.stack.start().to_addr(),
        kernel_image.kernel_stack_size(),
    );

    // Startup the Application Processors
    acpi::startup_aps(&acpi_tables);

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
