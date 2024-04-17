use memory::{VirtAddr, VirtualRange};

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
}
