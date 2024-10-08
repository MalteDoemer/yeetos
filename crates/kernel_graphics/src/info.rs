use crate::RgbColor;
use alloc::vec::Vec;
use memory::phys::{Frame, PhysAddr, PhysicalRange};

#[derive(Debug, Clone)]
pub struct FrameBufferInfo {
    base_address: PhysAddr,
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
            base_address: PhysAddr::zero(),
            pitch: 0,
            width: 0,
            height: 0,
            bits_per_pixel: 0,
            pixel_format: PixelFormat::EGA(EgaPixelFormat::new()),
        }
    }

    pub const fn new(
        base_address: PhysAddr,
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

    /// The physical address of the frame buffer memory.
    pub fn base_address(&self) -> PhysAddr {
        self.base_address
    }

    /// Calculates the enclosing physical range of the frame buffer.
    ///
    /// # Panics
    /// If the calculation overflows.
    pub fn physical_range(&self) -> PhysicalRange {
        let base_addr = self.base_address.frame_align_down();
        let size_in_bytes = (self.pitch * self.height) as memory::phys::Inner;
        let end_addr = (self.base_address + size_in_bytes).frame_align_up();
        PhysicalRange::new(Frame::new(base_addr), Frame::new(end_addr))
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

impl Default for FrameBufferInfo {
    fn default() -> Self {
        Self::new(
            0xb8000.into(),
            160,
            80,
            25,
            16,
            PixelFormat::EGA(EgaPixelFormat::new()),
        )
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
