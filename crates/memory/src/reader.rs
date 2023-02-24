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
    /// Panics based on the `overflow-checks`
    pub fn skip(&mut self, size: usize) {
        self.current += size
    }

    /// Aligns the reader up to the next multiple of `alignment`.
    ///
    /// ### Panics
    /// - Panics if `alignment` is zero
    /// - Panics based on the `overflow-checks`
    pub fn align_up(&mut self, alignment: usize) {
        let aligned = self.current.next_multiple_of(alignment);

        self.current = aligned;
    }

    /// Aligns the reader up to the next multiple of `alignment`.
    ///
    /// Returns `None` if alignment is zero or the operation would overflow.
    pub fn align_up_checked(&mut self, alignment: usize) -> Option<()> {
        let aligned = self.current.checked_next_multiple_of(alignment)?;
        self.current = aligned;
        Some(())
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
