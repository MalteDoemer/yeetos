use memory::phys::PhysAddr;
use memory::{KERNEL_BASE, PAGE_TABLE_ENTRIES};

mod init;

pub use init::init;

const KERNEL_P4_START_IDX: usize = (KERNEL_BASE >> 39) & 0x1FF;
const KERNEL_P4_END_IDX: usize = PAGE_TABLE_ENTRIES - 1;
const NUM_KERNEL_P3_TABLES: usize = KERNEL_P4_END_IDX - KERNEL_P4_START_IDX;

/// This global variable holds the physical address of the PML4T that is used during initialization
/// until the PML4T's are managed by the process manager / scheduler.
///
/// # Safety
/// This variable is initialized once during init_once() and is immutable after that.
/// Thus, any read access to `INITIAL_P4_ADDR` after init_once() has completed is safe.
static mut INITIAL_P4_ADDR: PhysAddr = PhysAddr::zero();

/// This global array holds the physical addresses of the kernel PDPT's. These tables are allocated
/// during init_once() and we allocate enough PDPT's to completely map the kernel's virtual
/// address space. We do this, so we can later share the kernel address space between processes by
/// simply copying all kernel PDPT entries into the PML4T of a process. Thus, when once process
/// modifies the kernel address space it will become immediately visible to all other processes.
static mut KERNEL_P3_ADDRS: [PhysAddr; NUM_KERNEL_P3_TABLES] =
    [PhysAddr::zero(); NUM_KERNEL_P3_TABLES];

