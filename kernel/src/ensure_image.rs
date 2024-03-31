//! This module exists for (at least for now) because it is important that
//! the linker generates all possible phdrs: RODATA, CODE, RELRO, DATA.
//! This is currently assumed by the loader and thus leads to a panic
//! if the program headers are missing. To solve this we just need to have
//! some dummy variables.

#[no_mangle]
static mut DATA: u8 = 1;
static RODATA: u8 = 2;

pub unsafe fn test() {
    unsafe {
        DATA = RODATA;
    }
}
