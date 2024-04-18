use core::{arch::asm, cell::RefCell, marker::PhantomData, ptr::NonNull};

use alloc::boxed::Box;
use memory::VirtAddr;
use x86::{
    bits32::task::TaskStateSegment,
    segmentation::{load_ds, load_es, load_fs, load_gs, load_ss},
};

use super::gdt::{self, GlobalDescriptorTable};

/// This type can be used to make a struct !Send .
type PhantomUnsend = PhantomData<*mut ()>;

extern "C" {
    /// - load_cs implemented in asm.s -
    /// This function reloads the cs register taking into
    /// account that the kernel uses postition independant code.
    fn load_cs(sel: u32);
}

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

/// Each core/cpu will hold a pointer to a `LocalWrapper` object using the gs
/// segment register. Thus when in kernel mode a `movl %ds:0, %eax` i.e. `mov eax, [gs:0]` will
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

    let local_addr = local.self_ref.as_ptr() as usize;

    local.local.borrow_mut().gdt.set_cpu_local(
        VirtAddr::new(local_addr),
        core::mem::size_of::<LocalWrapper>(),
    );

    // Note:
    // we have to load the gdt right here in order to not get exceptions when using load_gs()
    // Safety:
    // The GDT is assumed to be set up correctly.s
    unsafe {
        local.local.borrow_mut().gdt.load();
    }

    unsafe {
        load_ss(gdt::KERNEL_DATA_SEL);
        load_ds(gdt::KERNEL_DATA_SEL);
        load_es(gdt::KERNEL_DATA_SEL);
        load_fs(gdt::KERNEL_DATA_SEL);

        // This will point to the `local` object
        load_gs(gdt::KERNEL_CPU_LOCAL_DATA_SEL);

        // reload code segment
        load_cs(gdt::KERNEL_CODE_SEL.bits() as u32);
    }

    // Do not deallocate the memory.
    Box::leak(local);
}

pub fn get() -> &'static RefCell<Local> {
    unsafe {
        let addr = gs_deref();
        let ptr = addr as *mut LocalWrapper;
        let wrapper = &*ptr;
        &wrapper.local
    }
}

/// Reads the word at %gs:0
unsafe fn gs_deref() -> usize {
    let gs: u32;
    unsafe {
        asm!("movl %gs:0, {result}", result = out(reg) gs, options(att_syntax));
    }

    gs as usize
}
