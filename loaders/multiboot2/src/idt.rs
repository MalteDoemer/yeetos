//! This module handles setting up an IDT for the bootloader.
//! Note: this IDT is only intended to be used by the bootloader
//! and the kernel should set up its own IDT after gaining control.

use core::arch::asm;

use log::info;
use x86::{
    bits64::segmentation::Descriptor64,
    dtables::{lidt, DescriptorTablePointer},
    irq::{
        DIVIDE_ERROR_VECTOR, DOUBLE_FAULT_VECTOR, GENERAL_PROTECTION_FAULT_VECTOR,
        INVALID_OPCODE_VECTOR,
    },
    segmentation::{BuildDescriptor, DescriptorBuilder, GateDescriptorBuilder, SegmentSelector},
    Ring,
};

use crate::devices::{pic, pit};

pub const NUM_IDT_ENTRIES: usize = 256;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::empty();

#[repr(C)]
pub struct IntStackFrame {
    ss: u64,
    rsp: u64,
    rflags: u64,
    cs: u64,
    rip: u64,
}

#[repr(align(8), C)]
pub struct InterruptDescriptorTable {
    entries: [Descriptor64; NUM_IDT_ENTRIES],
}

pub unsafe trait IntHandlerFunc {
    fn addr(&self) -> usize;
}

pub type IntFunc = extern "x86-interrupt" fn(info: IntStackFrame);
pub type IntFuncErrCode = extern "x86-interrupt" fn(info: IntStackFrame, err_code: u64);
pub type HandlerFunc = extern "C" fn();

unsafe impl IntHandlerFunc for IntFunc {
    fn addr(&self) -> usize {
        *self as usize
    }
}

unsafe impl IntHandlerFunc for IntFuncErrCode {
    fn addr(&self) -> usize {
        *self as usize
    }
}

unsafe impl IntHandlerFunc for HandlerFunc {
    fn addr(&self) -> usize {
        *self as usize
    }
}

impl InterruptDescriptorTable {
    pub const fn empty() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            entries: [Descriptor64::NULL; NUM_IDT_ENTRIES],
        }
    }

    pub fn set_trap_handler<T: IntHandlerFunc>(&mut self, idx: u8, func: T) {
        let code_sel = SegmentSelector::new(1, Ring::Ring0);
        self.entries[idx as usize] =
            DescriptorBuilder::trap_gate_descriptor(code_sel, func.addr() as u64)
                .present()
                .dpl(Ring::Ring0)
                .finish();
    }

    pub fn set_interrupt_handler<T: IntHandlerFunc>(&mut self, idx: u8, func: T) {
        let code_sel = SegmentSelector::new(1, Ring::Ring0);

        self.entries[idx as usize] =
            DescriptorBuilder::interrupt_descriptor(code_sel, func.addr() as u64)
                .present()
                .dpl(Ring::Ring0)
                .finish();
    }

    pub fn remove_entry(&mut self, idx: u8) {
        self.entries[idx as usize] = Descriptor64::NULL;
    }

    /// # Safety
    /// Loading an incorrect IDT can cause serious memory unsafety.
    unsafe fn load(&self) {
        let ptr = DescriptorTablePointer::<InterruptDescriptorTable>::new(self);
        unsafe {
            lidt(&ptr);
        }
    }
}

pub fn init() {
    // Safety: there is no data-race accessing the IDT
    // during initialization.
    unsafe {
        IDT.set_trap_handler::<IntFunc>(DIVIDE_ERROR_VECTOR, divide_by_zero_handler);
        IDT.set_trap_handler::<IntFunc>(INVALID_OPCODE_VECTOR, invalid_opcode_handler);
        IDT.set_trap_handler::<IntFuncErrCode>(DOUBLE_FAULT_VECTOR, double_fault_handler);
        IDT.set_trap_handler::<IntFuncErrCode>(
            GENERAL_PROTECTION_FAULT_VECTOR,
            general_protection_fault_handler,
        );
        IDT.set_interrupt_handler::<IntFunc>(pic::PRIMARY_VECTOR_OFFSET + 0x00, pit::pit_interrupt);
        IDT.load();
    }
}

extern "x86-interrupt" fn divide_by_zero_handler(_frame: IntStackFrame) {
    panic!("divide by zero");
}

extern "x86-interrupt" fn general_protection_fault_handler(frame: IntStackFrame, err_code: u64) {
    info!("error: {:#x}", err_code);
    info!("instr: {:#x}", frame.rip);
    panic!("gerneral protection fault");
}

extern "x86-interrupt" fn invalid_opcode_handler(_frame: IntStackFrame) {
    panic!("invalid opcode");
}

extern "x86-interrupt" fn double_fault_handler(_frame: IntStackFrame, _err_code: u64) {
    // if we are here, there is something seriously wrong, so its probably not a good idea to call panic
    // so we instead do a hardcoded message with outb to port 0x3F8 (COM1)
    // The message consist of following bytes:
    // 0x64, 0x6f, 0x75, 0x62, 0x6c, 0x65, 0x20, 0x66, 0x61, 0x75, 0x6c, 0x74
    // which translates to "double fault"
    // and then we just halt the cpu.

    unsafe {
        asm!(
            "mov dx, 0x3F8",
            "mov al, 0x64",
            "out dx, al",
            "mov al, 0x6f",
            "out dx, al",
            "mov al, 0x75",
            "out dx, al",
            "mov al, 0x62",
            "out dx, al",
            "mov al, 0x6c",
            "out dx, al",
            "mov al, 0x65",
            "out dx, al",
            "mov al, 0x20",
            "out dx, al",
            "mov al, 0x66",
            "out dx, al",
            "mov al, 0x61",
            "out dx, al",
            "mov al, 0x75",
            "out dx, al",
            "mov al, 0x6c",
            "out dx, al",
            "mov al, 0x74",
            "out dx, al",
            "mov al, 0x0a",
            "out dx, al",
            "hlt",
        );
    }
}
