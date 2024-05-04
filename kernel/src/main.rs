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

extern crate alloc;

use log::info;
use spin::Once;

use boot_info::BootInfoHeader;
use kernel_graphics::{RgbColor, RgbWriter};

mod arch;
mod heap;
mod kresult;
mod mm;
mod panic_handler;

static ONCE: Once<()> = Once::new();

#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &BootInfoHeader, proc_id: usize) -> ! {
    kernel_logger::init();

    heap::init(boot_info);

    arch::cpu::features::verify();

    arch::cpu::init(proc_id);

    ONCE.call_once(|| unsafe {
        info!("FrameBuffer: {:?}", &boot_info.frame_buffer_info);

        let mut fb = kernel_graphics::RgbFrameBuffer::from_info(&boot_info.frame_buffer_info)
            .expect("unable to create frame buffer");

        let pixel = fb.make_pixel(RgbColor::new(0, 128, 128));

        for x in 0..100 {
            for y in 0..100 {
                fb.put_pixel_xy(x, y, pixel);
            }
        }
    });

    info!("[CPU {}]: done", proc_id);
    arch::cpu::halt();
}

pub fn write_serial_byte(byte: u8) {
    use x86::io::outb;
    unsafe {
        outb(0x3F8, byte);
    }
}
