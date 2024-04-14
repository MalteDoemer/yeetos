#![no_std]

use core::fmt::Write;

use log::{LevelFilter, Log};

#[cfg(feature = "serial-log")]
mod serial;

#[cfg(feature = "vga-log")]
mod text_vga;

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

        #[cfg(feature = "vga-log")]
        {
            let mut writer = text_vga::VGA_WRITER.lock();
            let _ = write!(writer, "[{}]: {}\n", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_once() {
    let _ = log::set_logger(&KernelLogger);
    log::set_max_level(LevelFilter::Trace);
}
