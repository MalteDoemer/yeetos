use boot_info::BootInfoHeader;
use spin::Once;

mod frame_bump_allocator;
mod frame_global_allocator;

use crate::arch;
pub use frame_global_allocator::GlobalFrameAllocator;

static INIT: Once<()> = Once::new();

pub fn init(boot_info: &BootInfoHeader) {
    INIT.call_once(|| {
        frame_global_allocator::init(boot_info);
    });

    arch::paging::init(boot_info);
}
