use core::{cell::RefCell, marker::PhantomData, ptr::NonNull};

use alloc::boxed::Box;
use x86::{
    bits64::task::TaskStateSegment,
    gs_deref,
    msr::{wrmsr, IA32_GS_BASE},
    segmentation::{gs, load_gs},
};

use super::gdt::{self, GlobalDescriptorTable};

/// This type can be used to make a struct !Send .
type PhantomUnsend = PhantomData<*mut ()>;

/// CPU local data.
pub struct Local {
    /// The processor id (apic id) of this core.
    pub proc_id: usize,
    /// The GDT for this CPU.
    /// Note: only the tss_desc entry is diffrent, all other fields are the same for all cores.
    pub gdt: GlobalDescriptorTable,
    /// The TSS which holds the stack pointer for system calls.
    pub tss: TaskStateSegment,
}

/// Each core/cpu will hold a pointer to a `LocalWrapper` object in it's
/// gsbase / kernel_gsbase msr. Thus when in kernel mode a `movq %ds:0, %rax` i.e. `mov rax, [gs:0]` will
/// read the `self_ref` of this struct, through which we can then access our cpu local data.
struct LocalWrapper {
    /// A pointer to this `LocalWrapper` struct.
    self_ref: NonNull<LocalWrapper>,
    /// The `Local` struct with dynamic borrow checking through a `RefCell`.
    local: RefCell<Local>,
    /// This is here to make LocalWrapper !Send because it should never be
    /// used across other cores/threads.
    _phantom: PhantomUnsend,
}

impl Local {
    pub fn new(proc_id: usize) -> Self {
        Self {
            proc_id,
            tss: TaskStateSegment::new(),
            gdt: GlobalDescriptorTable::new(),
        }
    }
}

pub(super) fn init(proc_id: usize) {
    let mut local = Box::new(LocalWrapper {
        self_ref: NonNull::dangling(),
        local: RefCell::new(Local::new(proc_id)),
        _phantom: PhantomData,
    });

    local.self_ref = local.as_ref().into();

    let ptr = Box::leak(local) as *mut LocalWrapper;

    unsafe {
        // Note: the value of the gs segment register is ignored in 64-bit mode
        // and only the value in GS_BASE is considered. Thus it is "nice" to load
        // the null descriptor before so that it is clear that gs segment register is unused.
        load_gs(gdt::NULL_SEL);

        wrmsr(IA32_GS_BASE, ptr as usize as u64);
    }
}

pub fn get() -> &'static RefCell<Local> {
    #[cfg(debug_assertions)]
    {
        let gs_val = gs();
        if gs_val.index() != 0 {
            panic!("used local::get() before local::init()");
        }
    }

    unsafe {
        let addr = gs_deref!(0) as usize;
        let ptr = addr as *mut LocalWrapper;
        let wrapper = &*ptr;
        &wrapper.local
    }
}
