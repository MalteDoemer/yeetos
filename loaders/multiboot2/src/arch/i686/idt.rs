#[repr(C)]
pub struct IntStackFrame {
    eflags: u32,
    cs: u32,
    eip: u32,
}

pub fn init() {
    todo!()
}
