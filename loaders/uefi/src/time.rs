#[no_mangle]
pub extern "C" fn sleep_ms(millis: u64) {
    sleep_us(millis * 1000);
}

#[no_mangle]
pub extern "C" fn sleep_us(micros: u64) {
    let system_table = uefi::helpers::system_table();
    let boot_services = system_table.boot_services();

    boot_services.stall(micros.try_into().unwrap());
}
