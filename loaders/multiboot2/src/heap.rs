use core::{
    alloc::{GlobalAlloc, Layout},
    mem::MaybeUninit,
    ptr::{null_mut, NonNull},
};

use linked_list_allocator::Heap;
use spin::Mutex;

#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator::empty();

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
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner
            .lock()
            .allocate_first_fit(layout)
            .ok()
            .map_or(null_mut(), |nn| nn.as_ptr())
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

pub fn init() {
    extern "C" {
        pub fn __heap_start();
        pub fn __heap_end();
    }

    let heap_start = __heap_start as usize;
    let heap_end = __heap_end as usize;

    let heap_size = heap_end - heap_start;

    // Safety:
    // The heap memory is only ever used here and nowhere else.
    let heap_mem =
        unsafe { core::slice::from_raw_parts_mut(heap_start as *mut MaybeUninit<u8>, heap_size) };

    ALLOCATOR.init(heap_mem);
}
