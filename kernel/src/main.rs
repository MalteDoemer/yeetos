#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]
// needed for gs_deref!()
#![feature(asm_const)]
// needed for the heap allocator
#![feature(alloc_error_handler)]
// needed for idt.rs
#![feature(abi_x86_interrupt)]

mod arch;
mod heap;
mod kresult;
mod panic_handler;

extern crate alloc;

use boot_info::BootInfoHeader;
use log::info;

#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &BootInfoHeader, proc_id: usize) -> ! {
    kernel_logger::init();

    heap::init(boot_info);

    arch::cpu::features::verify();
    
    arch::cpu::init(proc_id);

    info!("[CPU {}]: done", proc_id);
    arch::cpu::halt();
}


pub fn write_serial_byte(byte: u8) {
    use x86::io::outb;
    unsafe {
        outb(0x3F8, byte);
    }
}
