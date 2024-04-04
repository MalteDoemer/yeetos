use core::arch::asm;

pub fn test() {
    unsafe {
        asm!("mov dx, 0x3F8", "mov al, 0x64", "out dx, al",);

        asm!("1: hlt\njmp 1b", options(att_syntax));
    }
}
