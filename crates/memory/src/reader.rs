use crate::VirtAddr;

pub struct MemoryReader {
    current: usize,
}

impl MemoryReader {
    /// Creates a new `MemoryReader` starting at `addr`.
    pub fn new(addr: VirtAddr) -> Self {
        Self {
            current: addr.to_inner(),
        }
    }

    /// Returns the current address of the `MemoryReader`.
    pub fn addr(&self) -> VirtAddr {
        VirtAddr::new(self.current)
    }

    /// Returns a pointer to the current address.
    ///
    /// ### Safety
    /// This function is not `unsafe` since derefrencing
    /// the returned pointer is already unsafe.
    pub fn as_ptr<T>(&self) -> *const T {
        self.current as *const T
    }

    /// Returns a mutable pointer to the current address.
    ///
    /// ### Safety
    /// This function is not `unsafe` since derefrencing
    /// the returned pointer is already unsafe.
    pub fn as_ptr_mut<T>(&mut self) -> *mut T {
        self.current as *mut T
    }

    /// Skips `size` amount of bytes.
    /// Returns `None` if the addition would overflow.
    pub fn skip_checked(&mut self, size: usize) -> Option<()> {
        self.current.checked_add(size).map(|_| ())
    }

    /// Skips `size` amount of bytes.
    ///
    /// ### Panics
    /// Panics based on the `overflow-checks` setting (`true` for dev and `false` for release)
    pub fn skip(&mut self, size: usize) {
        self.current += size
    }

    /// Reads a `T` from the current address.
    ///
    /// ### Safety
    /// This is
    pub unsafe fn read<T: Copy>(&mut self) -> T {
        let res = *self.as_ptr_mut();
        self.skip(core::mem::size_of::<T>());
        res
    }

    /// Reads a `T` from the current address.
    ///
    /// ### Safety
    /// This is
    pub unsafe fn read_checked<T: Copy>(&mut self) -> Option<T> {
        if self
            .current
            .checked_add(core::mem::size_of::<T>())
            .is_none()
        {
            return None;
        }

        let res = *self.as_ptr_mut();

        self.skip(core::mem::size_of::<T>());
        
        Some(res)
    }
}
