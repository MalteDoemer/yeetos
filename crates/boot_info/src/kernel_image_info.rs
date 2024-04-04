use memory::{VirtAddr, VirtualRange};

pub struct KernelImageInfo {
    pub stack: VirtualRange,
    pub rodata: Option<VirtualRange>,
    pub code: VirtualRange,
    pub relro: Option<VirtualRange>,
    pub data: Option<VirtualRange>,
}

impl KernelImageInfo {
    pub const fn empty() -> Self {
        KernelImageInfo {
            stack: VirtualRange::zero(),
            rodata: None,
            code: VirtualRange::zero(),
            relro: None,
            data: None,
        }
    }

    pub fn start(&self) -> VirtAddr {
        self.stack.start().to_addr()
    }

    pub fn image_base(&self) -> VirtAddr {
        self.stack.end().to_addr()
    }

    pub fn end(&self) -> VirtAddr {
        if let Some(data) = self.data {
            data.end().to_addr()
        } else if let Some(relro) = self.relro {
            relro.end().to_addr()
        } else {
            self.code.end().to_addr()
        }
    }

    pub fn size_in_bytes(&self) -> usize {
        self.end() - self.start()
    }
}
