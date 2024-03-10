use x86::io::{inb, outb};

use super::io_delay;

pub const PRIMARY_VECTOR_OFFSET: u8 = 0x20;
pub const SECONDARY_VECTOR_OFFSET: u8 = 0x28;

pub const PRIMARY_PIC_PORT: u16 = 0x20;
pub const SECONDARY_PIC_PORT: u16 = 0xA0;

pub struct Pic(u16);

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

pub fn mask_irq(mut vector: u8) {
    assert!(vector < 16);

    let pic = if vector < 8 {
        Pic(PRIMARY_PIC_PORT)
    } else {
        vector -= 8;
        Pic(SECONDARY_PIC_PORT)
    };

    // Safety: The PIC IO ports are safe to access
    unsafe {
        let mut mask = pic.read_data();
        mask |= 1 << vector;
        pic.write_data(mask);
    }
}

pub fn unmask_irq(mut vector: u8) {
    assert!(vector < 16);

    let pic = if vector < 8 {
        Pic(PRIMARY_PIC_PORT)
    } else {
        vector -= 8;
        Pic(SECONDARY_PIC_PORT)
    };

    // Safety: The PIC IO ports are safe to access
    unsafe {
        let mut mask = pic.read_data();
        mask &= !(1 << vector);
        pic.write_data(mask);
    }
}

pub fn send_eoi(vector: u8) {
    assert!(vector < 16);

    let pic = if vector < 8 {
        Pic(PRIMARY_PIC_PORT)
    } else {
        Pic(SECONDARY_PIC_PORT)
    };

    // Safety: The PIC IO ports are safe to access
    unsafe {
        pic.write_cmd(0x20);
    }
}

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
