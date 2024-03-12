//! This module encapsulates the functionality of the PIC
//! interrupt controller. On x86 based pc systems there are
//! always two PIC's present, the primary and the secondary.
//!
//! All functions in this module must only be used after a
//! call to `init()`. After a call to `init()` all IRQ's are
//! masked and need to be manually unmasked if they are wanted.
//!
//! Note: the functions are not marked unsafe for simplicity, but
//! they do need to be run with CPL=0 and only after `init()` has been called.
use x86::io::{inb, outb};

use super::io_delay;

pub const PRIMARY_VECTOR_OFFSET: u8 = 0x20;
pub const SECONDARY_VECTOR_OFFSET: u8 = 0x28;

/// The standard port of the primary PIC on x86 based systems
const PRIMARY_PIC_PORT: u16 = 0x20;
/// The standard port of the secondary PIC on x86 based systems
const SECONDARY_PIC_PORT: u16 = 0xA0;

struct Pic(u16);

impl Pic {
    const CMD_PORT: u16 = 0;
    const DATA_PORT: u16 = 1;

    /// # Safety
    /// Performs an IO write.
    pub unsafe fn write_cmd(&self, cmd: u8) {
        unsafe {
            outb(self.0 + Self::CMD_PORT, cmd);
        }
    }

    /// # Safety
    /// Performs an IO write.
    pub unsafe fn write_data(&self, data: u8) {
        unsafe {
            outb(self.0 + Self::DATA_PORT, data);
        }
    }

    /// # Safety
    /// Performs an IO read.
    pub unsafe fn read_cmd(&self) -> u8 {
        unsafe { inb(self.0 + Self::CMD_PORT) }
    }

    /// # Safety
    /// Performs an IO read.
    pub unsafe fn read_data(&self) -> u8 {
        unsafe { inb(self.0 + Self::DATA_PORT) }
    }
}

fn select_pic(vector: u8) -> (Pic, u8) {
    if vector < 8 {
        (Pic(PRIMARY_PIC_PORT), vector)
    } else {
        (Pic(SECONDARY_PIC_PORT), vector - 8)
    }
}

/// Masks a specific IRQ (range 0..16).
/// Masking an IRQ means the PIC will not
/// generate this specific IRQ.
pub fn mask_irq(vector: u8) {
    assert!(vector < 16);

    let (pic, vector) = select_pic(vector);

    // Safety: The PIC IO ports are safe to access
    unsafe {
        let mut mask = pic.read_data();
        mask |= 1 << vector;
        pic.write_data(mask);
    }
}

/// Unmask a specific IRQ (range 0..16).
/// Unmasking an IRQ means the PIC is free
/// to generate this specific IRQ
pub fn unmask_irq(vector: u8) {
    assert!(vector < 16);

    let (pic, vector) = select_pic(vector);

    // Safety: The PIC IO ports are safe to access
    unsafe {
        let mut mask = pic.read_data();
        mask &= !(1 << vector);
        pic.write_data(mask);
    }
}

/// Send an end-of-interrupt signal to the
/// PIC. This informs the PIC that a specific IRQ
/// has been handled and it is free to generate this
/// IRQ again. This function is usually called in the
/// interrupt handler routine.
pub fn send_eoi(vector: u8) {
    assert!(vector < 16);

    let (pic, _) = select_pic(vector);

    // Safety: The PIC IO ports are safe to access
    unsafe {
        pic.write_cmd(0x20);
    }
}

/// Initializes the two PIC's, sets the vector
/// offsets to 0x20 and 0x28 and masks all IRQ's.
///
/// This function needs to be called before any other
/// function in this module and it is not thread safe.
pub fn init() {
    let primary = Pic(PRIMARY_PIC_PORT);
    let secondary = Pic(SECONDARY_PIC_PORT);

    // Safety: The PIC IO ports are safe to access
    unsafe {
        // start initialization
        primary.write_cmd(0x11);
        io_delay();
        secondary.write_cmd(0x11);
        io_delay();

        // set the vector offset
        primary.write_data(PRIMARY_VECTOR_OFFSET);
        io_delay();
        secondary.write_data(SECONDARY_VECTOR_OFFSET);
        io_delay();

        // set up cascading
        primary.write_data(0x04);
        io_delay();
        secondary.write_data(0x02);
        io_delay();

        // set interrupt mode to 8086/8088
        primary.write_data(0x01);
        io_delay();
        secondary.write_data(0x01);
        io_delay();

        // mask all interrupts
        primary.write_data(0xFF);
        io_delay();
        secondary.write_data(0xFF);
        io_delay();
    }
}
