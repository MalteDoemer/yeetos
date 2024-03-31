#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

static TEST4: [u64; 256] = [55; 256];

#[no_mangle]
static mut TEST3: u64 = 23;

static mut TEST: u64 = 0;

#[no_mangle]
pub extern "C" fn kernel_main() {
    unsafe {
        TEST = 6;
        TEST3 = 66;
    }

    let _x = TEST4[55];

    unsafe {
        asm!("mov dx, 0x3F8", "mov al, 0x64", "out dx, al",);
    }

    loop {}
}

pub fn test() {
    unsafe {
        TEST = TEST4[11];
    }
}

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
