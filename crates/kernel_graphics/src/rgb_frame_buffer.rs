use crate::{FrameBufferInfo, PixelFormat, RgbColor, RgbPixelFormat};
use vcell::VolatileCell;

/// This struct represents a frame buffer that stores pixels as u32's with 8-bit values for each
/// color
pub struct RgbFrameBuffer<'a> {
    /// The underlying memory of the framebuffer.
    ///
    /// # Note
    /// We use a `VolatileCell` here so that the compiler
    /// cannot optimize-away any writes to the framebuffer.
    buffer: &'a mut [VolatileCell<u32>],
    /// The pitch indicates how many __bytes__ per line there are.
    ///
    /// This means that `self.height() * self.pitch()` is the size of the frame buffer in bytes.
    pitch: usize,
    /// The width of the visible screen in pixels.
    width: usize,
    /// The height of the visible screen in pixels.
    height: usize,
    /// The position of the red value in each pixel.
    red_shift: u8,
    /// The position of the green value in each pixel.
    green_shift: u8,
    /// The position of blue red value in each pixel.
    blue_shift: u8,
}

#[derive(Debug)]
pub enum RgbFrameBufferError {
    InvalidBitsPerPixelValue,
    FrameBufferNotInRGBMode,
    InvalidColorChannelSize,
}

impl<'a> RgbFrameBuffer<'a> {
    // /// Creates a new `RgbFrameBuffer` from a frame buffer info struct.
    // ///
    // /// # Safety
    // /// This instance of `RgbFrameBuffer<'a>` must have exclusive write access to the memory
    // /// pointed to by `frame_buffer_info.base_address` with size (in bytes)
    // /// `frame_buffer_info.pitch() * frame_buffer_info.height()` for the duration of `'a`
    // pub unsafe fn from_info(
    //     frame_buffer_info: &FrameBufferInfo,
    // ) -> Result<RgbFrameBuffer<'a>, RgbFrameBufferError> {
    //     let format = Self::do_checks(&frame_buffer_info)?;
    // 
    //     let ptr = frame_buffer_info
    //         .base_address()
    //         .as_ptr_mut::<VolatileCell<u32>>();
    // 
    //     let size_in_bytes = frame_buffer_info.pitch() * frame_buffer_info.height();
    //     let len = size_in_bytes / 4;
    // 
    //     // Safety: fuction contract ensures exclusive access
    //     let slice = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
    // 
    //     Ok(Self::new(
    //         slice,
    //         frame_buffer_info.pitch(),
    //         frame_buffer_info.width(),
    //         frame_buffer_info.height(),
    //         format.red_field_position(),
    //         format.green_field_position(),
    //         format.blue_field_position(),
    //     ))
    // }

    fn _do_checks(
        frame_buffer_info: &FrameBufferInfo,
    ) -> Result<&RgbPixelFormat, RgbFrameBufferError> {
        // the bits-per-pixel has to be 32
        if frame_buffer_info.bits_per_pixel() != 32 {
            return Err(RgbFrameBufferError::InvalidBitsPerPixelValue);
        }

        // the frame buffer needs to be in RGB mode
        let format = if let PixelFormat::RGB(format) = frame_buffer_info.pixel_format() {
            Ok(format)
        } else {
            Err(RgbFrameBufferError::FrameBufferNotInRGBMode)
        }?;

        // the colors should each use 8-bit values
        if format.red_mask_size() != 8
            || format.green_mask_size() != 8
            || format.blue_mask_size() != 8
        {
            return Err(RgbFrameBufferError::InvalidColorChannelSize);
        }

        // we could also check that the positions reported for red green and blue are good values
        // but for now we just assume they are correct and trustable like any other information from
        // the boot loader

        Ok(format)
    }
}

impl<'a> RgbFrameBuffer<'a> {
    pub fn new(
        buffer: &'a mut [VolatileCell<u32>],
        pitch: usize,
        width: usize,
        height: usize,
        red_shift: u8,
        green_shift: u8,
        blue_shift: u8,
    ) -> Self {
        Self {
            buffer,
            pitch,
            width,
            height,
            red_shift,
            green_shift,
            blue_shift,
        }
    }

    pub fn buffer(&mut self) -> &mut [VolatileCell<u32>] {
        self.buffer
    }

    /// The width of the visible screen in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// The height of the visible screen in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// The pitch indicates how many __bytes__ per line there are.
    ///
    /// This means that `self.height() * self.pitch()` is the size of the frame buffer in bytes.
    pub fn pitch(&self) -> usize {
        self.pitch
    }

    /// Indicates how many pixels (including invisibles) there a per line.
    ///
    /// This is calculated by `self.pitch / 4`. This will mostly be the same as `self.width()` if
    /// there are no invisible pixels.
    #[inline(always)]
    pub fn pixels_per_line(&self) -> usize {
        self.pitch / 4
    }

    /// Creates a pixel value for a given RGB color.
    pub fn make_pixel(&self, rgb_color: RgbColor) -> u32 {
        let mut pixel = 0;
        pixel |= (rgb_color.red as u32) << self.red_shift;
        pixel |= (rgb_color.green as u32) << self.green_shift;
        pixel |= (rgb_color.blue as u32) << self.blue_shift;
        pixel
    }

    #[inline(always)]
    pub fn put_pixel(&mut self, index: usize, pixel: u32) {
        self.buffer[index].set(pixel);
    }

    #[inline(always)]
    pub fn get_pixel(&self, index: usize) -> u32 {
        self.buffer[index].get()
    }

    #[inline(always)]
    pub fn put_pixel_xy(&mut self, x: usize, y: usize, pixel: u32) {
        let index = y * self.pixels_per_line() + x;
        self.put_pixel(index, pixel);
    }

    #[inline(always)]
    pub fn get_pixel_xy(&self, x: usize, y: usize) -> u32 {
        let index = y * self.pixels_per_line() + x;
        self.get_pixel(index)
    }

    #[inline(always)]
    pub unsafe fn put_pixel_unchecked(&mut self, index: usize, pixel: u32) {
        unsafe {
            self.buffer
                .as_mut_ptr()
                .cast::<u32>()
                .add(index)
                .write_volatile(pixel);
        }
    }

    #[inline(always)]
    pub unsafe fn get_pixel_unchecked(&self, index: usize) -> u32 {
        unsafe {
            self.buffer
                .as_ptr()
                .cast::<u32>()
                .add(index)
                .read_volatile()
        }
    }
}
