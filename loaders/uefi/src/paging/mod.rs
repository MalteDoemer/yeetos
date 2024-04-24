use memory::phys::PhysicalRange;
use uefi::table::boot::BootServices;

#[cfg(target_arch = "x86_64")]
mod paging_x86_64;

mod arch {
    #[cfg(target_arch = "x86_64")]
    pub use super::paging_x86_64::*;
}

pub fn prepare(boot_services: &BootServices) {
    arch::prepare(boot_services);
}

pub fn get_kernel_page_tables_range() -> PhysicalRange {
    arch::get_kernel_page_tables_range()
}

pub fn activate() {
    arch::activate();
}
