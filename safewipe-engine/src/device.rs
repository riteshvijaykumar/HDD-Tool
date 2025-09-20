use anyhow::Result;
use serde::{Serialize, Deserialize};
use sysinfo::{System, Disks};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriveType {
    HDD,
    SSD,
    Removable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Interface {
    SATA,
    NVMe,
    USB,
    SCSI,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorCapabilities {
    pub supports_ata_secure_erase: bool,
    pub supports_nvme_sanitize: bool,
    pub supports_crypto_erase: bool,
    pub supports_enhanced_erase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub device_type: DriveType,
    pub interface: Interface,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub capabilities: VendorCapabilities,
    pub mount_points: Vec<String>,
    pub is_system_drive: bool,
}

impl Device {
    pub fn new(
        path: String,
        name: String,
        size: u64,
        device_type: DriveType,
        interface: Interface,
    ) -> Self {
        Self {
            path,
            name,
            size,
            device_type,
            interface,
            vendor: None,
            model: None,
            serial: None,
            capabilities: VendorCapabilities::default(),
            mount_points: Vec::new(),
            is_system_drive: false,
        }
    }

    pub fn with_vendor_info(mut self, vendor: String, model: String, serial: Option<String>) -> Self {
        self.vendor = Some(vendor);
        self.model = Some(model);
        self.serial = serial;
        self
    }

    pub fn with_capabilities(mut self, capabilities: VendorCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_mount_points(mut self, mount_points: Vec<String>) -> Self {
        self.mount_points = mount_points;
        self
    }

    pub fn mark_as_system_drive(mut self) -> Self {
        self.is_system_drive = true;
        self
    }

    /// Check if device supports secure sanitization methods
    pub fn supports_secure_erase(&self) -> bool {
        match self.device_type {
            DriveType::SSD => {
                self.capabilities.supports_ata_secure_erase ||
                self.capabilities.supports_nvme_sanitize ||
                self.capabilities.supports_crypto_erase
            }
            DriveType::HDD => self.capabilities.supports_ata_secure_erase,
            _ => false,
        }
    }

    /// Get recommended sanitization method for this device
    pub fn recommended_sanitization_method(&self) -> &'static str {
        if self.is_system_drive {
            return "clear"; // Safe default for system drives
        }

        match self.device_type {
            DriveType::SSD => {
                if self.capabilities.supports_crypto_erase {
                    "crypto-erase"
                } else if self.capabilities.supports_nvme_sanitize {
                    "nvme-sanitize"
                } else if self.capabilities.supports_ata_secure_erase {
                    "ata-secure-erase"
                } else {
                    "clear"
                }
            }
            DriveType::HDD => {
                if self.capabilities.supports_ata_secure_erase {
                    "ata-secure-erase"
                } else {
                    "clear"
                }
            }
            DriveType::Removable => "clear",
            DriveType::Unknown => "clear",
        }
    }
}

impl Default for VendorCapabilities {
    fn default() -> Self {
        Self {
            supports_ata_secure_erase: false,
            supports_nvme_sanitize: false,
            supports_crypto_erase: false,
            supports_enhanced_erase: false,
        }
    }
}

/// Drive detection and enumeration
pub struct DriveDetector {
    system: System,
}

impl DriveDetector {
    pub fn new() -> Self {
        let system = System::new_all();
        Self { system }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    /// List all detected drives
    pub fn list_devices(&mut self) -> Result<Vec<Device>> {
        self.refresh();
        let mut devices = Vec::new();

        let disks = Disks::new_with_refreshed_list();
        
        for disk in &disks {
            let device = self.create_device_from_disk(disk)?;
            devices.push(device);
        }

        // Add additional platform-specific detection
        #[cfg(windows)]
        {
            devices.extend(self.detect_windows_drives()?);
        }

        #[cfg(unix)]
        {
            devices.extend(self.detect_unix_drives()?);
        }

        Ok(devices)
    }

    fn create_device_from_disk(&self, disk: &sysinfo::Disk) -> Result<Device> {
        let path = disk.name().to_string_lossy().to_string();
        let size = disk.total_space();
        let mount_points = vec![disk.mount_point().to_string_lossy().to_string()];
        
        // Determine drive type and interface from available information
        let (device_type, interface) = self.determine_drive_characteristics(&path, size)?;
        let capabilities = self.detect_vendor_capabilities(&path, &device_type, &interface)?;
        
        let is_system = self.is_system_drive(&path);
        
        let device = Device::new(
            path.clone(),
            self.extract_device_name(&path),
            size,
            device_type,
            interface,
        )
        .with_capabilities(capabilities)
        .with_mount_points(mount_points);

        Ok(if is_system { device.mark_as_system_drive() } else { device })
    }

    fn determine_drive_characteristics(&self, path: &str, size: u64) -> Result<(DriveType, Interface)> {
        // Basic heuristics - in a real implementation, this would use platform-specific APIs
        let device_type = if size < 64_000_000_000 { // < 64GB, likely removable
            DriveType::Removable
        } else if path.contains("nvme") || path.contains("NVMe") {
            DriveType::SSD
        } else if path.contains("SSD") || path.contains("ssd") {
            DriveType::SSD
        } else {
            DriveType::HDD
        };

        let interface = if path.contains("nvme") || path.contains("NVMe") {
            Interface::NVMe
        } else if path.contains("USB") || path.contains("usb") {
            Interface::USB
        } else {
            Interface::SATA
        };

        Ok((device_type, interface))
    }

    fn detect_vendor_capabilities(&self, _path: &str, device_type: &DriveType, interface: &Interface) -> Result<VendorCapabilities> {
        let mut capabilities = VendorCapabilities::default();

        // Set capabilities based on device type and interface
        match (device_type, interface) {
            (DriveType::SSD, Interface::NVMe) => {
                capabilities.supports_nvme_sanitize = true;
                capabilities.supports_crypto_erase = true;
            }
            (DriveType::SSD, Interface::SATA) => {
                capabilities.supports_ata_secure_erase = true;
                capabilities.supports_crypto_erase = true;
            }
            (DriveType::HDD, Interface::SATA) => {
                capabilities.supports_ata_secure_erase = true;
            }
            _ => {
                // Basic capabilities for other types
            }
        }

        Ok(capabilities)
    }

    fn extract_device_name(&self, path: &str) -> String {
        std::path::Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    fn is_system_drive(&self, path: &str) -> bool {
        // Basic check - in real implementation, check if contains OS files
        #[cfg(windows)]
        {
            path.starts_with("C:") || path.contains("Windows")
        }
        #[cfg(unix)]
        {
            path == "/" || path.starts_with("/boot") || path.starts_with("/usr")
        }
        #[cfg(not(any(windows, unix)))]
        {
            false
        }
    }

    #[cfg(windows)]
    fn detect_windows_drives(&self) -> Result<Vec<Device>> {
        // Platform-specific Windows drive detection using WinAPI
        // This would use GetLogicalDrives, DeviceIoControl etc.
        Ok(Vec::new()) // Placeholder
    }

    #[cfg(unix)]
    fn detect_unix_drives(&self) -> Result<Vec<Device>> {
        // Platform-specific Unix drive detection using /proc, /sys
        // This would parse /proc/partitions, /sys/block etc.
        Ok(Vec::new()) // Placeholder
    }
}

/// Convenience function for backward compatibility
pub fn list_devices() -> Result<Vec<Device>> {
    let mut detector = DriveDetector::new();
    detector.list_devices()
}
