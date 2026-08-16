#![warn(clippy::all)]

pub mod image;
pub mod port;
pub mod protocol;
pub mod spotify;
pub mod text;

pub use image::{
    calculate_auto_fit_size, calculate_max_chars_per_line, calculate_max_lines, create_blank_image,
    create_text_image, image_to_rgb565_bytes, measure_text_with_font_size, Orientation,
    MAX_FONT_SIZE, MIN_FONT_SIZE,
};
pub use port::{find_display_port, open_connection, DisplayConfig, DisplayModel, PortInfo};
pub use protocol::send_image_to_display;
pub use spotify::{get_now_playing, NowPlaying};
pub use text::split_into_pages;
