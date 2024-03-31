pub enum Rsdp {
    V1(RsdpV1),
    V2(RsdpV2),
}

#[repr(C, packed)]
pub struct RsdpV1 {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_addr: u32,
}

#[repr(C, packed)]
pub struct RsdpV2 {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_addr: u32,
    pub length: u32,
    pub xsdt_addr: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3],
}

pub struct PCx86Info {
    pub rsdp: Rsdp,
}
