use crate::RgbColor;
use alloc::vec::Vec;
use memory::virt::VirtAddr;

#[derive(Debug, Clone)]
pub struct FrameBufferInfo {
    base_address: VirtAddr,
    pitch: usize,
    width: usize,
    height: usize,
    bits_per_pixel: usize,
    pixel_format: PixelFormat,
}

#[derive(Debug, Clone)]
pub enum PixelFormat {
    Indexed(IndexedPixelFormat),
    RGB(RgbPixelFormat),
    EGA(EgaPixelFormat),
}

#[derive(Debug, Clone)]
pub struct IndexedPixelFormat {
    color_pallet: Vec<RgbColor>,
}

#[derive(Debug, Copy, Clone)]
pub struct RgbPixelFormat {
    red_field_position: u8,
    red_mask_size: u8,
    green_field_position: u8,
    green_mask_size: u8,
    blue_field_position: u8,
    blue_mask_size: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct EgaPixelFormat {}

impl FrameBufferInfo {
    pub const fn empty() -> Self {
        Self {
            base_address: VirtAddr::zero(),
            pitch: 0,
            width: 0,
            height: 0,
            bits_per_pixel: 0,
            pixel_format: PixelFormat::EGA(EgaPixelFormat::new()),
        }
    }

    pub const fn new(
        base_address: VirtAddr,
        pitch: usize,
        width: usize,
        height: usize,
        bits_per_pixel: usize,
        pixel_format: PixelFormat,
    ) -> Self {
        Self {
            base_address,
            pitch,
            width,
            height,
            bits_per_pixel,
            pixel_format,
        }
    }

    /// The virtual address of the frame buffer memory.
    pub fn base_address(&self) -> VirtAddr {
        self.base_address
    }

    /// The pitch indicates how many __bytes__ per line there are.
    ///
    /// This means that `self.height() * self.pitch()` is the size of the frame buffer in bytes.
    pub fn pitch(&self) -> usize {
        self.pitch
    }

    /// The width of the visible area of frame buffer in __pixels__.
    pub fn width(&self) -> usize {
        self.width
    }

    /// The height of the frame buffer (how many lines).
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.bits_per_pixel
    }

    /// Describes what the expected format of a pixel is.
    pub fn pixel_format(&self) -> &PixelFormat {
        &self.pixel_format
    }
}

impl IndexedPixelFormat {
    pub const fn new(color_pallet: Vec<RgbColor>) -> Self {
        Self { color_pallet }
    }

    pub fn color_pallet(&self) -> &[RgbColor] {
        &self.color_pallet
    }
}

impl RgbPixelFormat {
    pub const fn new(
        red_field_position: u8,
        red_mask_size: u8,
        green_field_position: u8,
        green_mask_size: u8,
        blue_field_position: u8,
        blue_mask_size: u8,
    ) -> Self {
        Self {
            red_field_position,
            red_mask_size,
            green_field_position,
            green_mask_size,
            blue_field_position,
            blue_mask_size,
        }
    }

    pub fn red_field_position(&self) -> u8 {
        self.red_field_position
    }
    pub fn red_mask_size(&self) -> u8 {
        self.red_mask_size
    }
    pub fn green_field_position(&self) -> u8 {
        self.green_field_position
    }
    pub fn green_mask_size(&self) -> u8 {
        self.green_mask_size
    }
    pub fn blue_field_position(&self) -> u8 {
        self.blue_field_position
    }
    pub fn blue_mask_size(&self) -> u8 {
        self.blue_mask_size
    }
}

impl EgaPixelFormat {
    pub const fn new() -> Self {
        Self {}
    }
}
