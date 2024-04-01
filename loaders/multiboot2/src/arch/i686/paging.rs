use core::ops::{Deref, DerefMut};

use memory::paging::{Level2, Table};
use spin::{Mutex, MutexGuard};

/// This is a pointer to the recursive mapped page directory.
/// In order to access it the page_lock must be held and recursive mapping must be set up.
const PD: *mut Table<Level2> = 0xfffff000 as *mut _;

/// The `PAGE_LOCK` must be held for any access to the recursive mapping area and other paging operations.
static PAGE_LOCK: Mutex<()> = Mutex::new(());

extern "C" {
    fn enable_paging();
}

pub struct PageDirectoryGuard<'a> {
    table: &'a mut Table<Level2>,
    guard: MutexGuard<'a, ()>,
}

impl<'a> Deref for PageDirectoryGuard<'a> {
    type Target = Table<Level2>;

    fn deref(&self) -> &Self::Target {
        self.table
    }
}

impl<'a> DerefMut for PageDirectoryGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.table
    }
}

fn get_page_directory() -> PageDirectoryGuard<'static> {
    let guard = PAGE_LOCK.lock();
    let table = unsafe { &mut *PD };

    PageDirectoryGuard { table, guard }
}

pub fn enable_higher_half() {
    unsafe {
        enable_paging();
    }

    let mut pd = get_page_directory();

    // copy entries 0..256 to entries 768..1024
    // to enable higher half mapping
    for i in 0..0x100 {
        let entry = pd[i];
        pd[i + 0x300] = entry;
    }
}
