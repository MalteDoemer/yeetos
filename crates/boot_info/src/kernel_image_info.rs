use memory::{VirtAddr, VirtualRange};

pub struct KernelImageInfo {
    pub stack: VirtualRange,
    pub rodata: VirtualRange,
    pub code: VirtualRange,
    pub relro: VirtualRange,
    pub data: VirtualRange,
}

impl KernelImageInfo {
    pub const fn empty() -> Self {
        KernelImageInfo {
            stack: VirtualRange::zero(),
            rodata: VirtualRange::zero(),
            code: VirtualRange::zero(),
            relro: VirtualRange::zero(),
            data: VirtualRange::zero(),
        }
    }

    pub fn start(&self) -> VirtAddr {
        self.stack.start().to_addr()
    }

    pub fn end(&self) -> VirtAddr {
        self.data.end().to_addr()
    }

    pub fn size_in_bytes(&self) -> usize {
        self.end() - self.start()
    }
}
