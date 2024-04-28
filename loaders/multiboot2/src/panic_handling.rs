use core::{arch::asm, panic::PanicInfo};

use log::error;
use spin::Mutex;
use vga::text_mode::TextMode80x25;

static PANIC_LOCK: Mutex<()> = Mutex::new(());

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;

    let guard = PANIC_LOCK.try_lock();

    if guard.is_some() {
        error!("{}\n", info);
        boot_logger::get(|log| {
            // Safety: we have exclusive access to the VGA frame buffer due to PANIC_LOCK
            let mut writer = unsafe { TextMode80x25::new(0xb8000) };
            let _ = write!(writer, "{}", log.as_str());
        });
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
