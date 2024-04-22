use core::sync::atomic::{AtomicUsize, Ordering};

use uefi::table::boot::BootServices;

static BS_ADDR: AtomicUsize = AtomicUsize::new(0);

// static BS_ADDR: Once<VirtAddr> = Once::new();

pub fn init(boot_services: &BootServices) {
    let addr = boot_services as *const BootServices as usize;
    BS_ADDR.store(addr, Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn sleep_ms(millis: u64) {
    sleep_us(millis * 1000);
}

#[no_mangle]
pub extern "C" fn sleep_us(micros: u64) {
    let addr = BS_ADDR.load(Ordering::SeqCst);

    if addr == 0 {
        panic!("sleep_us() used before time::init()");
    }

    let boot_services = unsafe { &*(addr as *const BootServices) };

    boot_services.stall(micros.try_into().unwrap());
}
