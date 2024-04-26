use uefi::table::boot::BootServices;

use crate::arch;

pub fn init(boot_services: &BootServices) {
    arch::time::init(boot_services);
}

#[no_mangle]
pub extern "C" fn sleep_ms(millis: u64) {
    arch::time::busy_sleep_ms(millis);
}

#[no_mangle]
pub extern "C" fn sleep_us(micros: u64) {
    arch::time::busy_sleep_us(micros);
}
