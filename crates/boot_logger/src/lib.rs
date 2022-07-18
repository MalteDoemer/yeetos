#![no_std]

use arrayvec::ArrayString;
use spin::Mutex;

const BOOT_LOG_BUFFER_SIZE: usize = 2 * 1024;

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
        let _ = write!(writer, "[{}]: {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

pub fn init() {
    let _ = log::set_logger(&BootLogger);
    log::set_max_level(log::LevelFilter::Trace);
}

pub fn get<F: FnOnce(&str) -> ()>(f: F) {
    let guard = BUFFER.lock();
    let s = guard.as_str();
    f(s);
}
