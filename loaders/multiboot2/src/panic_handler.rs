use core::{arch::asm, fmt::Write, panic::PanicInfo};

use log::error;
use spin::Mutex;

static PANIC_LOCK: Mutex<()> = Mutex::new(());

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    let guard = PANIC_LOCK.try_lock();

    if guard.is_some() {
        error!("{}\n", info);
        boot_logger::get(|log| {
            let mut writer = serial::SERIAL_WRITER.lock();
            let _ = write!(writer, "{}", log);
        });
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
