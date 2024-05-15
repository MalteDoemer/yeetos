use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct AccessFlags: usize {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXEC = 1 << 2;

        const READ_WRITE = Self::READ.bits() | Self::WRITE.bits();

        const READ_EXEC = Self::READ.bits() | Self::EXEC.bits();
    }
}
