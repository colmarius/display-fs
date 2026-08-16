use crate::image::Orientation;
use crate::port::DisplayConfig;
use serialport::SerialPort;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;

const CMD_SET_ORIENTATION: u8 = 0x02;
const CMD_SET_BITMAP: u8 = 0x05;
const CMD_END: u8 = 0x0A;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Failed to send data: {0}")]
    SendFailed(#[from] std::io::Error),
}

/// Bitmap command covering the full physical screen of the given display.
/// Rotation is handled in the image data, so the header always uses
/// physical (portrait) dimensions.
fn create_bitmap_header(config: DisplayConfig) -> [u8; 10] {
    let x0: u16 = 0;
    let y0: u16 = 0;
    let x1: u16 = config.width - 1;
    let y1: u16 = config.height - 1;

    [
        CMD_SET_BITMAP,
        (x0 & 0xFF) as u8,
        (x0 >> 8) as u8,
        (y0 & 0xFF) as u8,
        (y0 >> 8) as u8,
        (x1 & 0xFF) as u8,
        (x1 >> 8) as u8,
        (y1 & 0xFF) as u8,
        (y1 >> 8) as u8,
        CMD_END,
    ]
}

/// Orientation command to initialize display orientation.
/// Flip variants are handled in image data rotation to avoid device quirks.
fn create_orientation_command(orientation: Orientation) -> [u8; 3] {
    // Orientation values: 0=portrait, 1=landscape
    let orientation_value = match orientation {
        Orientation::Portrait | Orientation::PortraitFlip => 0,
        Orientation::Landscape | Orientation::LandscapeFlip => 1,
    };
    [CMD_SET_ORIENTATION, orientation_value, CMD_END]
}

/// Send a full-screen RGB565 frame to the display.
pub fn send_image_to_display(
    port: &mut Box<dyn SerialPort>,
    config: DisplayConfig,
    image_data: &[u8],
    orientation: Orientation,
) -> Result<(), ProtocolError> {
    port.clear(serialport::ClearBuffer::All)
        .map_err(|e| ProtocolError::SendFailed(std::io::Error::other(e)))?;

    let orient_cmd = create_orientation_command(orientation);
    port.write_all(&orient_cmd)?;
    port.flush()?;
    sleep(Duration::from_millis(50));

    let header = create_bitmap_header(config);
    port.write_all(&header)?;
    port.flush()?;

    let chunk_size = config.width as usize * 4;
    for chunk in image_data.chunks(chunk_size) {
        port.write_all(chunk)?;
    }

    port.flush()?;
    sleep(Duration::from_millis(100));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port::DisplayModel;

    fn small_config() -> DisplayConfig {
        DisplayModel::Small.config()
    }

    #[test]
    fn test_bitmap_header_structure() {
        let header = create_bitmap_header(small_config());
        assert_eq!(header.len(), 10);
        assert_eq!(header[0], CMD_SET_BITMAP);
        assert_eq!(header[9], CMD_END);
    }

    #[test]
    fn test_bitmap_header_small_display_dimensions() {
        let header = create_bitmap_header(small_config());
        // x0 = 0, y0 = 0
        assert_eq!(&header[1..5], &[0x00, 0x00, 0x00, 0x00]);
        // Physical: x1 = 79 (0x4F), y1 = 159 (0x9F)
        assert_eq!(&header[5..9], &[0x4F, 0x00, 0x9F, 0x00]);
    }

    #[test]
    fn test_bitmap_header_large_display_dimensions() {
        let header = create_bitmap_header(DisplayModel::Large.config());
        assert_eq!(header[5], 0x3F); // x1 low (319)
        assert_eq!(header[6], 0x01); // x1 high
        assert_eq!(header[7], 0xDF); // y1 low (479)
        assert_eq!(header[8], 0x01); // y1 high
    }

    #[test]
    fn test_command_constants() {
        assert_eq!(CMD_SET_BITMAP, 0x05);
        assert_eq!(CMD_END, 0x0A);
    }

    #[test]
    fn test_orientation_command_values() {
        assert_eq!(create_orientation_command(Orientation::Portrait)[1], 0);
        assert_eq!(create_orientation_command(Orientation::Landscape)[1], 1);
        assert_eq!(create_orientation_command(Orientation::PortraitFlip)[1], 0);
        assert_eq!(create_orientation_command(Orientation::LandscapeFlip)[1], 1);
    }
}
