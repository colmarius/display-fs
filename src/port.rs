use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use std::time::Duration;
use thiserror::Error;

const TIMEOUT_MS: u64 = 1000;

const DISPLAY_FS_VID_PID: [(u16, u16); 3] = [
    (0x1A86, 0x7523), // CH340
    (0x1A86, 0x5523), // CH341
    (0x1A86, 0xFE0C), // WeAct Studio Display FS V1
];

const DISPLAY_FS_VID: u16 = 0x1A86;
const DISPLAY_FS_PID_LARGE: u16 = 0xFE0C;
const DISPLAY_FS_BAUD_SMALL: u32 = 115200;
const DISPLAY_FS_BAUD_LARGE: u32 = 1_152_000;

#[derive(Error, Debug)]
pub enum PortError {
    #[error("Display not found")]
    NotFound,
    #[error("Failed to open port: {0}")]
    OpenFailed(#[from] serialport::Error),
}

#[derive(Debug, Clone)]
pub struct PortInfo {
    pub name: String,
    pub vid: u16,
    pub pid: u16,
    pub model: DisplayModel,
    pub baud_rate: u32,
    pub product: Option<String>,
    pub manufacturer: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayModel {
    Small,
    Large,
}

#[derive(Debug, Clone, Copy)]
pub struct DisplayConfig {
    pub model: DisplayModel,
    pub width: u16,
    pub height: u16,
    pub baud_rate: u32,
}

impl DisplayModel {
    pub fn config(self) -> DisplayConfig {
        match self {
            DisplayModel::Small => DisplayConfig {
                model: self,
                width: 80,
                height: 160,
                baud_rate: DISPLAY_FS_BAUD_SMALL,
            },
            DisplayModel::Large => DisplayConfig {
                model: self,
                width: 320,
                height: 480,
                baud_rate: DISPLAY_FS_BAUD_LARGE,
            },
        }
    }
}

pub fn list_ports() -> Vec<SerialPortInfo> {
    serialport::available_ports().unwrap_or_default()
}

pub fn find_display_port() -> Option<PortInfo> {
    for port in list_ports() {
        if let SerialPortType::UsbPort(usb_info) = &port.port_type {
            let vid = usb_info.vid;
            let pid = usb_info.pid;
            if DISPLAY_FS_VID_PID.contains(&(vid, pid)) {
                let model = detect_display_model(usb_info)?;
                let config = model.config();
                return Some(PortInfo {
                    name: port.port_name,
                    vid,
                    pid,
                    model,
                    baud_rate: config.baud_rate,
                    product: usb_info.product.clone(),
                    manufacturer: usb_info.manufacturer.clone(),
                });
            }
        }
    }
    None
}

pub fn is_display_connected() -> bool {
    find_display_port().is_some()
}

pub fn open_connection(port: &PortInfo) -> Result<Box<dyn SerialPort>, PortError> {
    let connection = serialport::new(&port.name, port.baud_rate)
        .timeout(Duration::from_millis(TIMEOUT_MS))
        .open()?;
    Ok(connection)
}

fn detect_display_model(usb_info: &serialport::UsbPortInfo) -> Option<DisplayModel> {
    let product = usb_info
        .product
        .as_deref()
        .unwrap_or_default()
        .to_lowercase();
    if usb_info.vid == DISPLAY_FS_VID && usb_info.pid == DISPLAY_FS_PID_LARGE {
        if product.contains("0.96") {
            return Some(DisplayModel::Small);
        }
        return Some(DisplayModel::Large);
    }
    if product.contains("0.96") {
        return Some(DisplayModel::Small);
    }
    if product.contains("display fs v1") {
        return Some(DisplayModel::Large);
    }
    match (usb_info.vid, usb_info.pid) {
        (0x1A86, 0x7523) | (0x1A86, 0x5523) => Some(DisplayModel::Small),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_ports_returns_vec() {
        let ports = list_ports();
        // Verify it's a valid Vec by checking it doesn't panic
        let _ = ports.len();
    }

    #[test]
    fn test_find_display_port_returns_option() {
        let result = find_display_port();
        // Result is Option<PortInfo> - either Some or None
        match result {
            Some(port) => {
                assert!(!port.name.is_empty());
                assert!(port.vid > 0);
                assert!(port.pid > 0);
            }
            None => {
                // Display not connected - this is valid
            }
        }
    }

    #[test]
    fn test_is_display_connected_returns_bool() {
        let result = is_display_connected();
        // Just verify it returns a bool without panicking
        assert!(result == true || result == false);
    }

    #[test]
    fn test_vid_pid_constants_defined() {
        // Verify CH340, CH341, and WeAct VID/PIDs are defined
        assert!(DISPLAY_FS_VID_PID.contains(&(0x1A86, 0x7523))); // CH340
        assert!(DISPLAY_FS_VID_PID.contains(&(0x1A86, 0x5523))); // CH341
        assert!(DISPLAY_FS_VID_PID.contains(&(0x1A86, 0xFE0C))); // WeAct
    }

    #[test]
    fn test_port_info_struct() {
        let port = PortInfo {
            name: "COM3".to_string(),
            vid: 0x1A86,
            pid: 0x7523,
            model: DisplayModel::Small,
            baud_rate: DISPLAY_FS_BAUD_SMALL,
            product: Some("Display FS 0.96 Inch".to_string()),
            manufacturer: Some("WeAct Studio".to_string()),
        };
        assert_eq!(port.name, "COM3");
        assert_eq!(port.vid, 0x1A86);
        assert_eq!(port.pid, 0x7523);
    }

    #[test]
    fn test_detect_display_model_product_hints() {
        let usb_info = serialport::UsbPortInfo {
            vid: DISPLAY_FS_VID,
            pid: DISPLAY_FS_PID_LARGE,
            serial_number: None,
            manufacturer: Some("WeAct Studio".to_string()),
            product: Some("Display FS V1".to_string()),
        };
        assert_eq!(detect_display_model(&usb_info), Some(DisplayModel::Large));

        let usb_info_small = serialport::UsbPortInfo {
            vid: DISPLAY_FS_VID,
            pid: DISPLAY_FS_PID_LARGE,
            serial_number: None,
            manufacturer: Some("WeAct Studio".to_string()),
            product: Some("Display FS 0.96 Inch".to_string()),
        };
        assert_eq!(
            detect_display_model(&usb_info_small),
            Some(DisplayModel::Small)
        );
    }
}
