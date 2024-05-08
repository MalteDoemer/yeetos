use memory::virt::VirtAddr;
use spin::Once;

pub static KERNEL_ENTRY: Once<KernelEntryInfo> = Once::new();

#[derive(Copy, Clone)]
pub struct KernelEntryInfo {
    pub entry_point: VirtAddr,
    pub stacks_start: VirtAddr,
    pub stack_size: usize,
}

extern "C" {
    // This function is implemented in boot.s
    fn jmp_kernel_entry(
        boot_info_ptr: usize,
        processor_id: usize,
        entry_point: usize,
        stack_ptr: usize,
    ) -> !;
}

pub fn make_jump_to_kernel(processor_id: usize, entry: KernelEntryInfo) -> ! {
    let boot_info = crate::boot_info::get_boot_info_addr();

    // calculate stack
    let stack_addr = entry.stacks_start + processor_id * entry.stack_size;

    unsafe {
        jmp_kernel_entry(
            boot_info.to_inner(),
            processor_id,
            entry.entry_point.to_inner(),
            stack_addr.to_inner(),
        )
    };
}
