use core::fmt;

use memory::{to_higher_half, VirtAddr};
use spin::Mutex;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    pub const fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct VGAChar {
    vga_char: u8,
    color_code: ColorCode,
}

impl VGAChar {
    pub fn new(vga_char: u8, color_code: ColorCode) -> Self {
        VGAChar {
            vga_char,
            color_code,
        }
    }
}

pub struct VGAWriter {
    col_pos: usize,
    write_color: ColorCode,
    buffer_start: VirtAddr,
}

unsafe impl Send for VGAWriter {}
unsafe impl Sync for VGAWriter {}

impl VGAWriter {
    /// Creates a new VGAWriter on the given address.
    /// # Safety
    /// `vga_addr` must point to a valid buffer for vga text mode.
    const unsafe fn new(fg: Color, bg: Color, vga_addr: VirtAddr) -> VGAWriter {
        VGAWriter {
            col_pos: 0,
            write_color: ColorCode::new(fg, bg),
            buffer_start: vga_addr,
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),

            byte => {
                if self.col_pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.col_pos;

                // Safety:
                // We ensure during new() that buffer_start is a valid pointer.
                unsafe {
                    let buffer_start = self.buffer_start.as_ptr_mut::<VGAChar>();
                    let ptr = buffer_start.add(row * BUFFER_WIDTH + col);
                    ptr.write_volatile(VGAChar::new(byte, self.write_color));
                }

                self.col_pos += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // Safety:
        // We ensure during new() that buffer_start is a valid pointer.
        unsafe {
            let ptr = self.buffer_start.as_ptr_mut::<VGAChar>();

            core::ptr::copy(
                ptr.add(BUFFER_WIDTH),
                ptr,
                (BUFFER_HEIGHT - 1) * BUFFER_WIDTH,
            );
        }

        for i in 0..BUFFER_WIDTH {
            let row = BUFFER_HEIGHT - 1;

            // Safety:
            // We ensure during new() that buffer_start is a valid pointer.
            unsafe {
                let buffer_start = self.buffer_start.as_ptr_mut::<VGAChar>();
                let ptr = buffer_start.add(row * BUFFER_WIDTH + i);
                ptr.write_volatile(VGAChar::new(b' ', self.write_color));
            }
        }

        self.col_pos = 0;
    }
}

impl fmt::Write for VGAWriter {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let byte = if c.is_ascii() { c as u8 } else { 0xfe };

        match byte {
            0x20..=0x7e | b'\n' => self.write_byte(byte),
            _ => self.write_byte(0xfe),
        }

        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }
}

pub static VGA_WRITER: Mutex<VGAWriter> = Mutex::new(unsafe {
    VGAWriter::new(
        Color::White,
        Color::Black,
        to_higher_half(VirtAddr::new(0xb8000)),
    )
});
