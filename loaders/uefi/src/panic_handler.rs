use core::{
    ffi::c_void,
    fmt::Write,
    panic::PanicInfo,
    sync::atomic::{AtomicPtr, Ordering},
};

use log::error;
use spin::Mutex;
use uefi::table::{Boot, SystemTable};
use vga::text_mode::TextMode80x25;

use crate::arch;

/// Reference to the system table.
///
/// This table is only fully safe to use until UEFI boot services have been exited.
static SYSTEM_TABLE: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());

static PANIC_LOCK: Mutex<()> = Mutex::new(());

fn system_table_opt() -> Option<SystemTable<Boot>> {
    let ptr = SYSTEM_TABLE.load(Ordering::Acquire);
    // Safety: the `SYSTEM_TABLE` pointer either be null or a valid system
    // table.
    //
    // Null is the initial value, as well as the value set when exiting boot
    // services. Otherwise, the value is set by the call to `init`, which
    // requires a valid system table reference as input.
    unsafe { SystemTable::from_ptr(ptr) }
}

pub fn init(st: &SystemTable<Boot>) {
    if system_table_opt().is_some() {
        // Avoid double initialization.
        return;
    }

    SYSTEM_TABLE.store(st.as_ptr().cast_mut(), Ordering::SeqCst);
}

pub fn on_exit_boot_services() {
    SYSTEM_TABLE.store(core::ptr::null_mut(), Ordering::SeqCst);
}

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    let guard = PANIC_LOCK.try_lock();

    if guard.is_some() {
        // print the panic message
        error!("{}", info);

        if let Some(mut system_table) = system_table_opt() {
            // if we are still using boot_services then
            // we can use stdout to print the boot log
            boot_logger::get(|log| {
                let _ = system_table.stdout().write_str(log.as_str());
            });
        } else {
            // otherwise use the vga text buffer
            // Note: we just assume it is set up for 80x25 text mode
            boot_logger::get(|log| {
                // Safety: we have exclusive access to the VGA frame buffer since we use the PANIC_LOCK
                let mut writer = unsafe { TextMode80x25::new(0xb8000) };
                let _ = write!(writer, "{}", log.as_str());
            });
        }
    }

    arch::halt();
}
