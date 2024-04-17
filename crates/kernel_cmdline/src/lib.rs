#![no_std]

mod parser;

pub use parser::KernelCommandLineParser;

#[derive(Debug)]
pub struct KernelCommandLine {
    pub welcome: Option<()>,
    pub use_reloc: Option<bool>,
    pub stack_size: Option<usize>,
    pub initial_heap_size: Option<usize>,
}

impl KernelCommandLine {
    pub fn verfy(&self) {
        assert!(
            self.stack_size().next_multiple_of(memory::PAGE_SIZE) == self.stack_size(),
            "stack_size must be page-aligned"
        );

        assert!(
            self.initial_heap_size().next_multiple_of(memory::PAGE_SIZE)
                == self.initial_heap_size(),
            "initial_heap_size must be page-aligned"
        );
    }

    pub fn welcome(&self) -> bool {
        self.welcome.is_some()
    }

    pub fn use_reloc(&self) -> bool {
        self.use_reloc.unwrap_or(true)
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size.unwrap_or(16 * 0x1000)
    }

    pub fn initial_heap_size(&self) -> usize {
        self.initial_heap_size.unwrap_or(1 * 0x1000000)
    }
}
