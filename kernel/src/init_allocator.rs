//! The startup allocator is a simple bump-allocator
//! that provides memory allocation during the initialization phase.
//!

use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

use boot_info::BootInfoHeader;
use memory::{to_higher_half, MemoryMapEntryKind, Page, VirtualRange};
use spin::Mutex;

static INIT_ALLOC: Mutex<Option<InitAllocatorImpl>> = Mutex::new(None);

struct InitAllocatorImpl {
    range: VirtualRange,
    current_addr: usize,
}

impl InitAllocatorImpl {
    fn new(range: VirtualRange) -> Self {
        Self {
            range,
            current_addr: range.start().to_addr().to_inner(),
        }
    }

    fn allocate(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let align = layout.align();
        let size = layout.size();

        let aligned_addr = self
            .current_addr
            .checked_next_multiple_of(align)
            .ok_or(AllocError)?;

        let next_addr = aligned_addr.checked_add(size).ok_or(AllocError)?;

        if next_addr >= self.range.end().to_addr().to_inner() {
            Err(AllocError)
        } else {
            self.current_addr = next_addr;

            let ptr = NonNull::new(aligned_addr as *mut u8)
                .expect("InitAllocator::allocate() produced a null pointer");

            let ptr = NonNull::slice_from_raw_parts(ptr, size);

            Ok(ptr)
        }
    }
}

pub struct InitAllocator;

unsafe impl Allocator for InitAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        INIT_ALLOC.lock().as_mut().expect("msg").allocate(layout)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        panic!("tried to call InitAllocator::deallocate()")
    }
}

pub fn init(boot_info: &BootInfoHeader) {
    let mut mmap = boot_info
        .memory_map
        .entries
        .iter()
        .filter(|entry| !matches!(entry.kind, MemoryMapEntryKind::None))
        .skip_while(|entry| !matches!(entry.kind, MemoryMapEntryKind::KernelImage));

    let _kernel_image_entry = mmap.next().expect("KernelImage missing in memory map");
    let next_free = mmap
        .find(|entry| matches!(entry.kind, MemoryMapEntryKind::Free))
        .expect("no usable memory available after KernelImage");

    let phys_start = next_free.start;
    let phys_end = next_free.end;
    let virt_start = to_higher_half(phys_start.to_virt());
    let virt_end = to_higher_half(phys_end.to_virt());

    let start = Page::new(virt_start.page_align_up());
    let end = Page::new(virt_end.page_align_down());

    let range = VirtualRange::new_diff(start, end);

    let mut guard = INIT_ALLOC.lock();

    match *guard {
        Some(_) => panic!("init_allocator::init() already called before"),
        None => *guard = Some(InitAllocatorImpl::new(range)),
    }
}
