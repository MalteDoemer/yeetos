use core::sync::atomic::{AtomicU32, Ordering};

use x86::io::outb;

use crate::idt::IntStackFrame;

use super::io_delay;

/// Frequency divider
pub const PIT_DIVISOR: u16 = 2685;
/// Nanoseconds per pit-tick when using `PIT_DIVISOR` as divisor.
pub const NANO_SECONDS_PER_PIT: u64 = 2_250_285;

/// The standard port of the PIT on x86 based systems
const PIT_PORT: u16 = 0x40;

pub static PIT_TICKS: AtomicU32 = AtomicU32::new(0);

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

pub extern "x86-interrupt" fn pit_interrupt(_frame: IntStackFrame) {
    PIT_TICKS.fetch_add(1, Ordering::SeqCst);
    super::pic::send_eoi(0);
}

/// This function initializes the PIT channel 0 with
/// the divisor set to `PIT_DIVISOR`. This will make
/// the PIT generate IRQ0 every `NANO_SECONDS_PER_PIT` nano seconds.
///
/// In order for the interrupts to be serviced, IRQ0 needs to be unmasked
/// using the PIC, an appropriate interrupt handler needs to be installed
/// using the IDT and interrupts need to be enabled.
pub fn init() {
    let pit = Pit(PIT_PORT);

    // Safety:
    // The ports of the PIT are safe to access.
    unsafe {
        // select channel 0
        pit.write_cmd(0x35);
        io_delay();

        // set divisor
        pit.write_data(PIT_DIVISOR as u8);
        io_delay();
        pit.write_data((PIT_DIVISOR >> 8) as u8);
    }
}
