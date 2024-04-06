#![no_std]

mod parser;

pub use parser::KernelCommandLineParser;

#[derive(Debug)]
pub struct KernelCommandLine {
    pub welcome: Option<()>,
    pub kernel_use_reloc: Option<bool>,
    pub kernel_stack_size: Option<usize>,
}

impl KernelCommandLine {
    pub fn verfy(&self) {
        assert!(
            self.kernel_stack_size().next_multiple_of(memory::PAGE_SIZE)
                == self.kernel_stack_size(),
            "kernel_stack_size must be page-aligned"
        )
    }

    pub fn welcome(&self) -> bool {
        self.welcome.is_some()
    }

    pub fn kernel_use_reloc(&self) -> bool {
        self.kernel_use_reloc.unwrap_or(true)
    }

    pub fn kernel_stack_size(&self) -> usize {
        self.kernel_stack_size.unwrap_or(16 * 0x1000)
    }
}
