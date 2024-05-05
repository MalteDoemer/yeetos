use core::{fmt::Write, panic::PanicInfo};

use log::error;
use spin::Mutex;

use crate::arch;

static PANIC_LOCK: Mutex<()> = Mutex::new(());

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    let guard = PANIC_LOCK.try_lock();

    if guard.is_some() {
        error!("{}", info);
        boot_logger::get(|log| {
            let mut writer = serial::SERIAL_WRITER.lock();
            let _ = write!(writer, "{}", log);
        });
    }

    arch::halt();
}
