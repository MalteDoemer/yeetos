use memory::VirtAddr;
use x86::{
    bits64::{segmentation::Descriptor64, task::TaskStateSegment},
    dtables::{lgdt, DescriptorTablePointer},
    segmentation::{
        load_cs, load_ds, load_es, load_ss, BuildDescriptor, CodeSegmentType, DataSegmentType,
        Descriptor, DescriptorBuilder, GateDescriptorBuilder, SegmentDescriptorBuilder,
        SegmentSelector,
    },
    Ring,
};

use super::local;

pub const KERNEL_CODE_SEL: SegmentSelector = SegmentSelector::new(1, Ring::Ring0);
pub const KERNEL_DATA_SEL: SegmentSelector = SegmentSelector::new(2, Ring::Ring0);
pub const USER_CODE_SEL: SegmentSelector = SegmentSelector::new(3, Ring::Ring3);
pub const USER_DATA_SEL: SegmentSelector = SegmentSelector::new(4, Ring::Ring3);
pub const TSS_SEL: SegmentSelector = SegmentSelector::new(5, Ring::Ring0);

#[repr(C, align(8))]
pub struct GlobalDescriptorTable {
    null: Descriptor,
    kernel_code: Descriptor,
    kernel_data: Descriptor,
    user_code: Descriptor,
    user_data: Descriptor,
    tss_desc: Descriptor64,
}

impl GlobalDescriptorTable {
    pub fn new() -> Self {
        let kernel_code = DescriptorBuilder::code_descriptor(0, 0, CodeSegmentType::ExecuteRead)
            .present()
            .l()
            .dpl(Ring::Ring0)
            .finish();

        let kernel_data = DescriptorBuilder::data_descriptor(0, 0, DataSegmentType::ReadWrite)
            .present()
            .l()
            .dpl(Ring::Ring0)
            .finish();

        let user_code = DescriptorBuilder::code_descriptor(0, 0, CodeSegmentType::ExecuteRead)
            .present()
            .l()
            .dpl(Ring::Ring3)
            .finish();

        let user_data = DescriptorBuilder::data_descriptor(0, 0, DataSegmentType::ReadWrite)
            .present()
            .l()
            .dpl(Ring::Ring3)
            .finish();

        GlobalDescriptorTable {
            null: Descriptor::NULL,
            kernel_code,
            kernel_data,
            user_code,
            user_data,
            tss_desc: Descriptor64::NULL,
        }
    }

    pub fn set_tss_desc(&mut self, tss_addr: VirtAddr) {
        let base = tss_addr.to_inner() as u64;
        let limit = core::mem::size_of::<TaskStateSegment>() as u64;

        self.tss_desc =
            <DescriptorBuilder as GateDescriptorBuilder<u64>>::tss_descriptor(base, limit, true)
                .present()
                .finish();
    }

    pub unsafe fn load(&self) {
        let ptr = DescriptorTablePointer::<Self>::new(self);
        unsafe {
            lgdt(&ptr);
        }
    }
}

pub(super) fn init_all() {
    let local = local::get().borrow();

    // Safety: it is assumed that the GDT is correctly set up.
    unsafe {
        local.gdt.load();

        // It is important that we don't reload fs or gs here.
        // They are written using the gsbase fsbase msr's.
        load_ss(KERNEL_DATA_SEL);
        load_ds(KERNEL_DATA_SEL);
        load_es(KERNEL_DATA_SEL);

        load_cs(KERNEL_CODE_SEL);
    }
}
