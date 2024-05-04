use core::{arch::asm, panic::PanicInfo};

use log::error;
use spin::Mutex;

static PANIC_LOCK: Mutex<()> = Mutex::new(());

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    //use core::fmt::Write;

    let guard = PANIC_LOCK.try_lock();

    if guard.is_some() {
        error!("{}\n", info);
        boot_logger::get(|_log| {});
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
