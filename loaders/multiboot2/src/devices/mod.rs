use x86::io::outb;

pub mod pic;
pub mod pit;
pub mod serial;
pub mod tsc;

pub fn init() {
    pic::init();
    pit::init();
    tsc::init();
}

/// Makes a dummy write to IO port 0x80.
/// This ensures a small delay for slower io devices.
pub unsafe fn io_delay() {
    // Safety:
    // No safety issues with writing to port 0x80.
    unsafe {
        outb(0x80, 0);
    }
}
