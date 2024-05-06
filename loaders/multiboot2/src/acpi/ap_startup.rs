use alloc::alloc::Global;
use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering};

use acpi::{platform::interrupt::Apic, platform::ProcessorState, AcpiTables};
use kernel_image::KernelImage;
use log::info;
use memory::virt::VirtAddr;
use spin::Once;
use x86::{
    apic::xapic::XAPIC,
    apic::{ApicControl, ApicId},
};

use crate::{
    acpi::acpi_handler::IdentityMapAcpiHandler,
    arch::paging,
    boot_info,
    devices::tsc::{busy_sleep_ms, busy_sleep_us},
    idt,
};

pub static KERNEL_ENTRY: Once<VirtAddr> = Once::new();

#[no_mangle]
static KERNEL_STACKS_VADDR: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
static KERNEL_STACK_SIZE: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    fn jmp_kernel_entry(
        boot_info_ptr: usize,
        processor_id: usize,
        entry_point: usize,
        stack_ptr: usize,
    ) -> !;
}

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
            startup_ap(&mut local_apic, apic_id as u8);
        });
}

fn install_ap_trampoline() {
    extern "C" {
        pub fn ap_trampoline();
        pub fn ap_trampoline_end();
        pub fn ap_trampoline_dest();
    }

    let ap_trampoline_src = ap_trampoline as usize;
    let ap_trampoline_src_end = ap_trampoline_end as usize;
    let ap_trampoline_dest_addr = ap_trampoline_dest as usize;

    let src = ap_trampoline_src as *const u8;
    let dst = ap_trampoline_dest_addr as *mut u8;
    let size = ap_trampoline_src_end - ap_trampoline_src;

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

fn startup_ap(local_apic: &mut XAPIC, id: u8) {
    // This code follows the guidelines on https://wiki.osdev.org/Symmetric_Multiprocessing
    let apic_id = ApicId::XApic(id);

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

pub fn make_jump_to_kernel(processor_id: usize, entry_point_addr: VirtAddr) -> ! {
    let boot_info = boot_info::get_boot_info_addr();

    // calculate stack
    let stacks_base = VirtAddr::new(KERNEL_STACKS_VADDR.load(Ordering::SeqCst)).to_higher_half();
    let stack_size = KERNEL_STACK_SIZE.load(Ordering::SeqCst);
    let stack_addr = stacks_base + processor_id * stack_size;

    unsafe {
        jmp_kernel_entry(
            boot_info.to_inner(),
            processor_id,
            entry_point_addr.to_inner(),
            stack_addr.to_inner(),
        )
    };
}
