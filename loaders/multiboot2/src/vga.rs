use core::fmt;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
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
pub struct ColorCode(u8);

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

pub struct VGAWriter {
    col_pos: usize,
    write_color: ColorCode,
    buffer_start: *mut VGAChar,
}

unsafe impl Send for VGAWriter {}
unsafe impl Sync for VGAWriter {}

impl VGAWriter {
    pub fn new(fg: Color, bg: Color) -> VGAWriter {
        let vga_addr = 0xb8000;

        VGAWriter {
            col_pos: 0,
            write_color: ColorCode::new(fg, bg),
            buffer_start: vga_addr as *mut VGAChar,
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
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
                    let ptr = self.buffer_start.add(row * BUFFER_WIDTH + col);
                    ptr.write_volatile(VGAChar {
                        vga_char: byte,
                        color_code: self.write_color,
                    });
                }

                self.col_pos += 1;
            }
        }
    }

    pub fn new_line(&mut self) {
        // Safety:
        // We ensure during new() that buffer_start is a valid pointer.
        // FIXME: maybe write_volatile() needs to be used here?
        unsafe {
            core::ptr::copy(
                self.buffer_start.add(BUFFER_WIDTH),
                self.buffer_start,
                (BUFFER_HEIGHT - 1) * BUFFER_WIDTH,
            );
        }

        for i in 0..BUFFER_WIDTH {
            let row = BUFFER_HEIGHT - 1;

            // Safety:
            // We ensure during new() that buffer_start is a valid pointer.
            unsafe {
                let ptr = self.buffer_start.add(row * BUFFER_WIDTH + i);
                ptr.write_volatile(VGAChar {
                    vga_char: b' ',
                    color_code: self.write_color,
                });
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
