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
mod devices;
mod heap;
mod idt;
mod initrd;
mod kernel_image;
mod mmap;
mod multiboot2;
mod paging;
mod vga;

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
    sync::atomic::Ordering,
};

use log::{error, info};
use memory::{to_higher_half, VirtAddr};
use multiboot2::Multiboot2Info;

use crate::{
    initrd::Initrd,
    kernel_image::KernelImage,
    vga::{Color, VGAWriter},
};

global_asm!(include_str!("asm/boot.s"), options(att_syntax));

global_asm!(include_str!("asm/ap_startup.s"), options(att_syntax));

extern "C" {
    fn jmp_kernel_entry(
        boot_info_ptr: usize,
        processor_id: usize,
        entry_point: usize,
        stack_ptr: usize,
    ) -> !;
}

#[no_mangle]
pub extern "C" fn rust_entry(mboot_ptr: usize) -> ! {
    // Initialize logger
    boot_logger::init();
    info!("intitialized boot logger...");

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
    let initrd_module = mboot_info
        .module_by_name("initrd")
        .expect("initrd module not found");

    // Safety:
    // The memory from the initrd module should be safe to access
    let initrd = unsafe { Initrd::from_module(initrd_module) };

    // Search for the kernel file in the INITRD
    let kernel_file = initrd
        .file_by_name("kernel")
        .expect("kernel file not found");

    // Create the KernelImage struct:
    // - The load address will be initrd.end_addr()
    // - Right after the load address there will be
    //   place for num_cores * KERNEL_STACK_SIZE bytes
    //   to be used as stack memory
    // - After that the kernel code, rodata and data segments
    //   will be loaded
    // Note: this function does not yet load the kernel.
    let kernel_image = KernelImage::new(
        initrd.end_addr(),
        acpi::number_of_cores(&acpi_tables),
        kernel_file.data(),
    )
    .expect("unable to parse the kernel elf image");

    let kernel_image_info = kernel_image.get_kernel_image_info();

    // Create the physical memory map
    let memory_map = mmap::create_memory_map(
        &mboot_info,
        initrd.end_addr().to_phys(),
        kernel_image_info.end().to_phys(),
    );

    for entry in memory_map {
        info!("{:p}..{:p}: {:?}", entry.start, entry.end, entry.kind);
    }

    // Initialize some global variables that the ap initialization
    // code will use to set up the stacks for each core.
    acpi::init_kernel_stack_vars(
        kernel_image_info.stack.start().to_addr(),
        kernel_image.get_kernel_stack_size(),
    );

    // Startup the Application Processors
    acpi::startup_aps(&acpi_tables);

    // parse elf structure and load the kernel into memory
    kernel_image.load_kernel().expect("failed to load kernel");

    // Enable the higher half mapping
    paging::enable_higher_half();

    let entry_point = to_higher_half(kernel_image.get_kernel_entry_point());

    info!("kernel entry point: {:?}", entry_point);

    devices::tsc::busy_sleep_ms(100);

    info!(
        "we have a total of {} cores running!",
        acpi::AP_COUNT.load(Ordering::SeqCst) + 1
    );

    panic!("finished with main()");
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    error!("{}\n", info);

    boot_logger::get(|log| {
        use core::fmt::Write;
        let mut writer = VGAWriter::new(Color::White, Color::Black);
        let _ = writer.write_str(log);
    });

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
