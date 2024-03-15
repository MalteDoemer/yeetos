#[repr(C)]
pub struct KernelImageInfo {
    /// Start address of the kernel stack area
    pub kernel_stacks_start: VirtAddr,
    /// Size of a single kernel stack in bytes.
    pub kernel_stack_size: usize,
    /// Start address of the kernel code segment.
    pub kernel_code_start: VirtAddr,
    /// Size of the kernel code segment in bytes.
    pub kernel_code_size: usize,
    /// Start address of the kernel rodata segment.
    pub kernel_rodata_start: VirtAddr,
    /// Size of the kernel rodata segment in bytes.
    pub kernel_rodata_size: usize,
    /// Start address of the kernel data segment.
    pub kernel_data_start: VirtAddr,
    /// Size of the kernel data segment in bytes.
    pub kernel_data_size: usize,
}
