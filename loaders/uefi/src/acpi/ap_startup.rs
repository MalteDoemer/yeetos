use alloc::alloc::Global;
use core::sync::atomic::{AtomicUsize, Ordering};
use kernel_image::KernelImage;
use memory::virt::VirtAddr;
use spin::Once;
use x86::apic::{xapic::XAPIC, ApicControl, ApicId};

use acpi::{
    platform::{interrupt::Apic, ProcessorState},
    AcpiTables,
};

use crate::acpi::acpi_handler::IdentityMapAcpiHandler;
use crate::arch::time::{busy_sleep_ms, busy_sleep_us};

pub static KERNEL_ENTRY: Once<VirtAddr> = Once::new();

#[no_mangle]
static KERNEL_STACKS_VADDR: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
static KERNEL_STACK_SIZE: AtomicUsize = AtomicUsize::new(0);

pub fn startup_all_application_processors(
    acpi_tables: &AcpiTables<IdentityMapAcpiHandler>,
    kernel_image: &KernelImage,
) {
    let stacks_addr = kernel_image.kernel_image_info().stack.start_addr();
    KERNEL_STACKS_VADDR.store(stacks_addr.to_inner(), Ordering::SeqCst);
    KERNEL_STACK_SIZE.store(kernel_image.kernel_stack_size(), Ordering::SeqCst);

    let num_cores = kernel_image.num_cores();

    if num_cores >= 256 {
        panic!("number of cores may not exceed 255");
    }
    let platform_info = acpi_tables
        .platform_info()
        .expect("unable to get acpi platform info");

    let processor_info = platform_info
        .processor_info
        .expect("unable to get acpi processor info");

    let apic_info = match platform_info.interrupt_model {
        acpi::InterruptModel::Apic(apic) => apic,
        _ => panic!("acpi interrupt model unknown"),
    };

    let mut local_apic = get_local_apic(apic_info);

    install_ap_trampoline();

    processor_info
        .application_processors
        .iter()
        .filter(|&ap| ap.state == ProcessorState::WaitingForSipi)
        .map(|ap| ap.local_apic_id.try_into().unwrap())
        .for_each(|apic_id: usize| {
            // We only set up `num_cores` of kernel stacks, and we use the apic_id in boot.s to
            // load a stack pointer. Thus, if the apic_id is bigger than num_cores
            // (for whatever reason) we will get memory corruption.
            assert!(apic_id < num_cores);

            startup_ap(&mut local_apic, ApicId::XApic(apic_id as u8));
        });
}

fn install_ap_trampoline() {
    extern "C" {
        pub fn ap_trampoline();
        pub fn ap_trampoline_end();
        pub fn ap_trampoline_dest();
    }

    let ap_trampoline_addr = ap_trampoline as usize;
    let ap_trampoline_end_addr = ap_trampoline_end as usize;
    let ap_trampoline_dest_addr = 0x8000;

    let src = ap_trampoline_addr as *const u8;
    let dst = ap_trampoline_dest_addr as *mut u8;
    let size = ap_trampoline_end_addr - ap_trampoline_addr;

    unsafe {
        core::ptr::copy(src, dst, size);
    }
}

fn get_local_apic(apic_info: Apic<Global>) -> XAPIC {
    let base_addr: usize = apic_info
        .local_apic_address
        .try_into()
        .expect("local apic address to large");

    let ptr = base_addr as *mut u32;
    // Note: I'm not 100% sure on the size of this slice, but since it is never really used as
    // one it shouldn't really matter
    let size = 0x400;

    // Safety:
    // The memory for the local apic should be safely accessible.
    let slice = unsafe { core::slice::from_raw_parts_mut(ptr, size) };

    XAPIC::new(slice)
}

fn startup_ap<T: ApicControl>(local_apic: &mut T, apic_id: ApicId) {
    // This code follows the guidelines on https://wiki.osdev.org/Symmetric_Multiprocessing
    unsafe {
        local_apic.ipi_init(apic_id);

        local_apic.ipi_init_deassert();
        busy_sleep_ms(10);

        for _ in 0..2 {
            local_apic.ipi_startup(apic_id, 0x08);
            busy_sleep_us(200);
        }
    }
}
