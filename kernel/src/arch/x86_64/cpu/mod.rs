pub mod exceptions;
pub mod features;
pub mod gdt;
pub mod idt;
pub mod local;
pub mod tss;

use core::arch::asm;

/// Initializes the local, gdt, idt and tss modules in the correct order for every core.
pub fn init(proc_id: usize) {
    // Allocates and initializes the `Local` struct
    local::init(proc_id);

    // initializes and loads the per core GDT
    gdt::init();

    // initializes the IDT and loads it on every core
    idt::init();

    // initializes and loads the per core TSS
    tss::init();
}

pub fn halt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}
