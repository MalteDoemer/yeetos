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
use spin::Once;

static INIT: Once<()> = Once::new();

extern "C" {
    fn doesnt_exist();
}

#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &BootInfoHeader, proc_id: usize) -> ! {
    // perform one-time initialization on a single core
    INIT.call_once(|| init_once(boot_info));
    let _ = INIT.wait();

    // perform initialization on every core
    init_all(boot_info, proc_id);

    info!("[CPU {}]: done", proc_id);
    arch::cpu::halt();
}

fn init_once(boot_info: &BootInfoHeader) {
    kernel_logger::init_once();

    heap::init_once(boot_info);

    arch::cpu::init_once();
}

fn init_all(_boot_info: &BootInfoHeader, proc_id: usize) {
    arch::cpu::features::verify();

    arch::cpu::init_all(proc_id);
}

pub fn write_serial_byte(byte: u8) {
    use x86::io::outb;
    unsafe {
        outb(0x3F8, byte);
    }
}
