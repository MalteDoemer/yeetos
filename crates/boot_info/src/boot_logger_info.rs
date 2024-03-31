use arrayvec::ArrayString;

pub const BOOT_LOG_BUFFER_SIZE: usize = 2 * 1024;
pub type BootLoggerInfo = ArrayString<BOOT_LOG_BUFFER_SIZE>;
