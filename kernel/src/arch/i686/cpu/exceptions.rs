#![allow(improper_ctypes_definitions)]

use super::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn divide_by_zero(_frame: InterruptStackFrame) {
    panic!("divide by zero");
}

pub extern "x86-interrupt" fn debug(_frame: InterruptStackFrame) {
    panic!("debug");
}

pub extern "x86-interrupt" fn non_maskable_interrupt(_frame: InterruptStackFrame) {
    panic!("nmi");
}

pub extern "x86-interrupt" fn breakpoint(_frame: InterruptStackFrame) {
    panic!("breakpoint");
}

pub extern "x86-interrupt" fn overflow(_frame: InterruptStackFrame) {
    panic!("overflow");
}

pub extern "x86-interrupt" fn bound_range_exceeded(_frame: InterruptStackFrame) {
    panic!("bound range exceeded");
}

pub extern "x86-interrupt" fn invalid_opcode(_frame: InterruptStackFrame) {
    panic!("invalid opcode");
}

pub extern "x86-interrupt" fn device_not_available(_frame: InterruptStackFrame) {
    panic!("device not available");
}

pub extern "x86-interrupt" fn double_fault(_frame: InterruptStackFrame, error_code: u32) {
    panic!("double fault: {:#x}", error_code);
}

pub extern "x86-interrupt" fn invalid_tss(_frame: InterruptStackFrame, error_code: u32) {
    panic!("invalid tss: {:#x}", error_code);
}

pub extern "x86-interrupt" fn segment_not_present(_frame: InterruptStackFrame, error_code: u32) {
    panic!("segment not present: {:#x}", error_code);
}

pub extern "x86-interrupt" fn stack_segment_fault(_frame: InterruptStackFrame, error_code: u32) {
    panic!("stack segment fault: {:#x}", error_code);
}

pub extern "x86-interrupt" fn general_protection_fault(
    _frame: InterruptStackFrame,
    error_code: u32,
) {
    panic!("general protection fault: {:#x}", error_code);
}

pub extern "x86-interrupt" fn page_fault(_frame: InterruptStackFrame, error_code: u32) {
    panic!("page fault: {:#x}", error_code);
}

pub extern "x86-interrupt" fn floating_point_exception(_frame: InterruptStackFrame) {
    panic!("x87 floating point exception");
}

pub extern "x86-interrupt" fn alignment_check(_frame: InterruptStackFrame, error_code: u32) {
    panic!("alignment check: {:#x}", error_code);
}

pub extern "x86-interrupt" fn machine_check(_frame: InterruptStackFrame) {
    panic!("machine check");
}

pub extern "x86-interrupt" fn simd_floating_point_exception(_frame: InterruptStackFrame) {
    panic!("simd floating point exception");
}

pub extern "x86-interrupt" fn virtualization_exception(_frame: InterruptStackFrame) {
    panic!("virtualization exception");
}

pub extern "x86-interrupt" fn control_protection_exception(
    _frame: InterruptStackFrame,
    error_code: u32,
) {
    panic!("control protection exception: {:#x}", error_code);
}

pub extern "x86-interrupt" fn hypervisor_injection_exception(_frame: InterruptStackFrame) {
    panic!("hypervisor injection exception");
}

pub extern "x86-interrupt" fn vmm_communication_exception(
    _frame: InterruptStackFrame,
    error_code: u32,
) {
    panic!("vmm communication exception: {:#x}", error_code);
}

pub extern "x86-interrupt" fn security_exception(_frame: InterruptStackFrame, error_code: u32) {
    panic!("security exception: {:#x}", error_code);
}
