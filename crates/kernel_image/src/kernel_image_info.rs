use memory::virt::{Page, VirtAddr, VirtualRange};

#[derive(Debug)]
pub struct KernelImageInfo {
    pub stack: VirtualRange,
    pub rodata: Option<VirtualRange>,
    pub code: VirtualRange,
    pub relro: Option<VirtualRange>,
    pub data: Option<VirtualRange>,
    pub heap: VirtualRange,
}

impl KernelImageInfo {
    pub const fn empty() -> Self {
        KernelImageInfo {
            stack: VirtualRange::zero(),
            rodata: None,
            code: VirtualRange::zero(),
            relro: None,
            data: None,
            heap: VirtualRange::zero(),
        }
    }

    pub fn start(&self) -> VirtAddr {
        self.stack.start().to_addr()
    }

    pub fn image_base(&self) -> VirtAddr {
        self.stack.end().to_addr()
    }

    pub fn end(&self) -> VirtAddr {
        self.heap.end().to_addr()
    }

    pub fn size_in_bytes(&self) -> usize {
        self.end() - self.start()
    }

    pub fn to_higher_half(&self) -> Self {
        KernelImageInfo {
            stack: translate_range_to_higher_half(self.stack),
            rodata: translate_optional_range_to_higher_half(self.rodata),
            code: translate_range_to_higher_half(self.code),
            relro: translate_optional_range_to_higher_half(self.relro),
            data: translate_optional_range_to_higher_half(self.data),
            heap: translate_range_to_higher_half(self.heap),
        }
    }
}

fn translate_range_to_higher_half(range: VirtualRange) -> VirtualRange {
    let page = Page::new(range.start_addr().to_higher_half());
    VirtualRange::new(page, range.num_pages())
}

fn translate_optional_range_to_higher_half(range: Option<VirtualRange>) -> Option<VirtualRange> {
    range.map(|rng| translate_range_to_higher_half(rng))
}
