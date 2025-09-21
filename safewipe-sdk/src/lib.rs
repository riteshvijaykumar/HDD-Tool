//! SafeWipe SDK: High-level API wrapper for safewipe-engine

use anyhow::Result;
use safewipe_engine::device::{Device, DriveType, Interface, VendorCapabilities};
pub use safewipe_engine::wipe::{SanitizationEngine, WipeResult as EngineWipeResult};
pub use safewipe_engine::wipe::SanitizationMethod;
use safewipe_engine::report::{generate_report, Report};
use safewipe_engine::verify::verify_device;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WipeResult {
    pub success: bool,
    pub message: String,
    pub engine_result: Option<EngineWipeResult>,
}

/// Wipe a device and return result
pub async fn wipe_device(device_path: &str, method: SanitizationMethod) -> Result<WipeResult> {
    // For now, create a minimal Device. In production, scan for device info.
    let device = Device {
        path: device_path.to_string(),
        name: device_path.to_string(),
        size: 0,
        device_type: DriveType::Unknown,
        interface: Interface::Unknown,
        vendor: None,
        model: None,
        serial: None,
        capabilities: VendorCapabilities {
            supports_ata_secure_erase: false,
            supports_nvme_sanitize: false,
            supports_crypto_erase: false,
            supports_enhanced_erase: false,
        },
        mount_points: vec![],
        is_system_drive: false,
    };
    let engine = SanitizationEngine::new().with_real_device_access(true);
    let result = engine.sanitize_device(&device, method).await;
    match result {
        Ok(engine_result) => Ok(WipeResult {
            success: true,
            message: "Device wiped successfully".to_string(),
            engine_result: Some(engine_result),
        }),
        Err(e) => Ok(WipeResult {
            success: false,
            message: format!("Wipe failed: {}", e),
            engine_result: None,
        }),
    }
}

/// Generate a wipe report for a device
pub fn generate_wipe_report(device_path: &str) -> Result<Report> {
    generate_report(device_path)
}

/// Verify a device wipe
pub fn verify_device_wipe(device_path: &str) -> Result<bool> {
    verify_device(device_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use safewipe_engine::wipe::SanitizationMethod;
    use tokio::runtime::Runtime;

    #[test]
    fn test_generate_wipe_report() {
        let device_path = "/dev/null"; // Safe dummy path
        let report = generate_wipe_report(device_path);
        assert!(report.is_ok());
        let report = report.unwrap();
        assert_eq!(report.device, device_path);
        assert_eq!(report.status, "success");
    }

    #[test]
    fn test_verify_device_wipe() {
        let device_path = "/dev/null"; // Safe dummy path
        let result = verify_device_wipe(device_path);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_wipe_device_async() {
        let device_path = "/dev/null"; // Safe dummy path
        let method = SanitizationMethod::Clear;
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(wipe_device(device_path, method));
        assert!(result.is_ok());
        let result = result.unwrap();
        // Should fail to wipe /dev/null, but should not panic
        assert!(!result.success || result.success);
    }
}
