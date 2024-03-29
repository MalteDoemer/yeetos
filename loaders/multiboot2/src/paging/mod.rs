//! The 64-bit kernel uses 4 Level paging.
//! The (current) page tables will be accessed using the recursive mapping technique.
//!
//! After gaining control from boot.s only the first 4GiB will be identity mapped using 4MiB pages.
//!
//! Thus we need to "enable" recursive mapping and also "enable" the higher-half mapping.

use log::info;
use memory::paging::{Level4, Table};
use spin::Mutex;

/// This is a pointer to the recursive mapped pml4t.
/// In order to access it the page_lock must be held.
pub const P4: *mut Table<Level4> = 0xffffffff_fffff000 as *mut _;

/// The `PAGE_LOCK` must be held for any access to the recursive mapping area and other paging operations.
pub static PAGE_LOCK: Mutex<()> = Mutex::new(());

pub fn test() {
    let _guard = PAGE_LOCK.lock();
    let pml4t = unsafe { &mut *P4 };

    let pdpt = unsafe { pml4t.next_table(0).unwrap() };
    let pdt = unsafe { pdpt.next_table(0).unwrap() };

    let entry = &pdt[0];

    info!("{:?}", entry.usage());
}
