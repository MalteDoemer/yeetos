#![no_std]

use core::ops::Deref;

use arrayvec::ArrayString;
use boot_info::boot_logger_info::BOOT_LOG_BUFFER_SIZE;
use spin::Mutex;

static BUFFER: Mutex<ArrayString<BOOT_LOG_BUFFER_SIZE>> = Mutex::new(ArrayString::new_const());

struct BootLogger;

impl log::Log for BootLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        use core::fmt::Write;

        let mut guard = BUFFER.lock();
        let ref mut writer = *guard;
        // ignore the result since we can't do anything
        let _ = write!(writer, "[{}]: {}\n", record.level(), record.args());
    }

    fn flush(&self) {}
}

pub fn init() {
    let _ = log::set_logger(&BootLogger);
    log::set_max_level(log::LevelFilter::Trace);
}

pub fn get<F: FnOnce(&ArrayString<BOOT_LOG_BUFFER_SIZE>) -> ()>(f: F) {
    let guard = BUFFER.lock();
    f(guard.deref())
}
