use core::{
    cell::RefCell,
    marker::PhantomData,
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::boxed::Box;
use x86::{
    bits64::task::TaskStateSegment,
    gs_deref,
    msr::{wrmsr, IA32_GS_BASE},
};

use super::gdt::GlobalDescriptorTable;

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

pub(super) fn init_all(proc_id: usize) {
    let mut local = Box::new(LocalWrapper {
        self_ref: NonNull::dangling(),
        local: RefCell::new(Local::new(proc_id)),
        _phantom: PhantomData,
    });

    local.self_ref = local.as_ref().into();

    let ptr = Box::leak(local) as *mut LocalWrapper;

    unsafe {
        wrmsr(IA32_GS_BASE, ptr as usize as u64);
    }

    IS_INIT.store(true, Ordering::SeqCst);
}

#[cfg(debug_assertions)]
static IS_INIT: AtomicBool = AtomicBool::new(false);

pub fn get() -> &'static RefCell<Local> {
    #[cfg(debug_assertions)]
    if !IS_INIT.load(Ordering::SeqCst) {
        panic!("used local::get() before local::init_all()");
    }

    unsafe {
        let addr = gs_deref!(0) as usize;
        let ptr = addr as *mut LocalWrapper;
        let wrapper = &*ptr;
        &wrapper.local
    }
}
