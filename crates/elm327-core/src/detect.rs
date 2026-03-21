use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    WchUsbSerial,  // CH340-based (most common for ELM327)
    UsbSerial,     // Generic USB-serial
    SlabUsbToUart, // Silicon Labs CP210x
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DetectedDevice {
    pub path: PathBuf,
    pub device_type: DeviceType,
}

impl std::fmt::Display for DetectedDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.path.display(), self.device_type)
    }
}

/// Classify a device path into a DeviceType based on naming conventions.
///
/// # Examples
/// ```
/// use elm327_core::detect::{classify_device, DeviceType};
/// assert_eq!(classify_device("cu.wchusbserial14340"), DeviceType::WchUsbSerial);
/// ```
pub fn classify_device(path: &str) -> DeviceType {
    let name = path.to_lowercase();
    if name.contains("wchusbserial") {
        DeviceType::WchUsbSerial
    } else if name.contains("slab_usbtouart") || name.contains("cp210") {
        DeviceType::SlabUsbToUart
    } else if name.contains("usbserial") || name.contains("usbmodem") {
        DeviceType::UsbSerial
    } else {
        DeviceType::Unknown
    }
}

/// Enumerate /dev/cu.* and filter for likely OBD adapters.
/// Returns devices sorted by likelihood (WchUsbSerial first).
///
/// # Returns
/// A `Vec<DetectedDevice>` sorted with the most likely OBD adapter first.
/// Only includes devices matching known USB-serial chipset naming patterns.
pub fn detect_devices() -> Vec<DetectedDevice> {
    let mut devices = Vec::new();

    // Glob /dev/cu.*
    let entries = match std::fs::read_dir("/dev") {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Failed to read /dev: {}", e);
            return devices;
        }
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("cu.") {
            continue;
        }

        let path = entry.path();
        let device_type = classify_device(&name);

        // Only include known OBD-likely devices
        if device_type != DeviceType::Unknown {
            devices.push(DetectedDevice {
                path,
                device_type,
            });
        }
    }

    // Sort: WchUsbSerial first (most common ELM327), then others
    devices.sort_by_key(|d| match d.device_type {
        DeviceType::WchUsbSerial => 0,
        DeviceType::SlabUsbToUart => 1,
        DeviceType::UsbSerial => 2,
        DeviceType::Unknown => 3,
    });

    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_wch_usb_serial() {
        assert_eq!(
            classify_device("cu.wchusbserial14340"),
            DeviceType::WchUsbSerial
        );
    }

    #[test]
    fn classify_generic_usb_serial() {
        assert_eq!(
            classify_device("cu.usbserial-1420"),
            DeviceType::UsbSerial
        );
    }

    #[test]
    fn classify_slab_usb_to_uart() {
        assert_eq!(
            classify_device("cu.SLAB_USBtoUART"),
            DeviceType::SlabUsbToUart
        );
    }

    #[test]
    fn classify_bluetooth_is_unknown() {
        assert_eq!(
            classify_device("cu.Bluetooth-Incoming-Port"),
            DeviceType::Unknown
        );
    }

    #[test]
    fn display_detected_device() {
        let dev = DetectedDevice {
            path: PathBuf::from("/dev/cu.wchusbserial14340"),
            device_type: DeviceType::WchUsbSerial,
        };
        assert_eq!(
            dev.to_string(),
            "/dev/cu.wchusbserial14340 (WchUsbSerial)"
        );
    }
}
