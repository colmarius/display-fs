use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use std::sync::OnceLock;

/// Display orientation
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Orientation {
    /// 160x80 - wider than tall (default)
    #[default]
    Landscape,
    /// 80x160 - taller than wide
    Portrait,
    /// 160x80 - wider than tall, flipped 180°
    LandscapeFlip,
    /// 80x160 - taller than wide, flipped 180°
    PortraitFlip,
}

impl Orientation {
    /// Logical (width, height) of the drawing canvas for a display with the
    /// given physical portrait dimensions.
    pub fn dimensions(self, physical_width: u32, physical_height: u32) -> (u32, u32) {
        match self {
            Orientation::Landscape | Orientation::LandscapeFlip => {
                (physical_height, physical_width)
            }
            Orientation::Portrait | Orientation::PortraitFlip => (physical_width, physical_height),
        }
    }
}

#[cfg(feature = "japanese")]
const FONT_DATA: &[u8] = include_bytes!("../assets/fonts/NotoSansJP-Regular.otf");

#[cfg(not(feature = "japanese"))]
const FONT_DATA: &[u8] = include_bytes!("../assets/fonts/DejaVuSans.ttf");

/// The embedded font, parsed once.
fn font() -> &'static FontRef<'static> {
    static FONT: OnceLock<FontRef<'static>> = OnceLock::new();
    FONT.get_or_init(|| FontRef::try_from_slice(FONT_DATA).expect("Failed to load embedded font"))
}

pub fn create_blank_image(
    orientation: Orientation,
    physical_width: u32,
    physical_height: u32,
) -> RgbImage {
    let (width, height) = orientation.dimensions(physical_width, physical_height);
    RgbImage::from_pixel(width, height, Rgb([0, 0, 0]))
}

/// Render centered white-on-black text sized for the given display.
pub fn create_text_image(
    text: &str,
    font_size: f32,
    orientation: Orientation,
    physical_width: u32,
    physical_height: u32,
) -> RgbImage {
    let mut img = create_blank_image(orientation, physical_width, physical_height);
    draw_text(&mut img, text, font_size);
    img
}

fn draw_text(img: &mut RgbImage, text: &str, font_size: f32) {
    let font = font();
    let scale = PxScale::from(font_size);
    let line_height = font.as_scaled(scale).height();

    let lines: Vec<&str> = text.lines().collect();
    let total_height = line_height * lines.len() as f32;
    let start_y = ((img.height() as f32 - total_height) / 2.0).max(0.0) as i32;

    for (i, line) in lines.iter().enumerate() {
        let (line_width, _) = measure_text(font, scale, line);
        let x = ((img.width() as i32 - line_width as i32) / 2).max(0);
        let y = start_y + (i as f32 * line_height) as i32;

        draw_text_mut(img, Rgb([255, 255, 255]), x, y, scale, font, line);
    }
}

pub fn measure_text_with_font_size(text: &str, font_size: f32) -> (u32, u32) {
    measure_text(font(), PxScale::from(font_size), text)
}

/// Measure multi-line text dimensions at given font size.
/// Returns (max_line_width, total_height) for the text.
pub fn measure_multiline_text(text: &str, font_size: f32) -> (u32, u32) {
    let font = font();
    let scale = PxScale::from(font_size);
    let line_height = font.as_scaled(scale).height();

    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return (0, 0);
    }

    let max_width = lines
        .iter()
        .map(|line| measure_text(font, scale, line).0)
        .max()
        .unwrap_or(0);

    let total_height = (line_height * lines.len() as f32) as u32;

    (max_width, total_height)
}

pub const MIN_FONT_SIZE: f32 = 8.0;
pub const MAX_FONT_SIZE: f32 = 72.0;
const HORIZONTAL_PADDING: u32 = 8;
const VERTICAL_PADDING: u32 = 4;

/// Calculate the largest font size that fits text within display bounds.
/// Uses binary search between MIN_FONT_SIZE (8.0) and MAX_FONT_SIZE (72.0).
pub fn calculate_auto_fit_size(
    text: &str,
    orientation: Orientation,
    physical_width: u32,
    physical_height: u32,
) -> f32 {
    if text.is_empty() {
        return MIN_FONT_SIZE;
    }

    let (width, height) = orientation.dimensions(physical_width, physical_height);
    let max_text_width = width - HORIZONTAL_PADDING;
    let max_text_height = height - VERTICAL_PADDING;

    let mut low = MIN_FONT_SIZE;
    let mut high = MAX_FONT_SIZE;

    while high - low > 0.5 {
        let mid = (low + high) / 2.0;
        let (width, height) = measure_multiline_text(text, mid);

        if width <= max_text_width && height <= max_text_height {
            low = mid;
        } else {
            high = mid;
        }
    }

    low
}

fn measure_text(font: &FontRef, scale: PxScale, text: &str) -> (u32, u32) {
    let scaled_font = font.as_scaled(scale);
    let mut width = 0.0f32;
    let height = scaled_font.height();

    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        width += scaled_font.h_advance(glyph_id);
    }

    (width as u32, height as u32)
}

/// Estimate how many characters fit on one line, based on the 'x' glyph advance.
pub fn calculate_max_chars_per_line(
    font_size: f32,
    orientation: Orientation,
    physical_width: u32,
    physical_height: u32,
) -> usize {
    let font = font();
    let scaled_font = font.as_scaled(PxScale::from(font_size));

    let avg_width = scaled_font.h_advance(font.glyph_id('x'));
    let (width, _) = orientation.dimensions(physical_width, physical_height);

    if avg_width == 0.0 {
        return 0;
    }

    (width as f32 / avg_width).floor() as usize
}

pub fn calculate_max_lines(
    font_size: f32,
    orientation: Orientation,
    physical_width: u32,
    physical_height: u32,
) -> usize {
    let line_height = font().as_scaled(PxScale::from(font_size)).height();

    if line_height == 0.0 {
        return 0;
    }

    let (_, height) = orientation.dimensions(physical_width, physical_height);
    (height as f32 / line_height).floor() as usize
}

/// Convert a logical-orientation image to RGB565 bytes in physical scan order.
/// The hardware scans in portrait (physical_width x physical_height), so
/// landscape images are rotated 90° and flip variants 180°.
pub fn image_to_rgb565_bytes(
    img: &RgbImage,
    orientation: Orientation,
    physical_width: u32,
    physical_height: u32,
) -> Vec<u8> {
    let mut data = Vec::with_capacity((physical_width * physical_height * 2) as usize);

    match orientation {
        Orientation::Portrait => {
            // Send as-is.
            for y in 0..img.height() {
                for x in 0..img.width() {
                    let pixel = img.get_pixel(x, y);
                    push_rgb565(&mut data, pixel[0], pixel[1], pixel[2]);
                }
            }
        }
        Orientation::Landscape => {
            // Rotate 90° CW: logical x = py, logical y = (physical_width - 1) - px.
            for py in 0..physical_height {
                for px in 0..physical_width {
                    let lx = py;
                    let ly = (physical_width - 1) - px;
                    let pixel = img.get_pixel(lx, ly);
                    push_rgb565(&mut data, pixel[0], pixel[1], pixel[2]);
                }
            }
        }
        Orientation::PortraitFlip => {
            // Rotate 180°.
            for py in 0..physical_height {
                for px in 0..physical_width {
                    let lx = (physical_width - 1) - px;
                    let ly = (physical_height - 1) - py;
                    let pixel = img.get_pixel(lx, ly);
                    push_rgb565(&mut data, pixel[0], pixel[1], pixel[2]);
                }
            }
        }
        Orientation::LandscapeFlip => {
            // Rotate 90° CCW (180° from landscape).
            for py in 0..physical_height {
                for px in 0..physical_width {
                    let lx = (physical_height - 1) - py;
                    let ly = px;
                    let pixel = img.get_pixel(lx, ly);
                    push_rgb565(&mut data, pixel[0], pixel[1], pixel[2]);
                }
            }
        }
    }

    data
}

fn push_rgb565(data: &mut Vec<u8>, r: u8, g: u8, b: u8) {
    let r5 = (r >> 3) & 0x1F;
    let g6 = (g >> 2) & 0x3F;
    let b5 = (b >> 3) & 0x1F;
    let rgb565 = ((r5 as u16) << 11) | ((g6 as u16) << 5) | (b5 as u16);
    data.push((rgb565 & 0xFF) as u8);
    data.push((rgb565 >> 8) as u8);
}

#[cfg(test)]
fn rgb_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    let r5 = (r >> 3) & 0x1F;
    let g6 = (g >> 2) & 0x3F;
    let b5 = (b >> 3) & 0x1F;
    ((r5 as u16) << 11) | ((g6 as u16) << 5) | (b5 as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Physical dimensions of the small (0.96") display, used as the test fixture.
    const SMALL_W: u32 = 80;
    const SMALL_H: u32 = 160;
    const LANDSCAPE: Orientation = Orientation::Landscape;

    #[test]
    fn test_orientation_dimensions() {
        assert_eq!(
            Orientation::Landscape.dimensions(SMALL_W, SMALL_H),
            (160, 80)
        );
        assert_eq!(
            Orientation::Portrait.dimensions(SMALL_W, SMALL_H),
            (80, 160)
        );
        assert_eq!(
            Orientation::LandscapeFlip.dimensions(SMALL_W, SMALL_H),
            (160, 80)
        );
        assert_eq!(
            Orientation::PortraitFlip.dimensions(SMALL_W, SMALL_H),
            (80, 160)
        );
        assert_eq!(Orientation::Landscape.dimensions(320, 480), (480, 320));
    }

    #[test]
    fn test_default_orientation_is_landscape() {
        assert_eq!(Orientation::default(), Orientation::Landscape);
    }

    #[test]
    fn test_create_blank_image_dimensions_and_black() {
        let img = create_blank_image(LANDSCAPE, SMALL_W, SMALL_H);
        assert_eq!((img.width(), img.height()), (160, 80));
        assert_eq!(img.get_pixel(0, 0), &Rgb([0, 0, 0]));

        let portrait = create_blank_image(Orientation::Portrait, SMALL_W, SMALL_H);
        assert_eq!((portrait.width(), portrait.height()), (80, 160));
    }

    #[test]
    fn test_create_text_image_has_content() {
        let blank = create_blank_image(LANDSCAPE, SMALL_W, SMALL_H);
        let text_img = create_text_image("Test", 14.0, LANDSCAPE, SMALL_W, SMALL_H);

        // Text image should differ from blank (has white text)
        let blank_bytes = image_to_rgb565_bytes(&blank, LANDSCAPE, SMALL_W, SMALL_H);
        let text_bytes = image_to_rgb565_bytes(&text_img, LANDSCAPE, SMALL_W, SMALL_H);
        assert_ne!(blank_bytes, text_bytes);
    }

    #[test]
    fn test_rgb565_primary_colors() {
        assert_eq!(rgb_to_rgb565(0, 0, 0), 0x0000);
        assert_eq!(rgb_to_rgb565(255, 255, 255), 0xFFFF);
        assert_eq!(rgb_to_rgb565(255, 0, 0), 0xF800);
        assert_eq!(rgb_to_rgb565(0, 255, 0), 0x07E0);
        assert_eq!(rgb_to_rgb565(0, 0, 255), 0x001F);
    }

    #[test]
    fn test_rgb565_output_size() {
        let img = create_blank_image(LANDSCAPE, SMALL_W, SMALL_H);
        let data = image_to_rgb565_bytes(&img, LANDSCAPE, SMALL_W, SMALL_H);
        // 80 × 160 × 2 bytes = 25600 bytes
        assert_eq!(data.len(), 25600);

        let large = create_blank_image(LANDSCAPE, 320, 480);
        let data = image_to_rgb565_bytes(&large, LANDSCAPE, 320, 480);
        assert_eq!(data.len(), 320 * 480 * 2);
    }

    #[test]
    fn test_rgb565_landscape_rotation_mapping() {
        // Mark logical top-left pixel red in a landscape image; after the 90° CW
        // rotation it must land at physical (79, 0), i.e. the last pixel of the
        // first physical row.
        let mut img = create_blank_image(LANDSCAPE, SMALL_W, SMALL_H);
        img.put_pixel(0, 0, Rgb([255, 0, 0]));
        let data = image_to_rgb565_bytes(&img, LANDSCAPE, SMALL_W, SMALL_H);

        let idx = (79 * 2) as usize; // physical (x=79, y=0), 2 bytes per pixel
        let value = (data[idx] as u16) | ((data[idx + 1] as u16) << 8);
        assert_eq!(value, 0xF800);
    }

    #[cfg(feature = "japanese")]
    #[test]
    fn test_japanese_text_renders() {
        let blank = create_blank_image(LANDSCAPE, SMALL_W, SMALL_H);
        let text_img = create_text_image("こんにちは", 14.0, LANDSCAPE, SMALL_W, SMALL_H);

        let blank_bytes = image_to_rgb565_bytes(&blank, LANDSCAPE, SMALL_W, SMALL_H);
        let text_bytes = image_to_rgb565_bytes(&text_img, LANDSCAPE, SMALL_W, SMALL_H);
        assert_ne!(
            blank_bytes, text_bytes,
            "Japanese text should render visible content"
        );
    }

    #[test]
    fn test_auto_fit_single_char_large() {
        let size = calculate_auto_fit_size("X", LANDSCAPE, SMALL_W, SMALL_H);
        assert!(
            size > 40.0,
            "Single char should fit at large size, got {}",
            size
        );
    }

    #[test]
    fn test_auto_fit_long_text_smaller() {
        // Relative check so the assertion holds for any embedded font.
        let short = calculate_auto_fit_size("X", LANDSCAPE, SMALL_W, SMALL_H);
        let long = calculate_auto_fit_size("Hello World!", LANDSCAPE, SMALL_W, SMALL_H);
        assert!(
            long < short,
            "Long text ({}) should fit at a smaller size than short text ({})",
            long,
            short
        );
    }

    #[test]
    fn test_auto_fit_empty_string_min() {
        let size = calculate_auto_fit_size("", LANDSCAPE, SMALL_W, SMALL_H);
        assert_eq!(
            size, MIN_FONT_SIZE,
            "Empty string should return MIN_FONT_SIZE"
        );
    }

    #[test]
    fn test_auto_fit_multiline_smaller() {
        let single_size = calculate_auto_fit_size("Hello", LANDSCAPE, SMALL_W, SMALL_H);
        let multi_size = calculate_auto_fit_size("Hello\nWorld", LANDSCAPE, SMALL_W, SMALL_H);
        assert!(
            multi_size < single_size,
            "Multi-line should be smaller than single line"
        );
    }

    #[test]
    fn test_auto_fit_result_within_bounds() {
        let (max_w, max_h) = LANDSCAPE.dimensions(SMALL_W, SMALL_H);
        let max_text_width = max_w - HORIZONTAL_PADDING;
        let max_text_height = max_h - VERTICAL_PADDING;

        let size = calculate_auto_fit_size("Hello", LANDSCAPE, SMALL_W, SMALL_H);
        let (w, h) = measure_multiline_text("Hello", size);

        assert!(
            w <= max_text_width,
            "Width {} exceeds max {}",
            w,
            max_text_width
        );
        assert!(
            h <= max_text_height,
            "Height {} exceeds max {}",
            h,
            max_text_height
        );
    }
}
