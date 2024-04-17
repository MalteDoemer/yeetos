use core::{
    alloc::{GlobalAlloc, Layout},
    mem::MaybeUninit,
    ptr::NonNull,
};

use boot_info::BootInfoHeader;
use linked_list_allocator::Heap;
use memory::{to_lower_half, Frame, Page, PhysicalRange, VirtualRange};
use spin::{Mutex, Once};

#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator::empty();

static INIT: Once<()> = Once::new();

struct HeapAllocator {
    inner: Mutex<Heap>,
}

impl HeapAllocator {
    pub const fn empty() -> Self {
        Self {
            inner: Mutex::new(Heap::empty()),
        }
    }

    pub fn init(&self, mem: &'static mut [MaybeUninit<u8>]) {
        self.inner.lock().init_from_slice(mem);
    }

    // pub fn is_init(&self) -> bool {
    //     self.inner.lock().free()
    // }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner
            .lock()
            .allocate_first_fit(layout)
            .ok()
            .map_or(core::ptr::null_mut(), |nn| nn.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            self.inner
                .lock()
                .deallocate(NonNull::new_unchecked(ptr), layout);
        }
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("failed to allocate: {:?}", layout);
}

/// Initialize the kernel heap.
pub fn init(boot_info: &BootInfoHeader) {
    INIT.call_once(|| {
        let range =
            get_heap_range(boot_info).expect("not enough memory available for the heap allocator");

        // Safety: range is assumed to be accessible memory with a 'static lifetime.
        unsafe {
            init_unchecked(range);
        }
    });
}

/// Initialize the kernel heap from memory in `heap_memory`.
///
/// # Safety
/// - All memory in the range of `heap_memory` must be safe to access for
/// the `'static` lifetime.
/// - This function must only be called once.
unsafe fn init_unchecked(heap_memory: VirtualRange) {
    let heap_start: usize = heap_memory.start().to_addr().to_inner();
    let heap_end: usize = heap_memory.end().to_addr().to_inner();

    let heap_size = heap_end - heap_start;

    // Safety:
    // The heap memory is only ever used here and nowhere else.
    let heap_mem =
        unsafe { core::slice::from_raw_parts_mut(heap_start as *mut MaybeUninit<u8>, heap_size) };

    ALLOCATOR.init(heap_mem);
}

/// This function calculates the amount of heap memory needed by the kernel
/// and returns a `VirtualRange` from the end of the kernel image with the
/// calcualted and checked amount of heap memory.
///
/// # Returns
/// - `Some(range)` the virtual range for the heap memory
/// - `None` if the required memory is not available
/// # Note
/// After the initialization phase the kernel should no longer use the heap
/// allocator. The heap is only there to allocate certain structures during
/// init. Thus we can theoretically calculate the exact amount of memory needed.
/// But for now its just a hardcoded value to get things going.
pub fn get_heap_range(boot_info: &BootInfoHeader) -> Option<VirtualRange> {
    let heap_start = boot_info.kernel_image_info.end();
    let heap_size = 4 * 1024 * 1024;
    let heap_end = heap_start + heap_size;

    let heap_virt_range =
        VirtualRange::new_diff(Page::new(heap_start), Page::new(heap_end.page_align_up()));

    let heap_start_pyhs = to_lower_half(heap_start).to_phys();
    let heap_end_phys = to_lower_half(heap_end).to_phys();
    let heap_phys_range = PhysicalRange::new_diff(
        Frame::new(heap_start_pyhs),
        Frame::new(heap_end_phys.frame_align_up()),
    );

    // Now we check if the memory we need is actually there.
    if boot_info.memory_map.is_usable(heap_phys_range) {
        Some(heap_virt_range)
    } else {
        None
    }
}
