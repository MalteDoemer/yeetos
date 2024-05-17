use memory::{KERNEL_BASE, PAGE_TABLE_ENTRIES};

mod init;

pub use init::init;
use memory::phys::PhysAddr;

const KERNEL_P2_START_IDX: usize = (KERNEL_BASE >> 22) & 0x3FF;
const KERNEL_P2_END_IDX: usize = PAGE_TABLE_ENTRIES - 1;
const NUM_KERNEL_P1_TABLES: usize = KERNEL_P2_END_IDX - KERNEL_P2_START_IDX;

/// This global variable holds the physical address of the PD that is used during initialization
/// until the PD's are managed by the process manager / scheduler.
///
/// # Safety
/// This variable is initialized once during init_once() and is immutable after that.
/// Thus, any read access to `INITIAL_P2_ADDR` after init_once() has completed is safe.
static mut INITIAL_P2_ADDR: PhysAddr = PhysAddr::zero();

/// This global array holds the physical addresses of the kernel PT's. These tables are allocated
/// during init_once() and we allocate enough PT's to completely map the kernel's virtual
/// address space. We do this, so we can later share the kernel address space between processes by
/// simply copying all kernel PT entries into the PD of a process. Thus, when once process
/// modifies the kernel address space it will become immediately visible to all other processes.
static mut KERNEL_P1_ADDRS: [PhysAddr; NUM_KERNEL_P1_TABLES] =
    [PhysAddr::zero(); NUM_KERNEL_P1_TABLES];
