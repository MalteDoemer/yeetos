use crate::arch::paging;
use crate::devices::tsc::busy_sleep_us;
use crate::idt;
use crate::multiboot2::{RSDPDescriptor, RSDPDescriptorV1, RSDPDescriptorV2};
use acpi::AcpiTables;
use boot_acpi::handler::IdentityMappedAcpiHandler;
use kernel_image::KernelImage;
use log::info;
use memory::phys::PhysAddr;
use memory::virt::VirtAddr;
use spin::Once;

#[derive(Copy, Clone)]
pub struct KernelEntryInfo {
    pub entry_point: VirtAddr,
    pub stacks_start: VirtAddr,
    pub stack_size: usize,
}

pub static KERNEL_ENTRY: Once<KernelEntryInfo> = Once::new();

extern "C" {
    // This function is implemented in boot.s
    fn jmp_kernel_entry(
        boot_info_ptr: usize,
        processor_id: usize,
        entry_point: usize,
        stack_ptr: usize,
    ) -> !;
}

pub fn startup_all_application_processors(
    acpi_tables: &AcpiTables<IdentityMappedAcpiHandler>,
    kernel_image: &KernelImage,
) {
    // Defined in boot.s
    let page_table_addr = PhysAddr::new(0x1000);

    boot_acpi::ap_startup::startup_all_application_processors(
        acpi_tables,
        kernel_image,
        page_table_addr,
        busy_sleep_us,
    )
    .unwrap();
}

pub fn get_acpi_tables(rsdp: &RSDPDescriptor) -> AcpiTables<IdentityMappedAcpiHandler> {
    let addr = match rsdp {
        RSDPDescriptor::V1(ref rsdp) => rsdp as *const RSDPDescriptorV1 as usize,
        RSDPDescriptor::V2(ref rsdp) => rsdp as *const RSDPDescriptorV2 as usize,
    };

    let handler = get_handler();

    // Safety:
    // All memory from the rsdp should be correctly mapped and safe to dereference.
    let tables =
        unsafe { AcpiTables::from_rsdp(handler, addr) }.expect("parsing acpi tables failed");

    tables
}

fn get_handler() -> IdentityMappedAcpiHandler {
    #[cfg(target_arch = "x86_64")]
    {
        IdentityMappedAcpiHandler::lower_half()
    }

    #[cfg(target_arch = "x86")]
    {
        IdentityMappedAcpiHandler::all_physical_memory()
    }
}

#[no_mangle]
extern "C" fn rust_entry_ap(ap_id: usize) -> ! {
    info!("application processor #{} started", ap_id);

    // initialize paging for this AP
    paging::init_ap();

    // load IDT for this core
    idt::init_ap();

    // this waits until the BSP is finished initializing
    let entry_point = KERNEL_ENTRY.wait();

    make_jump_to_kernel(ap_id, *entry_point);
}

pub fn make_jump_to_kernel(processor_id: usize, entry: KernelEntryInfo) -> ! {
    let boot_info = crate::boot_info::get_boot_info_addr();

    // calculate stack
    let stack_addr = entry.stacks_start + processor_id * entry.stack_size;

    unsafe {
        jmp_kernel_entry(
            boot_info.to_inner(),
            processor_id,
            entry.entry_point.to_inner(),
            stack_addr.to_inner(),
        )
    };
}
