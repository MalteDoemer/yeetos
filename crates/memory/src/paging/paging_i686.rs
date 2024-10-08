use core::marker::PhantomData;

use bitflags::bitflags;
use zeroize::Zeroize;

use crate::{phys::PhysAddr, virt::Page, AccessFlags, PAGE_TABLE_ENTRIES};

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntryUsage {
    None = 0,
    Page = 1,
    Table = 2,
    Reserved1 = 3,
    Reserved2 = 4,
    Reserved3 = 5,
    Reserved4 = 6,
    Reserved5 = 7,
}

#[repr(transparent)]
#[derive(Clone, Copy, Zeroize)]
pub struct Entry(u32);

bitflags! {
    pub struct EntryFlags: u32 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USER =            1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const PAGE_SIZE =       1 << 7;
        const GLOBAL =          1 << 8;
    }
}

impl Entry {
    const USAGE_MASK: u32 = 0xE00;
    const USAGE_SHIFT: u32 = 9;

    const ADDR_MASK: u32 = 0xfffff000;

    // Get the flags of this entry.
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    /// Set the flags of this entry.
    pub fn set_flags(&mut self, flags: EntryFlags) {
        // clear all flag bits
        let val = self.0 & !EntryFlags::all().bits();
        let bts = flags.bits();

        // assign the flag bits to self.0
        self.0 = val | bts;
    }

    /// Get the usage of this entry.
    /// Note: these bits are user defined i.e they are ignored by the hardware.
    pub fn usage(&self) -> EntryUsage {
        let bits = (self.0 & Self::USAGE_MASK) >> Self::USAGE_SHIFT;

        match bits {
            0 => EntryUsage::None,
            1 => EntryUsage::Page,
            2 => EntryUsage::Table,
            3 => EntryUsage::Reserved1,
            4 => EntryUsage::Reserved2,
            5 => EntryUsage::Reserved3,
            6 => EntryUsage::Reserved4,
            7 => EntryUsage::Reserved5,
            _ => panic!("invalid entry usage bits"),
        }
    }

    /// Set the usage of this entry.
    /// Note: these bits are user defined i.e they are ignored by the hardware.
    pub fn set_usage(&mut self, entry_usage: EntryUsage) {
        // clear all entry usage bits
        let val = self.0 & !Self::USAGE_MASK;

        // shift the entry usage bits to the correct position
        let bts = ((entry_usage as u8) as u32) << Self::USAGE_SHIFT;

        // assign the entry usage bits to self.0
        self.0 = val | bts;
    }

    /// Get the physical address this entry is pointing to.
    pub fn addr(&self) -> u32 {
        self.0 & Self::ADDR_MASK
    }

    /// Set the physical address this entry should point to.
    pub fn set_addr(&mut self, addr: u32) {
        // clear all addr bits
        let val = self.0 & !Self::ADDR_MASK;

        // mask the address and ensure it is correctly aligned
        let bts = addr & Self::ADDR_MASK;
        assert!(bts == addr);

        // assign the address to self.0
        self.0 = val | bts;
    }
}

impl Entry {
    /// Creates an empty entry with `EntryUsage::None`
    pub fn empty() -> Self {
        Entry(0)
    }

    /// Creates a entry pointing to a page frame located at `addr`
    /// with access flags according to `access`.
    /// The entry's usage will be `EntryUsage::Page`.
    pub fn page_entry(addr: PhysAddr, access: AccessFlags) -> Self {
        let mut flags = EntryFlags::empty();

        if !access.is_empty() {
            flags.insert(EntryFlags::PRESENT);
        }

        if access.contains(AccessFlags::WRITE) {
            flags.insert(EntryFlags::WRITABLE);
        }

        let mut entry = Entry(0);
        entry.set_addr(addr.to_inner());
        entry.set_usage(EntryUsage::Page);
        entry.set_flags(flags);

        entry
    }

    /// Creates a table entry pointing to the table located at `addr`.
    /// This entry will have a usage of `EntryUsage::Table`.
    pub fn table_entry(addr: PhysAddr) -> Self {
        let mut entry = Entry(0);
        entry.set_flags(EntryFlags::PRESENT | EntryFlags::WRITABLE);
        entry.set_usage(EntryUsage::Table);
        entry.set_addr(addr.to_inner());
        entry
    }
}

pub enum Level {
    Level1,
    Level2,
}

pub trait TableLevel {
    const LEVEL: Level;
}

/// Level1 represents the page table (PT).
pub enum Level1 {}

/// Level2 represents the page directory (PD).
pub enum Level2 {}

impl TableLevel for Level1 {
    const LEVEL: Level = Level::Level1;
}

impl TableLevel for Level2 {
    const LEVEL: Level = Level::Level2;
}

/// A `Table` represents either a PT or a PD
/// The type argument `L` defines which paging structure it refers to.
/// All `Table` structs have a size of 4KiB.
#[repr(transparent)]
#[derive(Zeroize)]
pub struct Table<L: TableLevel> {
    entries: [Entry; PAGE_TABLE_ENTRIES],
    phantom: PhantomData<L>,
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

impl Table<Level2> {
    /// This function calculates the address of a next table using the
    /// recursive mapping technique.
    ///
    /// # Safety
    /// This function can only return a reliable address if `self` is a
    /// table using recursive mapping and recursive mapping is properly set up.
    unsafe fn next_table_address_unchecked(&self, index: usize) -> usize {
        ((self as *const _ as usize) << 10) | (index << 12)
    }

    /// Calculates the virtual address of the table at `index`.
    /// # Returns
    /// - `Some(addr)`if the entry at `index` referes to a table
    /// - `None` otherwise
    ///
    /// # Safety
    /// This function can only return a reliable address if `self` is a
    /// table using recursive mapping and recursive mapping is properly set up.
    unsafe fn next_table_address(&self, index: usize) -> Option<usize> {
        let usage = self[index].usage();

        if let EntryUsage::Table = usage {
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
    pub unsafe fn next_table<'a>(&'a self, index: usize) -> Option<&'a Table<Level1>> {
        let addr = self.next_table_address(index);
        addr.map(|addr| &*(addr as *const _))
    }

    /// This function returns a mutable reference of the table at `index` given that it exists.
    ///
    /// # Safety
    /// - this function relies on recursive mapping
    /// - rusts reference semantics need to be upheld i.e. exclusive-access to the recursive mapping area
    pub unsafe fn next_table_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut Table<Level1>> {
        let addr = self.next_table_address(index);
        addr.map(|addr| &mut *(addr as *mut _))
    }

    /// This function calculates the indices into the page directory and page table for the given address.
    pub fn get_table_indices(page: Page) -> (usize, usize) {
        let addr = page.to_addr().to_inner();

        let p2 = (addr >> 22) & 0x3FF;
        let p1 = (addr >> 12) & 0x3FF;

        (p2, p1)
    }
}
