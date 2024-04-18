use core::arch::asm;

pub mod exceptions;
pub mod features;
pub mod gdt;
pub mod idt;
pub mod local;
pub mod tss;

pub fn init(proc_id: usize) {
    local::init(proc_id);

    idt::init();
}

pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
