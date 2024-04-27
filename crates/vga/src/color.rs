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

impl Color {
    pub const fn from_raw(val: u8) -> Color {
        match val {
            0 => Color::Black,
            1 => Color::Blue,
            2 => Color::Green,
            3 => Color::Cyan,
            4 => Color::Red,
            5 => Color::Magenta,
            6 => Color::Brown,
            7 => Color::LightGray,
            8 => Color::DarkGray,
            9 => Color::LightBlue,
            10 => Color::LightGreen,
            11 => Color::LightCyan,
            12 => Color::LightRed,
            13 => Color::Pink,
            14 => Color::Yellow,
            15 => Color::White,
            _ => panic!("invalid argument"),
        }
    }
}

impl ColorCode {
    pub const fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }

    pub const fn foreground(&self) -> Color {
        Color::from_raw(self.0 & 0xF)
    }

    pub const fn background(&self) -> Color {
        Color::from_raw(self.0 >> 4)
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TextCharacter {
    vga_char: u8,
    color_code: ColorCode,
}

impl TextCharacter {
    pub fn new(vga_char: u8, color_code: ColorCode) -> Self {
        TextCharacter {
            vga_char,
            color_code,
        }
    }

    pub fn get_char(&self) -> u8 {
        self.vga_char
    }

    pub fn get_color_code(&self) -> ColorCode {
        self.color_code
    }
}
