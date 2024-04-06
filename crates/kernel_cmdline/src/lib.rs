#![no_std]

mod parser;

pub use parser::KernelCommandLineParser;

#[derive(Debug)]
pub struct KernelCommandLine {
    welcome: Option<()>,
    kernel_use_reloc: Option<bool>,
    kernel_stack_size: Option<usize>,
}

impl KernelCommandLine {
    pub fn welcome(&self) -> bool {
        self.welcome.is_some()
    }

    pub fn kernel_use_reloc(&self) -> bool {
        self.kernel_use_reloc.unwrap_or(true)
    }

    pub fn kernel_stack_size(&self) -> Option<usize> {
        self.kernel_stack_size
    }
}
