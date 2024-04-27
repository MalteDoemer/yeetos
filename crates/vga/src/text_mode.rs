use core::fmt;

use crate::color::{Color, ColorCode, TextCharacter};

pub const DEFAULT_COLOR: ColorCode = ColorCode::new(Color::White, Color::Black);

pub trait TextWriter: fmt::Write {
    const HEIGHT: usize;
    const WIDTH: usize;
}

pub struct VGATextMode80x25 {
    current_pos: usize,
    write_color: ColorCode,
    buffer_address: usize,
}

impl TextWriter for VGATextMode80x25 {
    const HEIGHT: usize = 25;
    const WIDTH: usize = 80;
}

impl fmt::Write for VGATextMode80x25 {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let byte = if c.is_ascii() { c as u8 } else { 0xfe };

        match byte {
            0x20..=0x7e | 0xfe | b'\n' => self.write_byte(byte),
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

impl VGATextMode80x25 {
    /// Creates a new `VGATextMode80x25` that can be
    /// used to write to a VGA device configured for 80x25 text mode.
    ///
    /// # Safety
    /// - `vga_address` must point to a valid VGA buffer configured for 80x25 text mode.
    /// - There must only be one instance of `VGATextMode80x25` accessing the vga buffer at a time
    pub const unsafe fn new(vga_address: usize) -> Self {
        Self {
            current_pos: 0,
            write_color: DEFAULT_COLOR,
            buffer_address: vga_address,
        }
    }

    pub fn set_color(&mut self, col: ColorCode) {
        self.write_color = col;
    }

    pub fn get_color(&self) -> ColorCode {
        self.write_color
    }

    pub fn clear_screen(&mut self) {
        self.current_pos = 0;

        let count = Self::HEIGHT * Self::WIDTH;
        let char = TextCharacter::new(b' ', self.write_color);

        for i in 0..count {
            self.write_char_at(char, i);
        }
    }

    fn buffer_start(&self) -> *mut TextCharacter {
        self.buffer_address as *mut TextCharacter
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x08 => self.backspace(),

            byte => {
                if self.current_pos >= Self::WIDTH {
                    self.new_line();
                }

                self.write_char_at(
                    TextCharacter::new(byte, self.write_color),
                    (Self::HEIGHT - 1) * self.current_pos,
                );

                self.current_pos += 1;
            }
        }
    }

    /// This function makes room for a new line by shifting all but the first line
    /// up by one line. Then we overwrite the last line with spaces so that we can write to it.
    fn new_line(&mut self) {
        // Safety:
        // We ensure during new() that buffer_address is a valid pointer.

        // Copy all lines up by one
        unsafe {
            let ptr = self.buffer_start();
            core::ptr::copy(ptr.add(Self::WIDTH), ptr, (Self::HEIGHT - 1) * Self::WIDTH);
        }

        // Overwrite the last line with spaces
        let row = Self::HEIGHT - 1;
        for i in 0..Self::WIDTH {
            self.write_char_at(
                TextCharacter::new(b' ', self.write_color),
                row * Self::WIDTH + i,
            );
        }
    }

    /// This function deletes the current character from the last row.
    fn backspace(&mut self) {
        if self.current_pos > 0 {
            self.current_pos -= 1;
            self.write_char_at(TextCharacter::new(b' ', self.write_color), self.current_pos);
        }
    }

    fn write_char_at(&self, char: TextCharacter, pos: usize) {
        assert!(pos < Self::WIDTH * Self::HEIGHT);

        // Safety:
        // We ensure during new() that buffer_address is a valid pointer.
        unsafe {
            let ptr = self.buffer_start().add(pos);
            ptr.write_volatile(char);
        }
    }
}
