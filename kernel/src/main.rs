#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]
// Used for init_allocator.rs
#![feature(allocator_api)]

mod arch;
mod init_allocator;
mod kresult;
mod panic_handler;

use boot_info::BootInfoHeader;
use log::info;
use spin::Once;

static INIT: Once<()> = Once::new();

extern "C" {
    fn doesnt_exist();
}

#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &BootInfoHeader, proc_id: usize) -> ! {
    INIT.call_once(|| {
        kernel_logger::init();

        arch::cpu::features::verify();

        init_allocator::init(boot_info);
    });

    let _ = INIT.wait();
    info!("[CPU {}]: done", proc_id);

    arch::cpu::halt();
}

pub fn write_serial_byte(byte: u8) {
    use x86::io::outb;
    unsafe {
        outb(0x3F8, byte);
    }
}
