pub mod pc_x86;
pub mod uefi;

use pc_x86::PCx86Info;
use uefi::UefiInfo;
pub enum PlatformInfo {
    None,
    PCX86(PCx86Info),
    UEFI(UefiInfo),
}
