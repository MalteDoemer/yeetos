pub mod pc_x86;

#[repr(C)]
pub enum PlatformInfo {
    PCX86(PCx86Info),
}
