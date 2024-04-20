use memory::virt::VirtAddr;
use spin::Once;
use x86::{
    bits32::eflags::EFlags,
    dtables::{lidt, DescriptorTablePointer},
    irq::{
        ALIGNMENT_CHECK_VECTOR, BOUND_RANGE_EXCEEDED_VECTOR, BREAKPOINT_VECTOR, DEBUG_VECTOR,
        DEVICE_NOT_AVAILABLE_VECTOR, DIVIDE_ERROR_VECTOR, DOUBLE_FAULT_VECTOR,
        GENERAL_PROTECTION_FAULT_VECTOR, INVALID_OPCODE_VECTOR, INVALID_TSS_VECTOR,
        MACHINE_CHECK_VECTOR, NONMASKABLE_INTERRUPT_VECTOR, OVERFLOW_VECTOR, PAGE_FAULT_VECTOR,
        SEGMENT_NOT_PRESENT_VECTOR, SIMD_FLOATING_POINT_VECTOR, STACK_SEGEMENT_FAULT_VECTOR,
        VIRTUALIZATION_VECTOR, X87_FPU_VECTOR,
    },
    segmentation::{
        BuildDescriptor, Descriptor, DescriptorBuilder, GateDescriptorBuilder, SegmentSelector,
    },
    Ring,
};

use crate::arch::cpu::exceptions;

use super::gdt;

const IDT_ENTRIES: usize = 256;

/// This trait represents any function that can be used as an interrupt handler.
/// It is unsafe since interrupt handlers are very specific function that have to follow several additional rules.
pub unsafe trait InterruptHandlerFunction {
    const HAS_ERROR_CODE: bool;

    fn addr(&self) -> usize;
}

/// This struct represents the processor state pushed onto the stack when an interrupt occures.
///
/// # Note
/// It is my understanding that `stack_pointer` and `stack_segment` only get pushed by the cpu if there is a privilege level change
/// (i.e from Ring3 to Ring0). Simmilarly when executing `iretq` the cpu only pops the stack pointer and segment if the privilege level changes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptStackFrame {
    pub instruction_pointer: VirtAddr,
    pub code_segment: SegmentSelector,
    _reserved1: [u8; 2],
    pub cpu_flags: EFlags,
    pub stack_pointer: VirtAddr,
    pub stack_segment: SegmentSelector,
    _reserved2: [u8; 2],
}

/// The Interrupt Descriptor Table.
#[repr(C, align(8))]
pub struct InterruptDescriptorTable {
    entries: [Descriptor; IDT_ENTRIES],
}

#[allow(improper_ctypes_definitions)]
pub type InterruptHandler = extern "x86-interrupt" fn(frame: InterruptStackFrame);

#[allow(improper_ctypes_definitions)]
pub type InterruptHandlerErrorCode =
    extern "x86-interrupt" fn(frame: InterruptStackFrame, err_code: u32);

unsafe impl InterruptHandlerFunction for InterruptHandler {
    const HAS_ERROR_CODE: bool = false;

    fn addr(&self) -> usize {
        *self as usize
    }
}

unsafe impl InterruptHandlerFunction for InterruptHandlerErrorCode {
    const HAS_ERROR_CODE: bool = true;

    fn addr(&self) -> usize {
        *self as usize
    }
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        let mut idt = Self {
            entries: [Descriptor::NULL; IDT_ENTRIES],
        };

        // Safety: it is assumed that these entries are all properly configured.
        unsafe {
            idt.set_exception_handlers();
        }

        idt
    }

    pub unsafe fn load(&self) {
        let ptr = DescriptorTablePointer::<Self>::new(self);
        unsafe {
            lidt(&ptr);
        }
    }

    /// Set a trap gate entry. Trap gates are used to handle CPU exceptions.
    /// # Parameters
    /// - `index` the index of the entry to set
    /// - `code_sel` the code segment selector to use when executing the function. This should almost always be `gdt::KERNEL_CODE_SEL`.
    /// - `dpl` specifies the privilege level needed to "call" this entry using the `INT` instruction. This should be `Ring::Ring0` for most entries except for the system call interrupt.
    /// - `function` the function that is going to handle this exception
    pub unsafe fn set_trap_handler<T: InterruptHandlerFunction>(
        &mut self,
        index: u8,
        code_sel: SegmentSelector,
        dpl: Ring,
        function: T,
    ) {
        #[cfg(debug_assertions)]
        Self::verify_index::<T>(index);

        self.entries[index as usize] =
            <DescriptorBuilder as GateDescriptorBuilder<u32>>::trap_gate_descriptor(
                code_sel,
                function.addr() as u32,
            )
            .present()
            .dpl(dpl)
            .finish();
    }

    /// Set a interrupt gate entry. Interrupt gates are used to handle ordinary interrupts (Timer, Network, ...) and systemcalls (INT 0x80).
    /// # Parameters
    /// - `index` the index of the entry to set
    /// - `code_sel` the code segment selector to use when executing the function. This should almost always be `gdt::KERNEL_CODE_SEL`.
    /// - `dpl` specifies the privilege level needed to "call" this entry using the `INT` instruction. This should be `Ring::Ring0` for most entries except for the system call interrupt.
    /// - `function` the function that is going to handle this interrupt
    pub unsafe fn set_interrupt_gate_handler<T: InterruptHandlerFunction>(
        &mut self,
        index: u8,
        code_sel: SegmentSelector,
        dpl: Ring,
        function: T,
    ) {
        #[cfg(debug_assertions)]
        Self::verify_index::<T>(index);

        self.entries[index as usize] =
            <DescriptorBuilder as GateDescriptorBuilder<u32>>::interrupt_descriptor(
                code_sel,
                function.addr() as u32,
            )
            .present()
            .dpl(dpl)
            .finish();
    }

    pub unsafe fn set_exception_handler(&mut self, index: u8, function: InterruptHandler) {
        unsafe {
            self.set_trap_handler(index, gdt::KERNEL_CODE_SEL, Ring::Ring0, function);
        }
    }
    pub unsafe fn set_exception_handler_error_code(
        &mut self,
        index: u8,
        function: InterruptHandlerErrorCode,
    ) {
        unsafe {
            self.set_trap_handler(index, gdt::KERNEL_CODE_SEL, Ring::Ring0, function);
        }
    }

    pub unsafe fn set_kernel_interrupt_handler(&mut self, index: u8, function: InterruptHandler) {
        unsafe {
            self.set_interrupt_gate_handler(index, gdt::KERNEL_CODE_SEL, Ring::Ring0, function);
        }
    }

    unsafe fn set_exception_handlers(&mut self) {
        unsafe {
            self.set_exception_handler(DIVIDE_ERROR_VECTOR, exceptions::divide_by_zero);
            self.set_exception_handler(DEBUG_VECTOR, exceptions::debug);
            self.set_exception_handler(
                NONMASKABLE_INTERRUPT_VECTOR,
                exceptions::non_maskable_interrupt,
            );
            self.set_exception_handler(BREAKPOINT_VECTOR, exceptions::breakpoint);
            self.set_exception_handler(OVERFLOW_VECTOR, exceptions::overflow);
            self.set_exception_handler(
                BOUND_RANGE_EXCEEDED_VECTOR,
                exceptions::bound_range_exceeded,
            );
            self.set_exception_handler(INVALID_OPCODE_VECTOR, exceptions::invalid_opcode);
            self.set_exception_handler(
                DEVICE_NOT_AVAILABLE_VECTOR,
                exceptions::device_not_available,
            );

            self.set_exception_handler_error_code(DOUBLE_FAULT_VECTOR, exceptions::double_fault);
            self.set_exception_handler_error_code(INVALID_TSS_VECTOR, exceptions::invalid_tss);
            self.set_exception_handler_error_code(
                SEGMENT_NOT_PRESENT_VECTOR,
                exceptions::segment_not_present,
            );
            self.set_exception_handler_error_code(
                STACK_SEGEMENT_FAULT_VECTOR,
                exceptions::stack_segment_fault,
            );
            self.set_exception_handler_error_code(
                GENERAL_PROTECTION_FAULT_VECTOR,
                exceptions::general_protection_fault,
            );
            self.set_exception_handler_error_code(PAGE_FAULT_VECTOR, exceptions::page_fault);
            self.set_exception_handler_error_code(
                ALIGNMENT_CHECK_VECTOR,
                exceptions::alignment_check,
            );

            self.set_exception_handler_error_code(0x15, exceptions::control_protection_exception);
            self.set_exception_handler_error_code(0x1D, exceptions::vmm_communication_exception);
            self.set_exception_handler_error_code(0x1E, exceptions::security_exception);

            self.set_exception_handler(X87_FPU_VECTOR, exceptions::floating_point_exception);
            self.set_exception_handler(MACHINE_CHECK_VECTOR, exceptions::machine_check);
            self.set_exception_handler(
                SIMD_FLOATING_POINT_VECTOR,
                exceptions::simd_floating_point_exception,
            );

            self.set_exception_handler(VIRTUALIZATION_VECTOR, exceptions::virtualization_exception);
            self.set_exception_handler(0x1C, exceptions::hypervisor_injection_exception);
        }
    }

    #[cfg(debug_assertions)]
    fn verify_index<T: InterruptHandlerFunction>(index: u8) {
        let has_err_code = T::HAS_ERROR_CODE;

        match index {
            15 | 22..=27 | 31 => {
                panic!("idt entry {} is reserved and should not be assigned", index)
            }

            9 => {
                panic!("idt entry 9 (coprocessor segment overrun) is deprecated and should not be asigned")
            }

            0..=7 | 16 | 18..=20 | 28 | 32..=255 => {
                if has_err_code {
                    panic!("idt entry {} should have no error code", index)
                }
            }

            8 | 10..=14 | 17 | 21 | 29 | 30 => {
                if !has_err_code {
                    panic!("idt entry {} should have an error code", index)
                }
            }
        }
    }
}

static IDT: Once<InterruptDescriptorTable> = Once::new();

pub(super) fn init() {
    IDT.call_once(|| InterruptDescriptorTable::new());

    // Safety: it is assumed that the IDT is correctly set up.
    unsafe {
        IDT.wait().load();
    }
}
