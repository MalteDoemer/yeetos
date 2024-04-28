use spin::Mutex;
use memory::{to_higher_half, virt::VirtAddr};
use vga::{text_mode::TextMode80x25, color::Color};

pub static VGA_WRITER: Mutex<TextMode80x25> = Mutex::new(unsafe {
    TextMode80x25::new_with_col(
        Color::White,
        Color::Black,
        to_higher_half(VirtAddr::new(0xb8000)).to_inner(),
    )
});
