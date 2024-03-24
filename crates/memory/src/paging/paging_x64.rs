use core::marker::PhantomData;

use bitflags::bitflags;
use zeroize::Zeroize;

use crate::{AccessFlags, Frame, Page, PAGE_TABLE_ENTRIES};

pub trait TableLevel {}

pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

/// Level1 represents the page table (PT).
pub enum Level1 {}

/// Level2 represents the page directory (PD).
pub enum Level2 {}

/// Level3 represents the page directory pointer table (PDPT).
pub enum Level3 {}

/// Level4 represents the page map level 4 table (PML4T).
pub enum Level4 {}

impl TableLevel for Level1 {}

impl TableLevel for Level2 {}

impl TableLevel for Level3 {}

impl TableLevel for Level4 {}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

/// This struct represents any page table entry. (for all levels)
#[repr(transparent)]
#[derive(Zeroize)]
pub struct Entry(u64);

/// On x86 there are 3 bits in the page table entry available for system software.
/// We use these bits to specify the usage of an entry.
#[repr(u8)]
#[derive(Debug)]
#[non_exhaustive]
pub enum EntryUsage {
    /// Entry is not backed by physical memory.
    Empty = 0,
    /// Entry is backed by physical memory.
    Normal = 1,
}

/// A `Table` represents any of the x86_64 paging tables.
/// The type argument `L` defines which paging structure it referes to.
/// All `Table` structs have a size of 4KiB.
#[repr(transparent)]
#[derive(Zeroize)]
pub struct Table<L: TableLevel> {
    entries: [Entry; PAGE_TABLE_ENTRIES],
    phantom: PhantomData<L>,
}

bitflags! {
    pub struct EntryFlags: u64 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USER =            1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const NO_EXECUTE =      1 << 63;
    }
}

impl EntryUsage {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }
}

impl Entry {
    const USAGE_SHIFT: u64 = 9;
    const USAGE_MASK: u64 = 0b111 << Self::USAGE_SHIFT;
    const FRAME_MASK: u64 = 0x000fffff_fffff000;

    /// Creates a entry that is not present and not backed by physical memory.
    pub const fn empty() -> Self {
        Entry(0)
    }

    /// Set the usage of this entry.
    ///
    /// # Note
    /// The usage bits are "user defined" i.e. the hardware
    /// ignores them and they are only used by the software.
    pub fn set_usage(&mut self, usage: EntryUsage) {
        // clear all usage bits
        let val = self.0 & !Self::USAGE_MASK;

        // shift the usage bits to the correct position
        let bts = ((usage as u8) as u64) << Self::USAGE_SHIFT;

        // assign the usage bits to self.0
        self.0 = val | bts;
    }

    /// Set the flags of this entry.
    pub fn set_flags(&mut self, flags: EntryFlags) {
        // clear all flag bits
        let val = self.0 & !EntryFlags::all().bits();
        let bts = flags.bits();

        // assign the flag bits to self.0
        self.0 = val | bts;
    }

    /// Sets the physical address that this entry is pointing to.
    pub fn set_pointing_addr(&mut self, addr: u64) {
        assert!(addr & Self::FRAME_MASK == addr);
        let val = self.0 & !Self::FRAME_MASK;
        let bts = addr & Self::FRAME_MASK;
        self.0 = val | bts;
    }

    /// Get the usage of this entry.
    pub fn usage(&self) -> EntryUsage {
        let bits = ((self.0 & Self::USAGE_MASK) >> Self::USAGE_SHIFT) as u8;

        match bits {
            0 => EntryUsage::Empty,
            1 => EntryUsage::Normal,
            _ => panic!("invalid usage bits in page table entry"),
        }
    }

    /// Get the flags of this entry.
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    /// Get the physical address this entry is pointing to.
    pub fn pointing_addr(&self) -> u64 {
        self.0 & Self::FRAME_MASK
    }

    /// Checks if this entry has the present flag set.
    pub fn is_present(&self) -> bool {
        self.flags().contains(EntryFlags::PRESENT)
    }

    /// Checks if this entry is marked readable.
    pub fn is_readable(&self) -> bool {
        match self.usage() {
            EntryUsage::Empty => false,
            EntryUsage::Normal => self.is_present(),
        }
    }

    /// Checks if this entry is marked writable.
    pub fn is_writable(&self) -> bool {
        match self.usage() {
            EntryUsage::Empty => false,
            EntryUsage::Normal => self.is_present() && self.flags().contains(EntryFlags::WRITABLE),
        }
    }

    /// Checks if this entry is marked executable.
    pub fn is_executable(&self) -> bool {
        match self.usage() {
            EntryUsage::Empty => false,
            EntryUsage::Normal => {
                self.is_present() && !self.flags().contains(EntryFlags::NO_EXECUTE)
            }
        }
    }

    /// Creates a normal entry pointing to `frame` with flags corresponding to `access`.
    /// If `access` is `AccessFlags::empty()` the entry will not be present.
    /// The type of the entry will be `EntryUsage::Normal`.
    pub fn normal_entry(frame: Frame, access: AccessFlags) -> Self {
        let mut entry = Self::empty();
        entry.set_pointing_addr(frame.to_addr().to_inner());
        entry.set_usage(EntryUsage::Normal);

        let mut flags = EntryFlags::empty();

        if !access.is_empty() {
            flags.insert(EntryFlags::PRESENT);
        }

        if access.contains(AccessFlags::WRITE) {
            flags.insert(EntryFlags::WRITABLE);
        }

        if !access.contains(AccessFlags::EXEC) {
            flags.insert(EntryFlags::NO_EXECUTE);
        }

        entry.set_flags(flags);

        entry
    }

    /// Creates a entry that points to anther paging table located at `frame`.
    pub fn table_entry(frame: Frame) -> Self {
        let mut entry = Self::empty();
        entry.set_flags(EntryFlags::PRESENT | EntryFlags::WRITABLE);
        entry.set_pointing_addr(frame.to_addr().to_inner());
        entry
    }
}

impl<L: TableLevel> core::ops::Index<usize> for Table<L> {
    type Output = Entry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L: TableLevel> core::ops::IndexMut<usize> for Table<L> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<L: HierarchicalLevel> Table<L> {
    /// This function calculates the address of a next table using the
    /// recursive mapping technique.
    ///
    /// # Safety
    /// This function can only return a reliable address if `self` is a
    /// table using recursive mapping and recursive mapping is properly set up.
    unsafe fn next_table_address_unchecked(&self, index: usize) -> usize {
        ((self as *const _ as usize) << 9) | (index << 12)
    }

    /// Calculates the virtual address of the table at `index`.
    /// # Returns
    /// `Some(addr)` when the entry is present and `None` otherwise.
    ///
    /// # Note
    /// For now this function assumes that only 4KiB pages are used
    /// i.e. no Huge Pages at all.
    ///
    /// # Safety
    /// This function can only return a reliable address if `self` is a
    /// table using recursive mapping and recursive mapping is properly set up.
    pub unsafe fn next_table_address(&self, index: usize) -> Option<usize> {
        let flags = self[index].flags();

        assert!(!flags.contains(EntryFlags::HUGE_PAGE));

        if flags.contains(EntryFlags::PRESENT) {
            Some(unsafe { self.next_table_address_unchecked(index) })
        } else {
            None
        }
    }

    /// This function returns a readonly reference of the table at `index` given that it exists.
    ///
    /// # Safety
    /// - this function relies on recursive mapping
    /// - rusts reference semantics need to be upheld i.e. read-access to the recursive mapping area
    pub unsafe fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        let addr = unsafe { self.next_table_address(index) };
        addr.map(|addr| unsafe { &*(addr as *const _) })
    }

    /// This function returns a mutable reference of the table at `index` given that it exists.
    ///
    /// # Safety
    /// - this function relies on recursive mapping
    /// - rusts reference semantics need to be upheld i.e. exclusive-access to the recursive mapping area
    pub unsafe fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        let addr = unsafe { self.next_table_address(index) };
        addr.map(|addr| unsafe { &mut *(addr as *mut _) })
    }
}

impl Table<Level4> {
    pub fn get_table_indices(page: Page) -> (usize, usize, usize, usize) {
        let addr = page.to_addr().to_inner();

        let p4 = (addr >> 39) & 0x1ff;
        let p3 = (addr >> 30) & 0x1ff;
        let p2 = (addr >> 21) & 0x1ff;
        let p1 = (addr >> 12) & 0x1ff;

        (p4, p3, p2, p1)
    }
}
