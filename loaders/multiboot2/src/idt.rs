use core::arch::asm;

use log::info;
use x86::{
    dtables::{lidt, DescriptorTablePointer},
    irq::{
        DIVIDE_ERROR_VECTOR, DOUBLE_FAULT_VECTOR, GENERAL_PROTECTION_FAULT_VECTOR,
        INVALID_OPCODE_VECTOR,
    },
    segmentation::{BuildDescriptor, DescriptorBuilder, GateDescriptorBuilder, SegmentSelector},
    Ring,
};

use crate::devices::{pic, pit};

#[cfg(target_arch = "x86")]
mod arch {
    use x86::segmentation::Descriptor;

    pub type DescriptorType = Descriptor;
    pub type InnerType = u32;

    #[derive(Debug)]
    #[repr(C)]
    pub struct IntStackFrame {
        eflags: u32,
        cs: u32,
        eip: u32,
    }

    impl IntStackFrame {
        pub fn flags(&self) -> u32 {
            self.eflags
        }

        pub fn code_segment(&self) -> u32 {
            self.cs
        }

        pub fn instruction_pointer(&self) -> u32 {
            self.eip
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod arch {
    use x86::bits64::segmentation::Descriptor64;

    pub type DescriptorType = Descriptor64;
    pub type InnerType = u64;

    #[repr(C)]
    pub struct IntStackFrame {
        rflags: u64,
        cs: u64,
        rip: u64,
    }

    impl IntStackFrame {
        pub fn flags(&self) -> u64 {
            self.rflags
        }

        pub fn code_segment(&self) -> u64 {
            self.cs
        }

        pub fn instruction_pointer(&self) -> u64 {
            self.rip
        }
    }
}

pub use arch::IntStackFrame;

pub const NUM_IDT_ENTRIES: usize = 256;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::empty();

#[repr(align(8), C)]
pub struct InterruptDescriptorTable {
    entries: [arch::DescriptorType; NUM_IDT_ENTRIES],
}

pub unsafe trait IntHandlerFunc {
    fn addr(&self) -> usize;
}

pub type IntFunc = extern "x86-interrupt" fn(info: arch::IntStackFrame);
pub type IntFuncErrCode =
    extern "x86-interrupt" fn(info: arch::IntStackFrame, err_code: arch::InnerType);

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

impl InterruptDescriptorTable {
    pub const fn empty() -> Self {
        InterruptDescriptorTable {
            entries: [arch::DescriptorType::NULL; NUM_IDT_ENTRIES],
        }
    }

    pub fn remove_entry(&mut self, idx: u8) {
        self.entries[idx as usize] = arch::DescriptorType::NULL;
    }

    /// # Safety
    /// Loading an incorrect IDT can cause serious memory unsafety.
    unsafe fn load(&self) {
        let ptr = DescriptorTablePointer::<InterruptDescriptorTable>::new(self);
        unsafe {
            lidt(&ptr);
        }
    }

    pub fn set_trap_handler<T: IntHandlerFunc>(&mut self, idx: u8, func: T) {
        let code_sel = SegmentSelector::new(1, Ring::Ring0);

        self.entries[idx as usize] =
            DescriptorBuilder::trap_gate_descriptor(code_sel, func.addr() as arch::InnerType)
                .present()
                .dpl(Ring::Ring0)
                .finish();
    }

    pub fn set_interrupt_handler<T: IntHandlerFunc>(&mut self, idx: u8, func: T) {
        let code_sel = SegmentSelector::new(1, Ring::Ring0);

        self.entries[idx as usize] =
            DescriptorBuilder::interrupt_descriptor(code_sel, func.addr() as arch::InnerType)
                .present()
                .dpl(Ring::Ring0)
                .finish();
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

extern "x86-interrupt" fn divide_by_zero_handler(_frame: arch::IntStackFrame) {
    panic!("divide by zero");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    frame: arch::IntStackFrame,
    err_code: arch::InnerType,
) {
    info!("error: {:#x}", err_code);
    info!("stack_frame: {:?}", frame);
    // info!("instr: {:#x}", frame.instruction_pointer());
    panic!("gerneral protection fault");
}

extern "x86-interrupt" fn invalid_opcode_handler(_frame: arch::IntStackFrame) {
    panic!("invalid opcode");
}

extern "x86-interrupt" fn double_fault_handler(
    _frame: arch::IntStackFrame,
    _err_code: arch::InnerType,
) {
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
