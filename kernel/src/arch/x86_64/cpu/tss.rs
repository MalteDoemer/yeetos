use x86::{
    bits64::{segmentation::Descriptor64, task::TaskStateSegment},
    segmentation::{BuildDescriptor, DescriptorBuilder, GateDescriptorBuilder},
    task::load_tr,
};

use super::{gdt, local};

pub(super) fn init() {
    let mut local = local::get().borrow_mut();

    // Set up the TSS system descriptor
    let tss_addr = get_tss_addr(&local.tss);
    local.gdt.set_tss_desc(get_tss_desc(tss_addr));

    // See https://wiki.osdev.org/Task_State_Segment on meaning of this value.
    local.tss.iomap_base = core::mem::size_of::<TaskStateSegment>() as u16;

    // Safety: GDT and TSS are assumed to be correctly set up
    unsafe {
        load_tr(gdt::TSS_SEL);
    }
}

fn get_tss_addr(tss: &TaskStateSegment) -> usize {
    tss as *const TaskStateSegment as usize
}

fn get_tss_desc(tss_addr: usize) -> Descriptor64 {
    let base = tss_addr as u64;
    let limit = core::mem::size_of::<TaskStateSegment>() as u64;

    <DescriptorBuilder as GateDescriptorBuilder<u64>>::tss_descriptor(base, limit, true)
        .present()
        .finish()
}
