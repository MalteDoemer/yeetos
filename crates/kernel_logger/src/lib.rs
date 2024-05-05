#![no_std]

use core::fmt::Write;

use log::{LevelFilter, Log};

struct KernelLogger;

impl Log for KernelLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        #[cfg(feature = "serial-log")]
        {
            let mut writer = serial::SERIAL_WRITER.lock();
            let _ = write!(writer, "[{}]: {}\n", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    // Note: it is not a problem to call set_logger on multiple cores
    // as only the first call will set the logger and subsequent calls
    // will return an error which is ignored.
    let _ = log::set_logger(&KernelLogger);
    log::set_max_level(LevelFilter::Trace);
}
