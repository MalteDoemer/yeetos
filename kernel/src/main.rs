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

#[no_mangle]
pub extern "C" fn kernel_main(_boot_info: &BootInfoHeader, _proc_id: usize) -> ! { 
    INIT.call_once(|| {    
        kernel_logger::init();
        info!("hey from the kernel!");
    });

    arch::cpu::halt();
}
