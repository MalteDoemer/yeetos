use core::{
    alloc::{GlobalAlloc, Layout},
    mem::MaybeUninit,
    ptr::NonNull,
};

use boot_info::BootInfoHeader;
use linked_list_allocator::Heap;
use log::info;
use memory::virt::VirtualRange;
use spin::{Mutex, Once};

#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator::empty();

static INIT: Once<()> = Once::new();

struct HeapAllocator {
    inner: Mutex<Heap>,
}

struct HeapStats {
    used: usize,
    free: usize,
    total: usize,
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

    fn stats(&self) -> HeapStats {
        let heap = self.inner.lock();

        let used = heap.used();
        let free = heap.free();
        let total = heap.size();

        HeapStats { used, free, total }
    }
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
        let range = boot_info.kernel_image_info.heap;

        // Safety: range is assumed to be accessible memory with a 'static lifetime.
        unsafe {
            init_unchecked(range);
        }

        let total = ALLOCATOR.stats().total;
        info!(
            "the kernel heap has a size of {} MiB",
            total / (1024 * 1024)
        );
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
