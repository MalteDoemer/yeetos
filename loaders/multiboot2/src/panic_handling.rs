use core::{arch::asm, panic::PanicInfo};

use log::error;
use spin::Mutex;

use crate::vga::{Color, VGAWriter};

static PANIC_LOCK: Mutex<()> = Mutex::new(());

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;

    let guard = PANIC_LOCK.try_lock();
    if guard.is_some() {
        error!("{}\n", info);
        boot_logger::get(|log| {
            let mut writer = VGAWriter::new(Color::White, Color::Black);
            let _ = writer.write_str(log.as_str());
        });
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
