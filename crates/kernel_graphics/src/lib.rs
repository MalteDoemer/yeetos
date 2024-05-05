//! This crate abstracts the various frame buffers provided by the boot environment for example grub
//! or the uefi GOP.

#![no_std]

mod info;
mod rgb_frame_buffer;

extern crate alloc;

pub use info::{EgaPixelFormat, FrameBufferInfo, IndexedPixelFormat, PixelFormat, RgbPixelFormat};
pub use rgb_frame_buffer::{RgbFrameBuffer, RgbFrameBufferError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl RgbColor {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        RgbColor { red, green, blue }
    }
}

impl Position {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}
