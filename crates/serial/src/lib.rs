#![no_std]

use spin::Mutex;
use x86::io::outb;

// This struct is used so no one can create a `SerialWriter` from
// outside this crate.
struct Token;

pub struct SerialWriter(Token);

const COM1_ADDR: u16 = 0x3F8;

unsafe fn write_byte(byte: u8) {
    unsafe {
        outb(COM1_ADDR, byte);
    }
}

impl core::fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        let byte = if c.is_ascii() { c as u8 } else { b'?' };
        unsafe {
            write_byte(byte);
        }
        Ok(())
    }
}

pub static SERIAL_WRITER: Mutex<SerialWriter> = Mutex::new(SerialWriter(Token));
