use memory::phys::PhysicalRange;
use uefi::table::boot::BootServices;

use crate::arch;

pub fn prepare(boot_services: &BootServices) {
    arch::paging::prepare(boot_services);
}

pub fn get_kernel_page_tables_range() -> PhysicalRange {
    arch::paging::get_kernel_page_tables_range()
}

pub fn activate() {
    arch::paging::activate();
}
