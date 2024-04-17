pub mod exceptions;
pub mod features;
pub mod gdt;
pub mod idt;
pub mod local;
pub mod tss;

use core::arch::asm;

pub fn init_once() {
    // initializes the shared IDT
    idt::init_once();
}

/// Initializes the local, gdt, idt and tss modules in the correct order for every core.
pub fn init_all(proc_id: usize) {
    // Allocates and initializes the `Local` struct
    local::init_all(proc_id);

    // initializes and loads the per core GDT
    gdt::init_all();

    // loads the shared IDT
    // Note: idt::init_once() is called before
    idt::init_all();

    // initializes and loads the per core TSS
    tss::init_all();
}

pub fn halt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}
