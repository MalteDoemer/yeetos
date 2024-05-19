#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]
// needed for gs_deref!() macro
#![feature(asm_const)]
// needed for the heap allocator
#![feature(alloc_error_handler)]
// needed for try_new() functions
#![feature(allocator_api)]
// needed for idt.rs
#![feature(abi_x86_interrupt)]
// used for fallible allocations
#![feature(vec_push_within_capacity)]

extern crate alloc;

use log::info;

use crate::mm::{GlobalFrameAllocator, KernelVirtualAllocator};
use boot_info::BootInfoHeader;
use memory::phys::PageFrameAllocator;
use memory::virt::VirtualRangeAllocator;

mod arch;
mod heap;
mod kresult;
mod mm;
mod panic_handler;

#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &BootInfoHeader, proc_id: usize) -> ! {
    kernel_logger::init();

    heap::init(boot_info);

    arch::cpu::features::verify();

    arch::cpu::init(proc_id);

    mm::init(boot_info);

    let frame = GlobalFrameAllocator.alloc();
    info!("[CPU {}]: got frame: {:?}", proc_id, frame);

    let range = KernelVirtualAllocator.alloc(4, 1).unwrap();
    info!(
        "[CPU {}]: got range: {:?} - {:#x}",
        proc_id,
        range.start(),
        range.num_pages()
    );

    info!("[CPU {}]: done", proc_id);
    arch::cpu::halt();
}

pub fn write_serial_byte(byte: u8) {
    use x86::io::outb;
    unsafe {
        outb(0x3F8, byte);
    }
}
