#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]

mod arch;
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
pub extern "C" fn kernel_main(_boot_info: &BootInfoHeader, proc_id: usize) -> ! {
    INIT.call_once(|| {
        kernel_logger::init();
        info!("hey from the kernel!");
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
