//! SafeWipe SDK: Comprehensive UI-friendly wrapper for safewipe-engine

mod ffi;

use anyhow::Result;
use safewipe_engine::device::{Device, DriveDetector, DriveType, Interface, VendorCapabilities};
pub use safewipe_engine::wipe::{SanitizationEngine, SanitizationMethod, WipeResult as EngineWipeResult, WipeProgress, WipeStatus, WipeResult};
pub use safewipe_engine::report::{generate_report, Report};
use safewipe_engine::verify::verify_device;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Re-export commonly used types
pub use safewipe_engine::wipe::{ClearPattern, PurgeMethod, NvmeSanitizeMode};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub size_formatted: String,
    pub device_type: DriveType,
    pub interface: Interface,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub capabilities: VendorCapabilities,
    pub mount_points: Vec<String>,
    pub is_system_drive: bool,
    pub recommended_method: String,
    pub supported_methods: Vec<String>,
    pub health_status: String,
}

impl From<Device> for DeviceInfo {
    fn from(device: Device) -> Self {
        let size_formatted = format_bytes(device.size);
        let supported_methods = get_supported_methods(&device);
        let recommended_method = device.recommended_sanitization_method().to_string();

        Self {
            id: format!("device_{}", Uuid::new_v4().to_string()[..8].to_lowercase()),
            path: device.path.clone(),
            name: device.name,
            size: device.size,
            size_formatted,
            device_type: device.device_type,
            interface: device.interface,
            vendor: device.vendor,
            model: device.model,
            serial: device.serial,
            capabilities: device.capabilities,
            mount_points: device.mount_points,
            is_system_drive: device.is_system_drive,
            recommended_method,
            supported_methods,
            health_status: "Unknown".to_string(), // TODO: Implement health check
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WipeOperation {
    pub id: String,
    pub device_id: String,
    pub device_path: String,
    pub method: SanitizationMethod,
    pub status: WipeStatus,
    pub progress: Option<WipeProgress>,
    pub result: Option<EngineWipeResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemStats {
    pub total_devices: usize,
    pub system_devices: usize,
    pub wipeable_devices: usize,
    pub active_operations: usize,
    pub completed_operations: usize,
    pub failed_operations: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SafeWipeConfig {
    pub allow_system_drives: bool,
    pub auto_verify: bool,
    pub real_device_access: bool,
    pub backup_reports: bool,
    pub log_level: String,
}

impl Default for SafeWipeConfig {
    fn default() -> Self {
        Self {
            allow_system_drives: false,
            auto_verify: true,
            real_device_access: false,
            backup_reports: true,
            log_level: "info".to_string(),
        }
    }
}

/// Main SDK client for SafeWipe operations
pub struct SafeWipeClient {
    config: Arc<RwLock<SafeWipeConfig>>,
    operations: Arc<RwLock<HashMap<String, WipeOperation>>>,
    progress_sender: broadcast::Sender<WipeProgress>,
    engine: SanitizationEngine,
}

impl SafeWipeClient {
    pub fn new() -> Self {
        let (progress_sender, _) = broadcast::channel(100);

        Self {
            config: Arc::new(RwLock::new(SafeWipeConfig::default())),
            operations: Arc::new(RwLock::new(HashMap::new())),
            progress_sender,
            engine: SanitizationEngine::new(),
        }
    }

    pub fn with_config(mut self, config: SafeWipeConfig) -> Self {
        self.config = Arc::new(RwLock::new(config.clone()));
        self.engine = self.engine.with_real_device_access(config.real_device_access);
        self
    }

    /// Get current configuration
    pub async fn get_config(&self) -> SafeWipeConfig {
        self.config.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: SafeWipeConfig) -> Result<()> {
        *self.config.write().await = config.clone();
        // Update engine with new config
        // Note: This would require rebuilding the engine in a real implementation
        Ok(())
    }

    /// Subscribe to progress updates
    pub fn subscribe_progress(&self) -> broadcast::Receiver<WipeProgress> {
        self.progress_sender.subscribe()
    }

    /// List all available storage devices
    pub async fn list_devices(&self) -> ApiResponse<Vec<DeviceInfo>> {
        match self.scan_devices().await {
            Ok(devices) => ApiResponse::success(devices),
            Err(e) => ApiResponse::error(format!("Failed to scan devices: {}", e)),
        }
    }

    async fn scan_devices(&self) -> Result<Vec<DeviceInfo>> {
        let mut detector = DriveDetector::new();
        let devices = detector.list_devices().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(devices.into_iter().map(DeviceInfo::from).collect())
    }

    /// Get detailed information about a specific device
    pub async fn get_device_info(&self, device_id: &str) -> ApiResponse<DeviceInfo> {
        match self.find_device_by_id(device_id).await {
            Ok(Some(device)) => ApiResponse::success(device),
            Ok(None) => ApiResponse::error("Device not found".to_string()),
            Err(e) => ApiResponse::error(format!("Error getting device info: {}", e)),
        }
    }

    async fn find_device_by_id(&self, device_id: &str) -> Result<Option<DeviceInfo>> {
        let devices = self.scan_devices().await?;
        Ok(devices.into_iter().find(|d| d.id == device_id))
    }

    /// Start a wipe operation
    pub async fn start_wipe(
        &self,
        device_id: &str,
        method: SanitizationMethod,
    ) -> ApiResponse<String> {
        // Find the device
        let device_info = match self.find_device_by_id(device_id).await {
            Ok(Some(info)) => info,
            Ok(None) => return ApiResponse::error("Device not found".to_string()),
            Err(e) => return ApiResponse::error(format!("Error finding device: {}", e)),
        };

        // Safety checks
        let config = self.config.read().await;
        if device_info.is_system_drive && !config.allow_system_drives {
            return ApiResponse::error("Cannot wipe system drive. Enable in settings if required.".to_string());
        }

        let operation_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let operation = WipeOperation {
            id: operation_id.clone(),
            device_id: device_id.to_string(),
            device_path: device_info.path.clone(),
            method: method.clone(),
            status: WipeStatus::Starting,
            progress: None,
            result: None,
            created_at: now,
            updated_at: now,
        };

        // Store operation
        self.operations.write().await.insert(operation_id.clone(), operation);

        // Execute the operation directly for now (in a real implementation, you'd use a task queue)
        if let Err(e) = self.execute_wipe_operation(operation_id.clone()).await {
            eprintln!("Wipe operation failed: {}", e);
            // Update operation status to failed
            let mut ops = self.operations.write().await;
            if let Some(op) = ops.get_mut(&operation_id) {
                op.status = WipeStatus::Failed(e.to_string());
                op.updated_at = Utc::now();
            }
            return ApiResponse::error(format!("Wipe operation failed: {}", e));
        }
        ApiResponse::success(operation_id)
    }

    /// Internal: Execute a wipe operation and update progress
    async fn execute_wipe_operation(&self, operation_id: String) -> anyhow::Result<()> {
        let op = {
            let ops = self.operations.read().await;
            ops.get(&operation_id).cloned()
        };
        let op = match op {
            Some(o) => o,
            None => return Err(anyhow::anyhow!("Operation not found")),
        };
        let mut engine = SanitizationEngine::new();
        let device_path = op.device_path.clone();
        let method = op.method.clone();
        let sender = self.progress_sender.clone();
        engine = engine.with_progress_callback(move |progress| {
            let _ = sender.send(progress.clone());
        });
        // Find the device struct for sanitize_device
        let mut detector = DriveDetector::new();
        let device = detector.list_devices()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .into_iter()
            .find(|d| d.path == device_path)
            .ok_or_else(|| anyhow::anyhow!("Device not found for wipe operation"))?;
        // Run the wipe (async)
        let result = engine.sanitize_device(&device, method).await.map_err(|e| anyhow::anyhow!(e))?;
        // Update operation with result
        let mut ops = self.operations.write().await;
        if let Some(op) = ops.get_mut(&operation_id) {
            op.status = WipeStatus::Completed;
            op.result = Some(result);
            op.updated_at = Utc::now();
        }
        Ok(())
    }

    /// List all available sanitization methods
    pub fn list_sanitization_methods(&self) -> ApiResponse<Vec<SanitizationMethod>> {
        let methods = vec![
            SanitizationMethod::Clear,
            SanitizationMethod::Purge,
            SanitizationMethod::Destroy,
        ];
        ApiResponse::success(methods)
    }

    /// Get recommended sanitization methods for all devices
    pub async fn get_recommendations(&self) -> ApiResponse<HashMap<String, String>> {
        match self.scan_devices().await {
            Ok(devices) => {
                let mut map = HashMap::new();
                for d in devices {
                    map.insert(d.id.clone(), d.recommended_method.clone());
                }
                ApiResponse::success(map)
            },
            Err(e) => ApiResponse::error(format!("Failed to get recommendations: {}", e)),
        }
    }

    /// Verify a device after wipe
    pub async fn verify_device(&self, device_id: &str) -> ApiResponse<bool> {
        let device = match self.find_device_by_id(device_id).await {
            Ok(Some(info)) => info,
            Ok(None) => return ApiResponse::error("Device not found".to_string()),
            Err(e) => return ApiResponse::error(format!("Error finding device: {}", e)),
        };
        match verify_device(&device.path) {
            Ok(result) => ApiResponse::success(result),
            Err(e) => ApiResponse::error(format!("Verification failed: {}", e)),
        }
    }

    /// Generate a report for a device
    pub async fn generate_report(&self, device_id: &str) -> ApiResponse<Report> {
        let device = match self.find_device_by_id(device_id).await {
            Ok(Some(info)) => info,
            Ok(None) => return ApiResponse::error("Device not found".to_string()),
            Err(e) => return ApiResponse::error(format!("Error finding device: {}", e)),
        };
        match generate_report(&device.path) {
            Ok(report) => ApiResponse::success(report),
            Err(e) => ApiResponse::error(format!("Report generation failed: {}", e)),
        }
    }

    /// Get status of a wipe operation
    pub async fn get_operation_status(&self, operation_id: &str) -> ApiResponse<WipeOperation> {
        let operations = self.operations.read().await;
        match operations.get(operation_id) {
            Some(operation) => ApiResponse::success(operation.clone()),
            None => ApiResponse::error("Operation not found".to_string()),
        }
    }

    /// List all operations
    pub async fn list_operations(&self) -> ApiResponse<Vec<WipeOperation>> {
        let operations = self.operations.read().await;
        let ops: Vec<WipeOperation> = operations.values().cloned().collect();
        ApiResponse::success(ops)
    }

    /// Cancel a running operation
    pub async fn cancel_operation(&self, operation_id: &str) -> ApiResponse<bool> {
        let mut operations = self.operations.write().await;
        if let Some(operation) = operations.get_mut(operation_id) {
            if matches!(operation.status, WipeStatus::Starting | WipeStatus::InProgress) {
                operation.status = WipeStatus::Aborted;
                operation.updated_at = Utc::now();
                return ApiResponse::success(true);
            }
        }
        ApiResponse::error("Operation not found or cannot be cancelled".to_string())
    }

    /// Generate a report for a completed operation
    pub async fn generate_operation_report(&self, operation_id: &str) -> ApiResponse<Report> {
        let operations = self.operations.read().await;
        if let Some(operation) = operations.get(operation_id) {
            match generate_report(&operation.device_path) {
                Ok(report) => ApiResponse::success(report),
                Err(e) => ApiResponse::error(format!("Failed to generate report: {}", e)),
            }
        } else {
            ApiResponse::error("Operation not found".to_string())
        }
    }

    /// Get system statistics
    pub async fn get_system_stats(&self) -> ApiResponse<SystemStats> {
        let devices = match self.scan_devices().await {
            Ok(devices) => devices,
            Err(e) => return ApiResponse::error(format!("Failed to get system stats: {}", e)),
        };

        let operations = self.operations.read().await;

        let stats = SystemStats {
            total_devices: devices.len(),
            system_devices: devices.iter().filter(|d| d.is_system_drive).count(),
            wipeable_devices: devices.iter().filter(|d| !d.is_system_drive).count(),
            active_operations: operations.values()
                .filter(|op| matches!(op.status, WipeStatus::Starting | WipeStatus::InProgress | WipeStatus::Verifying))
                .count(),
            completed_operations: operations.values()
                .filter(|op| matches!(op.status, WipeStatus::Completed))
                .count(),
            failed_operations: operations.values()
                .filter(|op| matches!(op.status, WipeStatus::Failed(_)))
                .count(),
        };

        ApiResponse::success(stats)
    }
}

impl Clone for SafeWipeClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            operations: self.operations.clone(),
            progress_sender: self.progress_sender.clone(),
            engine: SanitizationEngine::new(), // Note: Engine doesn't support clone, so we create new
        }
    }
}

// Documented: All public methods of SafeWipeClient are async (except list_sanitization_methods),
// return ApiResponse<T> for UI use, and are robust for cross-platform use.
// This SDK can be used by any UI (Tauri, CLI, FFI, etc.) on Windows, Linux, or macOS.

// Utility functions

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn get_supported_methods(device: &Device) -> Vec<String> {
    let mut methods = vec!["clear".to_string()];

    match device.device_type {
        DriveType::SSD => {
            if device.capabilities.supports_ata_secure_erase {
                methods.push("purge".to_string());
            }
            if device.capabilities.supports_nvme_sanitize {
                methods.push("purge".to_string());
            }
            if device.capabilities.supports_crypto_erase {
                methods.push("purge".to_string());
            }
        }
        DriveType::HDD => {
            if device.capabilities.supports_ata_secure_erase {
                methods.push("purge".to_string());
            }
        }
        _ => {}
    }

    methods.push("destroy".to_string());
    methods.sort();
    methods.dedup();
    methods
}

// Remove device_info_to_device and legacy wrappers that reference it
// Fix generate_wipe_report and verify_device_wipe wrappers to match anyhow::Result return type
pub fn generate_wipe_report(device_path: &str) -> std::result::Result<Report, std::io::Error> {
    generate_report(device_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn verify_device_wipe(device_path: &str) -> std::result::Result<bool, std::io::Error> {
    verify_device(device_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_client_creation() {
        let client = SafeWipeClient::new();
        let config = Runtime::new().unwrap().block_on(client.get_config());
        assert!(!config.real_device_access);
        assert!(config.auto_verify);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_api_response() {
        let success_response = ApiResponse::success("test".to_string());
        assert!(success_response.success);
        assert_eq!(success_response.data, Some("test".to_string()));

        let error_response: ApiResponse<String> = ApiResponse::error("error message".to_string());
        assert!(!error_response.success);
        assert_eq!(error_response.error, Some("error message".to_string()));
    }
}
