use memory::VirtAddr;
use x86::{
    bits64::segmentation::Descriptor64,
    dtables::{lgdt, DescriptorTablePointer},
    segmentation::{
        BuildDescriptor, CodeSegmentType, DataSegmentType, Descriptor, DescriptorBuilder,
        GateDescriptorBuilder, SegmentDescriptorBuilder, SegmentSelector,
    },
    Ring,
};

pub const NULL_SEL: SegmentSelector = SegmentSelector::new(0, Ring::Ring0);
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

        let tss_desc =
            <DescriptorBuilder as GateDescriptorBuilder<u64>>::tss_descriptor(0, 0, true)
                .present()
                .finish();

        GlobalDescriptorTable {
            null: Descriptor::NULL,
            kernel_code,
            kernel_data,
            user_code,
            user_data,
            tss_desc,
        }
    }

    pub fn set_tss_desc(&mut self, tss_addr: VirtAddr, size_in_bytes: usize) {
        let base = tss_addr.to_inner() as u64;
        let limit = size_in_bytes as u64;
        self.tss_desc.set_base_limit(base, limit);
    }

    pub unsafe fn load(&self) {
        let ptr = DescriptorTablePointer::<Self>::new(self);
        unsafe {
            lgdt(&ptr);
        }
    }
}
