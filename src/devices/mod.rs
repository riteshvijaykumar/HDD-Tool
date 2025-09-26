//! Device-specific erasure modules
//! 
//! This module contains device-specific erasure methods organized by storage device type.
//! Each device type has its own specialized erasure functions that are called after
//! disk analysis determines the appropriate method.

pub mod hdd;
pub mod ssd;
pub mod nvme;
pub mod usb;
pub mod sdcard;

use std::io;
use std::sync::{Arc, Mutex};
use crate::advanced_wiper::{DeviceInfo, WipingProgress, WipingAlgorithm};

/// Common interface for all device types
pub trait DeviceEraser {
    /// Analyze the device to determine optimal erasure method
    fn analyze_device(&self, device_path: &str) -> io::Result<DeviceInfo>;
    
    /// Perform device-specific erasure
    fn erase_device(
        &self,
        device_info: &DeviceInfo,
        algorithm: WipingAlgorithm,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()>;
    
    /// Verify erasure completion
    fn verify_erasure(&self, device_info: &DeviceInfo) -> io::Result<bool>;
    
    /// Get recommended algorithms for this device type
    fn get_recommended_algorithms(&self) -> Vec<WipingAlgorithm>;
}

/// Device type detection and factory
pub struct DeviceFactory;

impl DeviceFactory {
    /// Create appropriate eraser based on device analysis
    pub fn create_eraser(device_info: &DeviceInfo) -> Box<dyn DeviceEraser> {
        match device_info.device_type {
            crate::advanced_wiper::DeviceType::HDD => Box::new(hdd::HddEraser::new()),
            crate::advanced_wiper::DeviceType::SSD => Box::new(ssd::SsdEraser::new()),
            crate::advanced_wiper::DeviceType::NVMe => Box::new(nvme::NvmeEraser::new()),
            crate::advanced_wiper::DeviceType::USBDrive => Box::new(usb::UsbEraser::new()),
            crate::advanced_wiper::DeviceType::SDCard => Box::new(sdcard::SdCardEraser::new()),
            crate::advanced_wiper::DeviceType::MMC | 
            crate::advanced_wiper::DeviceType::EMmc => Box::new(sdcard::SdCardEraser::new()),
            crate::advanced_wiper::DeviceType::CompactFlash => Box::new(usb::UsbEraser::new()),
            crate::advanced_wiper::DeviceType::Other(_) => Box::new(hdd::HddEraser::new()), // Default fallback
        }
    }
    
    /// Analyze device and return appropriate eraser
    pub fn analyze_and_create(device_path: &str) -> io::Result<(DeviceInfo, Box<dyn DeviceEraser>)> {
        // First, do a generic analysis to determine device type
        let temp_eraser = hdd::HddEraser::new();
        let device_info = temp_eraser.analyze_device(device_path)?;
        
        // Create the appropriate specialized eraser
        let eraser = Self::create_eraser(&device_info);
        
        // Re-analyze with the specialized eraser for more detailed info
        let detailed_info = eraser.analyze_device(device_path)?;
        
        Ok((detailed_info, eraser))
    }
}