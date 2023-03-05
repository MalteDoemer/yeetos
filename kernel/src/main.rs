#![no_std]
#![no_main]

use core::panic::PanicInfo;

static TEST4: [u64; 256] = [55; 256];

static mut TEST3: u64 = 23;

static mut TEST: u64 = 0;

#[no_mangle]
pub extern "C" fn kernel_main() {
    unsafe {
        TEST = 6;
        TEST3 = 66;
    }

    let _x = TEST4[55];

    test();

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
