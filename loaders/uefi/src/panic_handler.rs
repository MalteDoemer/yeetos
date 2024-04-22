use core::{arch::asm, panic::PanicInfo};

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    // error!("{}", info);
    // arch::cpu::halt();

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
