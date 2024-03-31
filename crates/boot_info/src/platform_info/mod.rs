pub mod pc_x86;

use pc_x86::PCx86Info;
pub enum PlatformInfo {
    None,
    PCX86(PCx86Info),
}
