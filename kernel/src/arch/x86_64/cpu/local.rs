use core::{cell::RefCell, marker::PhantomData, ptr::NonNull};

use alloc::boxed::Box;
use memory::VirtAddr;
use x86::{
    bits64::task::TaskStateSegment,
    gs_deref,
    msr::{wrmsr, IA32_GS_BASE},
    segmentation::{gs, load_cs, load_ds, load_es, load_fs, load_gs, load_ss},
    task::load_tr,
};

use super::gdt::{self, GlobalDescriptorTable};

/// This type can be used to make a struct !Send .
type PhantomUnsend = PhantomData<*mut ()>;

/// CPU local data.
pub struct Local {
    /// The processor id (apic id) of this core.
    proc_id: usize,
    /// The GDT for this CPU.
    /// Note: only the tss_desc entry is diffrent, all other fields are the same for all cores.
    gdt: GlobalDescriptorTable,
    /// The TSS which holds the stack pointer for system calls.
    tss: TaskStateSegment,
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

    pub fn proc_id(&self) -> usize {
        self.proc_id
    }

    pub fn gdt(&self) -> &GlobalDescriptorTable {
        &self.gdt
    }

    pub fn tss(&self) -> &TaskStateSegment {
        &self.tss
    }
}

pub(super) fn init(proc_id: usize) {
    let mut wrapper = Box::new(LocalWrapper {
        self_ref: NonNull::dangling(),
        local: RefCell::new(Local::new(proc_id)),
        _phantom: PhantomData,
    });

    wrapper.self_ref = wrapper.as_ref().into();

    let local_base = wrapper.self_ref.as_ptr() as usize;

    let mut local = wrapper.local.borrow_mut();

    // load the gdt and segment registers
    unsafe {
        local.gdt.load();

        load_ss(gdt::KERNEL_DATA_SEL);
        load_ds(gdt::KERNEL_DATA_SEL);
        load_es(gdt::KERNEL_DATA_SEL);

        load_cs(gdt::KERNEL_CODE_SEL);

        // Note: the value of the gs segment register is ignored in 64-bit mode
        // and only the value in GS_BASE is considered. Thus it is "nice" to load
        // the null descriptor before so that it is clear that gs segment register is unused.
        load_gs(gdt::NULL_SEL);

        // Same as gs
        load_fs(gdt::NULL_SEL);

        // Write the GS_BASE model specific register.
        wrmsr(IA32_GS_BASE, local_base as u64);
    }

    // load the tss
    unsafe {
        // Set up the TSS system descriptor
        let tss_addr = VirtAddr::new(&local.tss as *const TaskStateSegment as usize);
        let tss_size = core::mem::size_of::<TaskStateSegment>();

        local.gdt.set_tss_desc(tss_addr, tss_size);

        // See https://wiki.osdev.org/Task_State_Segment on meaning of this value.
        local.tss.iomap_base = core::mem::size_of::<TaskStateSegment>() as u16;

        load_tr(gdt::TSS_SEL);
    }

    drop(local);

    // Do not deallocate the memory for the local object
    Box::leak(wrapper);
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
