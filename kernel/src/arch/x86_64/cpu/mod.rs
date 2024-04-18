pub mod exceptions;
pub mod features;
pub mod gdt;
pub mod idt;
pub mod local;

use core::arch::asm;

/// Initializes the local, gdt, idt and tss modules in the correct order for every core.
pub fn init(proc_id: usize) {
    local::init(proc_id);

    idt::init();
}

pub fn halt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}
