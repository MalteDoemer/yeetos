//! The 64-bit kernel uses 4 Level paging.
//! The (current) page tables will be accessed using the recursive mapping technique.
//!
use core::ops::{Deref, DerefMut};

use log::info;
use memory::paging::{Level4, Table};
use spin::{Mutex, MutexGuard};

/// This is a pointer to the recursive mapped pml4t.
/// In order to access it the page_lock must be held.
const P4: *mut Table<Level4> = 0xffffffff_fffff000 as *mut _;

/// The `PAGE_LOCK` must be held for any access to the recursive mapping area and other paging operations.
static PAGE_LOCK: Mutex<()> = Mutex::new(());

pub struct PageMapLevelFourGuard<'a> {
    table: &'a mut Table<Level4>,
    guard: MutexGuard<'a, ()>,
}

impl<'a> Deref for PageMapLevelFourGuard<'a> {
    type Target = Table<Level4>;

    fn deref(&self) -> &Self::Target {
        self.table
    }
}

impl<'a> DerefMut for PageMapLevelFourGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.table
    }
}

/// Obtain a reference to the recursively mapped pml4t.
/// This function locks the `PAGE_LOCK` and returns a guard object
/// that releases the lock when dropped.
pub fn get_page_map_level_four() -> PageMapLevelFourGuard<'static> {
    let guard = PAGE_LOCK.lock();
    let table = unsafe { &mut *P4 };

    PageMapLevelFourGuard { table, guard }
}

pub fn test() {
    let p4 = get_page_map_level_four();

    let pdpt = unsafe { p4.next_table(0).unwrap() };
    let pdt = unsafe { pdpt.next_table(0).unwrap() };

    let entry = &pdt[0];

    info!("{:?}", entry.usage());
}
