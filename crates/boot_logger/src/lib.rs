#![no_std]

use core::ops::Deref;

use arrayvec::ArrayString;
use boot_info::boot_logger_info::BOOT_LOG_BUFFER_SIZE;
use spin::Mutex;

static BOOT_LOGGER: BootLogger = BootLogger::new();

struct BootLogger {
    buffer: Mutex<ArrayString<BOOT_LOG_BUFFER_SIZE>>,
}

impl BootLogger {
    pub const fn new() -> Self {
        BootLogger {
            buffer: Mutex::new(ArrayString::new_const()),
        }
    }
}

impl log::Log for BootLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        use core::fmt::Write;

        let mut guard = self.buffer.lock();
        let ref mut writer = *guard;
        // ignore the result since we can't do anything
        let _ = write!(writer, "[{}]: {}\n", record.level(), record.args());
    }

    fn flush(&self) {}
}

pub fn init() {
    let _ = log::set_logger(&BOOT_LOGGER);
    log::set_max_level(log::LevelFilter::Trace);
}

pub fn get<F: FnOnce(&ArrayString<BOOT_LOG_BUFFER_SIZE>) -> ()>(f: F) {
    let guard = BOOT_LOGGER.buffer.lock();
    f(guard.deref())
}
