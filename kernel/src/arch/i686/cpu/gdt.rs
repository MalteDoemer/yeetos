use memory::VirtAddr;
use x86::{
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
pub const KERNEL_CPU_LOCAL_DATA_SEL: SegmentSelector = SegmentSelector::new(3, Ring::Ring0);
pub const USER_CODE_SEL: SegmentSelector = SegmentSelector::new(4, Ring::Ring3);
pub const USER_DATA_SEL: SegmentSelector = SegmentSelector::new(5, Ring::Ring3);
pub const TSS_SEL: SegmentSelector = SegmentSelector::new(6, Ring::Ring0);

#[repr(C, align(8))]
pub struct GlobalDescriptorTable {
    null: Descriptor,
    kernel_code: Descriptor,
    kernel_data: Descriptor,
    kernel_cpu_local_data: Descriptor,
    user_code: Descriptor,
    user_data: Descriptor,
    tss_desc: Descriptor,
}

impl GlobalDescriptorTable {
    pub fn new() -> Self {
        let kernel_code =
            DescriptorBuilder::code_descriptor(0, 0xFFFFF, CodeSegmentType::ExecuteRead)
                .limit_granularity_4kb()
                .dpl(Ring::Ring0)
                .present()
                // .avl()
                .db()
                .finish();

        let kernel_data =
            DescriptorBuilder::data_descriptor(0, 0xFFFFF, DataSegmentType::ReadWrite)
                .limit_granularity_4kb()
                .dpl(Ring::Ring0)
                .present()
                .avl()
                .db()
                .finish();

        let kernel_cpu_local_data =
            DescriptorBuilder::data_descriptor(0, 0, DataSegmentType::ReadWrite)
                .dpl(Ring::Ring0)
                .present()
                .avl()
                .db()
                .finish();

        let user_code =
            DescriptorBuilder::code_descriptor(0, 0xFFFFF, CodeSegmentType::ExecuteRead)
                .limit_granularity_4kb()
                .dpl(Ring::Ring3)
                .present()
                .avl()
                .db()
                .finish();

        let user_data = DescriptorBuilder::data_descriptor(0, 0xFFFFF, DataSegmentType::ReadWrite)
            .limit_granularity_4kb()
            .dpl(Ring::Ring3)
            .present()
            .avl()
            .db()
            .finish();

        let tss_desc =
            <DescriptorBuilder as GateDescriptorBuilder<u32>>::tss_descriptor(0, 0, true)
                .present()
                .finish();

        GlobalDescriptorTable {
            null: Descriptor::NULL,
            kernel_code,
            kernel_data,
            kernel_cpu_local_data,
            user_code,
            user_data,
            tss_desc,
        }
    }

    pub fn set_tss_desc(&mut self, addr: VirtAddr, size_in_bytes: usize) {
        let base = addr.to_inner() as u32;
        let limit = size_in_bytes as u32;
        self.tss_desc.set_base_limit(base, limit);
    }

    pub fn set_cpu_local(&mut self, addr: VirtAddr, size_in_bytes: usize) {
        let base = addr.to_inner() as u32;
        let limit = size_in_bytes as u32;
        self.kernel_cpu_local_data.set_base_limit(base, limit);
    }

    pub unsafe fn load(&self) {
        let ptr = DescriptorTablePointer::<Self>::new(self);
        unsafe {
            lgdt(&ptr);
        }
    }
}
