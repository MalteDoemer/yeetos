use core::{
    alloc::{GlobalAlloc, Layout},
    mem::MaybeUninit,
    ptr::{addr_of_mut, null_mut, NonNull},
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

const HEAP_SIZE: usize = 16 * 4096;

static mut HEAP_POOL: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::new(0); HEAP_SIZE];

pub fn init() {
    let slice = unsafe { &mut *addr_of_mut!(HEAP_POOL) };
    ALLOCATOR.init(slice);
}
