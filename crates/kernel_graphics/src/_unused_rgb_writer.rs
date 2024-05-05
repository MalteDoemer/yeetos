use crate::{Position, RgbColor, RgbFrameBuffer};
use psf_rs::Font;

pub struct RgbWriter<'a> {
    font: Font<'a>,
    frame_buffer: RgbFrameBuffer<'a>,
    fg_color: u32,
    bg_color: u32,
    current_pos: usize,
}

impl<'a> RgbWriter<'a> {
    pub fn new(
        font: Font<'a>,
        frame_buffer: RgbFrameBuffer<'a>,
        fg: RgbColor,
        bg: RgbColor,
    ) -> Self {
        let fg_color = frame_buffer.make_pixel(fg);
        let bg_color = frame_buffer.make_pixel(bg);

        Self {
            font,
            frame_buffer,
            fg_color,
            bg_color,
            current_pos: 0,
        }
    }

    fn char_width(&self) -> usize {
        self.font.header.glyph_width as usize
    }

    fn char_height(&self) -> usize {
        self.font.header.glyph_height as usize
    }

    fn chars_per_line(&self) -> usize {
        self.frame_buffer.width() / self.char_width()
    }

    fn num_char_lines(&self) -> usize {
        self.frame_buffer.height() / self.char_height()
    }

    fn right_corner_pos(&self) -> Position {
        let y = self.frame_buffer.height() - self.char_height();
        Position::new(0, y)
    }

    fn current_pos(&self) -> Position {
        let x = self.current_pos * self.char_width();
        let y = self.right_corner_pos().y;

        Position::new(x, y)
    }

    fn pixels_per_char_line(&self) -> usize {
        self.frame_buffer.width() * self.char_height()
    }

    fn new_line(&mut self) {
        let ptr = self.frame_buffer.buffer().as_mut_ptr().cast::<u32>();

        // Copy all lines up by one
        unsafe {
            core::ptr::copy(
                ptr.add(self.pixels_per_char_line()),
                ptr,
                (self.num_char_lines() - 1) * self.chars_per_line(),
            );
        }

        // Clear out the last line
        let mut pos = self.right_corner_pos();

        for _ in 0..self.chars_per_line() {
            self.write_char_at(' ', pos);
            pos.x += self.char_width();
        }

        self.current_pos = 0;
    }

    fn write_char_at(&mut self, char: char, position: Position) {
        self.font.display_glyph(char, |b, x, y| {
            let real_x = position.x + x as usize;
            let real_y = position.y + y as usize;
            let col = if b == 0 { self.bg_color } else { self.fg_color };
            self.frame_buffer.put_pixel_xy(real_x, real_y, col);
        });
    }

    fn write_char(&mut self, char: char) {
        match char {
            '\n' => self.new_line(),
            char => {
                if self.current_pos >= self.chars_per_line() {
                    self.new_line();
                }

                let x = self.current_pos * self.char_width();
                let y = self.right_corner_pos().y;

                self.write_char_at(char, Position::new(x, y));

                self.current_pos += 1;
            }
        }
    }
}

impl core::fmt::Write for RgbWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.write_char(c.into());
        Ok(())
    }
}
