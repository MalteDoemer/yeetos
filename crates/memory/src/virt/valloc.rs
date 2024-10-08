use crate::virt::VirtualRange;

pub trait VirtualRangeAllocator {
    /// This function returns the range of virtual memory
    /// that is managed by this allocator.
    fn range(&self) -> VirtualRange;

    /// Allocate a range of virtual memory with a size of at least `num_pages * PAGE_SIZE`
    /// and alignment at least `align`.
    /// # Parameters
    /// - `num_pages` the number of virtual pages the allocated region should have
    /// - `alignment` the requested alignment in pages, must be a power of two
    fn alloc(&mut self, num_pages: usize, alignment: usize) -> Option<VirtualRange>;

    /// Try to mark a specific range of virtual memory as allocated. This function
    /// can be used to "allocate" memory regions that need a specific address such as
    /// the kernel image, memory mapped IO or the zero page.
    /// # Parameters
    /// - `range` the exact range of virtual memory to allocate
    fn alloc_specific(&mut self, range: VirtualRange) -> Option<()>;

    /// Deallocate a range of virtual memory previously obtained by `alloc()`
    fn dealloc(&mut self, range: VirtualRange) -> Option<()>;

    /// Check if `range` belongs to this allocator.
    fn contains(&self, range: VirtualRange) -> bool {
        self.range().contains_range(range)
    }
}
