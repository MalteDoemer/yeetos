use x86::{bits64::task::TaskStateSegment, task::load_tr};

use super::{gdt, local};

pub(super) fn init_all() {
    let mut local = local::get().borrow_mut();

    // Set up the TSS system descriptor
    let tss_addr = &local.tss as *const TaskStateSegment as usize;
    local.gdt.set_tss_desc(tss_addr.into());

    // See https://wiki.osdev.org/Task_State_Segment on meaning of this value.
    local.tss.iomap_base = core::mem::size_of::<TaskStateSegment>() as u16;

    // Safety: GDT and TSS are assumed to be correctly set up
    unsafe {
        load_tr(gdt::TSS_SEL);
    }
}
