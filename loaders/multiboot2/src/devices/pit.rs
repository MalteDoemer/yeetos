use x86::io::outb;

use super::io_delay;

/// Nanoseconds per pit-tick
pub const NANO_SECONDS_PER_PIT: u64 = 2250285;
/// Frequency divider
const PIT_DIVISOR: u16 = 2685;

const PIT_PORT: u16 = 0x40;

struct Pit(u16);

impl Pit {
    const CHAN0_PORT: u16 = 0;
    const CMD_PORT: u16 = 3;

    /// # Safety
    /// Performs an IO write
    pub unsafe fn write_cmd(&self, cmd: u8) {
        unsafe {
            outb(self.0 + Self::CMD_PORT, cmd);
        }
    }

    /// # Safety
    /// Performs an IO write
    pub unsafe fn write_data(&self, data: u8) {
        unsafe {
            outb(self.0 + Self::CHAN0_PORT, data);
        }
    }
}

pub fn init() {
    let pit = Pit(PIT_PORT);

    // Safety:
    // The ports of the PIT are safe to access.
    unsafe {
        // select channel 0
        pit.write_cmd(0x35);
        io_delay();

        // set divisor
        pit.write_data((PIT_DIVISOR & 0xFF) as u8);
        io_delay();
        pit.write_data((PIT_DIVISOR >> 8) as u8);
    }
}
