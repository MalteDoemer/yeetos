use crate::mm::virtual_bump_allocator::VirtualBumpAllocator;
use crate::mm::{get_initial_kernel_regions, InitialKernelRegion};
use alloc::vec::Vec;
use boot_info::BootInfoHeader;
use memory::virt::{Page, VirtAddr, VirtualRange, VirtualRangeAllocator};
use memory::{KERNEL_BASE, KERNEL_END};
use spin::Mutex;

static GLOBAL_ALLOC: AllocatorImpl = AllocatorImpl::new();

struct AllocatorImpl {
    inner: Mutex<VirtualBumpAllocator>,
}

impl AllocatorImpl {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(VirtualBumpAllocator::new(kernel_virtual_range())),
        }
    }

    pub fn init(&self, _boot_info: &BootInfoHeader) {
        // nothing to do for now
    }

    pub fn alloc(&self, num_pages: usize, alignment: usize) -> Option<VirtualRange> {
        self.inner.lock().alloc(num_pages, alignment)
    }

    pub fn alloc_specific(&self, range: VirtualRange) -> Option<()> {
        self.inner.lock().alloc_specific(range)
    }

    pub fn dealloc(&self, range: VirtualRange) -> Option<()> {
        self.inner.lock().dealloc(range)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct KernelVirtualAllocator;

impl VirtualRangeAllocator for KernelVirtualAllocator {
    fn range(&self) -> VirtualRange {
        kernel_virtual_range()
    }

    fn alloc(&mut self, num_pages: usize, alignment: usize) -> Option<VirtualRange> {
        GLOBAL_ALLOC.alloc(num_pages, alignment)
    }

    fn alloc_specific(&mut self, range: VirtualRange) -> Option<()> {
        GLOBAL_ALLOC.alloc_specific(range)
    }

    fn dealloc(&mut self, range: VirtualRange) -> Option<()> {
        GLOBAL_ALLOC.dealloc(range)
    }
}

const fn kernel_virtual_range() -> VirtualRange {
    let start = VirtAddr::new(KERNEL_BASE);
    let end = VirtAddr::new(KERNEL_END);
    VirtualRange::new(Page::new(start), Page::new(end))
}

pub(super) fn init(boot_info: &BootInfoHeader) {
    GLOBAL_ALLOC.init(boot_info);

    let regions = get_initial_kernel_regions(&boot_info.memory_map, &boot_info.kernel_image_info)
        .expect("unable to obtain initial kernel regions");

    let merged = merge_initial_kernel_regions(regions);

    for region in merged {
        KernelVirtualAllocator
            .alloc_specific(region)
            .expect("unable to allocate virtual address space for initial kernel regions")
    }
}

fn merge_initial_kernel_regions(mut regions: Vec<InitialKernelRegion>) -> Vec<VirtualRange> {
    assert!(!regions.is_empty());
    regions.sort_unstable_by_key(|region| region.virt_range.start_addr().to_inner());

    let mut res = Vec::new();
    res.push(regions[0].virt_range);

    for region in regions.iter().skip(1) {
        let last_idx = res.len() - 1;

        let last = res[last_idx];
        let current = region.virt_range;

        if can_merge(last, current) {
            let merged = merge(last, current);
            res[last_idx] = merged;
        } else {
            res.push(region.virt_range);
        }
    }

    res
}

fn merge(left: VirtualRange, right: VirtualRange) -> VirtualRange {
    left.union_with(right)
}

fn can_merge(left: VirtualRange, right: VirtualRange) -> bool {
    left.end() == right.start()
}
