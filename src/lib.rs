#![warn(clippy::all)]

pub mod image;
pub mod port;
pub mod protocol;
pub mod spotify;
pub mod text;

pub use image::{
    calculate_auto_fit_size, calculate_auto_fit_size_for_display, calculate_auto_fit_size_oriented,
    calculate_max_chars_per_line, calculate_max_chars_per_line_for_display,
    calculate_max_chars_per_line_oriented, calculate_max_lines, calculate_max_lines_for_display,
    calculate_max_lines_oriented, create_blank_image_for_display, create_text_image,
    create_text_image_for_display, create_text_image_oriented, image_to_rgb565_bytes,
    image_to_rgb565_bytes_for_display, image_to_rgb565_bytes_oriented, measure_text_with_font_size,
    Orientation, DISPLAY_HEIGHT, DISPLAY_WIDTH,
};
pub use port::{
    find_display_port, is_display_connected, open_connection, DisplayConfig, DisplayModel,
    PortInfo,
};
pub use protocol::{
    send_image_to_display, send_image_to_display_for_model, send_image_to_display_oriented,
};
pub use spotify::{get_now_playing, NowPlaying};
pub use text::{split_into_pages, split_into_pages_for_display};
