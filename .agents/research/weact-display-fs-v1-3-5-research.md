# Research: WeAct Display FS V1 (3.5 inch)

**Date:** 2026-02-11
**Status:** Complete
**Tags:** display-fs, weact, usb-cdc, protocol, rust, python

## Summary

The 3.5-inch Display FS V1 is a USB 2.0 Full Speed CDC device with a 320x480 RGB565 IPS LCD and on-board humidity/temperature sensor. WeAct publishes a Python System Monitor app (forked from Turing Smart Screen) with protocol and implementation details, and a SourceForge protocol spreadsheet describing the full command set (write/read/auto-report). The display shares the same command protocol as the 0.96-inch version, but differs in resolution, baud rate (1,152,000 in reference code), and adds a humiture auto-report command.

## Key Learnings

- Official WeAct docs list the 3.5" model as 320x480 RGB565 with USB2.0 FS (CDC) and a humidity/temperature sensor; the 0.96" is 80x160 RGB565 with no sensor.
- Reference Python code auto-detects the device by scanning the port description for "AB" (3.5") and "AD" (0.96") and sets the resolution accordingly.
- The reference Python app opens the serial port at **1,152,000 baud** for both devices.
- The WeAct protocol spreadsheet defines command bytes, packet lengths, read responses, and humiture auto-report format.
- Image transfer supports raw RGB565 payloads or a FastLZ-compressed variant (4k chunking) with 500ms timeout per chunk.

## Details

### Hardware Specs (WeAct Studio)

From WeAct Studio System Monitor README / SourceForge:

- **Display FS V1 (3.5 inch)**
  - Resolution: **320 x 480** RGB565
  - Communication: **USB2.0 FS (CDC)**
  - Sensor: **Humidity + Temperature**
  - Size: **58.5mm x 87.7mm x 9.0mm**
- **Display FS V1 (0.96 inch)**
  - Resolution: **80 x 160** RGB565
  - Communication: **USB2.0 FS (CDC)**
  - Sensor: **None**
  - Size: **43mm x 14.5mm**

### Device Detection (Python reference)

From `weact_device_setting.py` in WeActStudio.SystemMonitor:

- Auto-detects ports by scanning the port description for:
  - `"AB"` → 3.5-inch device (type 0)
  - `"AD"` → 0.96-inch device (type 1)
- Sets dimensions based on type:
  - Type 0: 320x480 (portrait), 480x320 (landscape)
  - Type 1: 80x160 (portrait), 160x80 (landscape)
- Opens serial port at **1,152,000 baud**, 8N1, 200ms timeout.

### Protocol (SourceForge XLSX v1.1)

Source: `WeAct Studio Display Communication protocol_v1.1.xlsx` (Doc folder on SourceForge).

**Write commands** (all packets end with `0x0A`):

- `0x02 SET_ORIENTATION` (len 3): `0x02, x, 0x0A` where `x=0-4`
- `0x03 SET_BRIGHTNESS` (len 5): `0x03, x, y_l8, y_h8, 0x0A`
  - `x = brightness (0-255)`
  - `y = brightness time (0-5000 ms)`
- `0x04 FULL` (len 12): `0x04, xs_l8, xs_h8, ys_l8, ys_h8, xe_l8, xe_h8, ye_l8, ye_h8, c_l8, c_h8, 0x0A`
- `0x05 SET_BITMAP` (len 10): `0x05, xs_l8, xs_h8, ys_l8, ys_h8, xe_l8, xe_h8, ye_l8, ye_h8, 0x0A`
  - Image data (RGB565) follows immediately. Timeout 500ms.
- `0x06 ENABLE_HUMITURE_REPORT` (len 4): `0x06, y_l8, y_h8, 0x0A`
  - `y=report time (500-65535), y=0 disables`
- `0x07 FREE` (len 2): `0x07, 0x0A` (show initial screen)
- `0x08 LCD_REG_ADDR` (len 3): `0x08, x, 0x0A`
- `0x09 LCD_REG_DATA` (len 3): `0x09, x, 0x0A`
- `0x10 SET_UNCONNECT_BRIGHTNESS` (len 3): `0x10, x, 0x0A`
- `0x11 SET_UNCONNECT_ORIENTATION` (len 3): `0x11, x, 0x0A` (`x=0-5`)
- `0x15 SET_BITMAP_WITH_FASTLZ` (len 10): same header as `SET_BITMAP`, compressed payload
- `0x40 SYSTEM_RESET` (len 2): `0x40, 0x0A`

**Orientation values**

- `PORTRAIT = 0`
- `REVERSE_PORTRAIT = 1`
- `LANDSCAPE = 2`
- `REVERSE_LANDSCAPE = 3`
- `ROTATE = 5`

**Read commands** (request and response, all end with `0x0A`):

- `0x81 WHO_AM_I` → device info
- `0x82 READ_ORIENTATION` → `0x82, x, 0x0A`
- `0x83 READ_BRIGHTNESS` → `0x83, x, 0x0A`
- `0x90 READ_UNCONNECT_BRIGHTNESS` → `0x90, x, 0x0A`
- `0x91 READ_UNCONNECT_ORIENTATION` → `0x91, x, 0x0A`
- `0xC2 READ_SYSTEM_VERSION` → `0xC2, version..., 0x0A`
- `0xC3 READ_SYSTEM_SERIAL_NUM` → `0xC3, serial..., 0x0A`

**Auto report**

- `0x86 HUMITURE_REPORT` (len 6): `0x86, t_l8, t_h8, h_l8, h_h8, 0x0A`
  - Temperature/humidity values are `u16`/`i16` and scaled by 100 in the reference UI.

### Reference Implementation (Python)

- `weact_device_setting.py` sends commands to set orientation/brightness and display bitmap data.
- Uses FastLZ for bitmap updates; chunk size is `width * 4` bytes.
- Converts RGB to RGB565 little-endian.
- Uses a background RX thread and reads auto humiture reports when enabled.

## Sources

- [WeActStudio.SystemMonitor README (Support Hardware)](https://raw.githubusercontent.com/WeActStudio/WeActStudio.SystemMonitor/main/README.md) – 3.5-inch specs, USB CDC, sensor.
- [WeAct Studio System Monitor protocol XLSX v1.1](https://sourceforge.net/projects/weact-studio-system-monitor/files/Doc/WeAct%20Studio%20Display%20Communication%20protocol_v1.1.xlsx/download) – command set, read responses, auto-report.
- [WeActStudio.SystemMonitor weact_device_setting.py](https://raw.githubusercontent.com/WeActStudio/WeActStudio.SystemMonitor/main/weact_device_setting.py) – detection logic, baud rate, command usage.
- [WeAct Studio System Monitor SourceForge files](https://sourceforge.net/projects/weact-studio-system-monitor/files/) – hardware tables and docs links.
- [Bit Trade One WADFS-35 manual](https://bit-trade-one.co.jp/en/wadfs35/manual/) – Windows usage and tool references.

## Open Questions

- [ ] Confirm USB VID/PID and port description strings for the 3.5-inch device on macOS/Linux.
- [ ] Validate whether the 3.5-inch device requires 1,152,000 baud or supports 115,200.
- [ ] Verify FastLZ payload format and chunking details for large frames.
- [ ] Identify any model-specific limitations (brightness range, orientation quirks).

## Related Research

- [[display_fs_v1_research.md]] – 0.96-inch device overview.
