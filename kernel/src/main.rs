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

use crate::mm::{
    FixedFrameAllocator, GlobalFrameAllocator, KernelVirtualAllocator, PhysicalMemoryObject,
};
use boot_info::BootInfoHeader;
use memory::phys::{Frame, PageFrameAllocator, PhysicalRange};
use memory::FRAME_SIZE;

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

    let fb = &boot_info.frame_buffer_info;
    let fixed = FixedFrameAllocator::new(fb.physical_range());
    let pmo = PhysicalMemoryObject::new_shared_in(fixed.num_frames(), fixed).unwrap();

    info!("[CPU {}]: done", proc_id);
    arch::cpu::halt();
}

pub fn write_serial_byte(byte: u8) {
    use x86::io::outb;
    unsafe {
        outb(0x3F8, byte);
    }
}
