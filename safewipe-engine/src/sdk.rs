use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};
use anyhow::Result;

use crate::device::{Device, DriveDetector};
use crate::sanitization::{SafeWipeController, SanitizationPlan, SanitizationReport};
use crate::wipe::{SanitizationMethod, WipeProgress, WipeResult};

/// Main SDK interface for SafeWipe functionality
///
/// This provides a high-level API for integrating SafeWipe capabilities
/// into other applications without needing HTTP/web interface.
pub struct SafeWipeSDK {
    controller: Arc<Mutex<SafeWipeController>>,
    detector: Arc<Mutex<DriveDetector>>,
    active_operations: Arc<Mutex<HashMap<String, SanitizationReport>>>,
    progress_tx: broadcast::Sender<ProgressEvent>,
}

/// Progress event emitted during sanitization operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub operation_id: String,
    pub device_name: String,
    pub progress: WipeProgress,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// SDK configuration options
#[derive(Debug, Clone)]
pub struct SDKConfig {
    /// Enable progress callbacks
    pub enable_progress_callbacks: bool,
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,
    /// Custom progress callback buffer size
    pub progress_buffer_size: usize,
}

impl Default for SDKConfig {
    fn default() -> Self {
        Self {
            enable_progress_callbacks: true,
            max_concurrent_operations: 4,
            progress_buffer_size: 100,
        }
    }
}

/// Result type for SDK operations
pub type SDKResult<T> = Result<T, SafeWipeError>;

/// SDK-specific error types
#[derive(Debug, thiserror::Error)]
pub enum SafeWipeError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Operation not found: {0}")]
    OperationNotFound(String),

    #[error("Invalid sanitization method: {0}")]
    InvalidMethod(String),

    #[error("System drive cannot be sanitized: {0}")]
    SystemDriveProtected(String),

    #[error("Operation already in progress: {0}")]
    OperationInProgress(String),

    #[error("SDK internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl SafeWipeSDK {
    /// Create a new SafeWipe SDK instance
    pub fn new() -> Self {
        Self::with_config(SDKConfig::default())
    }

    /// Create a new SafeWipe SDK instance with custom configuration
    pub fn with_config(config: SDKConfig) -> Self {
        let (progress_tx, _) = broadcast::channel(config.progress_buffer_size);

        Self {
            controller: Arc::new(Mutex::new(SafeWipeController::new())),
            detector: Arc::new(Mutex::new(DriveDetector::new())),
            active_operations: Arc::new(Mutex::new(HashMap::new())),
            progress_tx,
        }
    }

    /// Subscribe to progress events
    ///
    /// Returns a receiver that will get progress updates for all operations
    pub fn subscribe_to_progress(&self) -> broadcast::Receiver<ProgressEvent> {
        self.progress_tx.subscribe()
    }

    /// Scan and discover all storage devices
    pub async fn scan_devices(&self) -> SDKResult<Vec<Device>> {
        let mut detector = self.detector.lock()
            .map_err(|e| SafeWipeError::Internal(anyhow::anyhow!("Lock error: {}", e)))?;

        detector.list_devices()
            .map_err(SafeWipeError::Internal)
    }

    /// Get a specific device by name
    pub async fn get_device(&self, name: &str) -> SDKResult<Device> {
        let devices = self.scan_devices().await?;
        devices.into_iter()
            .find(|d| d.name == name)
            .ok_or_else(|| SafeWipeError::DeviceNotFound(name.to_string()))
    }

    /// Get sanitization recommendations for all devices
    pub async fn get_recommendations(&self) -> SDKResult<HashMap<String, String>> {
        let mut controller = self.controller.lock()
            .map_err(|e| SafeWipeError::Internal(anyhow::anyhow!("Lock error: {}", e)))?;

        let devices = controller.scan_drives().await
            .map_err(SafeWipeError::Internal)?;

        Ok(controller.get_recommendations(&devices))
    }

    /// Get sanitization recommendation for a specific device
    pub async fn get_device_recommendation(&self, device_name: &str) -> SDKResult<String> {
        let device = self.get_device(device_name).await?;
        let mut controller = self.controller.lock()
            .map_err(|e| SafeWipeError::Internal(anyhow::anyhow!("Lock error: {}", e)))?;

        let recommendations = controller.get_recommendations(&[device]);
        Ok(recommendations.get(device_name)
            .cloned()
            .unwrap_or_else(|| "No recommendation available".to_string()))
    }

    /// Create a sanitization plan for specific devices
    pub async fn create_sanitization_plan(
        &self,
        device_names: &[String],
        method: SanitizationMethod
    ) -> SDKResult<SanitizationPlan> {
        let mut controller = self.controller.lock()
            .map_err(|e| SafeWipeError::Internal(anyhow::anyhow!("Lock error: {}", e)))?;

        let all_devices = controller.scan_drives().await
            .map_err(SafeWipeError::Internal)?;

        let selected_devices: Vec<Device> = all_devices
            .into_iter()
            .filter(|d| device_names.contains(&d.name))
            .collect();

        if selected_devices.is_empty() {
            return Err(SafeWipeError::DeviceNotFound("No valid devices found".to_string()));
        }

        // Check for system drives
        for device in &selected_devices {
            if device.is_system_drive {
                return Err(SafeWipeError::SystemDriveProtected(device.name.clone()));
            }
        }

        controller.create_sanitization_plan(selected_devices, method)
            .map_err(SafeWipeError::Internal)
    }

    /// Execute sanitization on specified devices
    ///
    /// Returns an operation ID that can be used to track progress
    pub async fn sanitize_devices(
        &self,
        device_names: &[String],
        method: SanitizationMethod
    ) -> SDKResult<String> {
        let plan = self.create_sanitization_plan(device_names, method).await?;
        self.execute_sanitization_plan(plan).await
    }

    /// Execute a pre-created sanitization plan
    pub async fn execute_sanitization_plan(&self, plan: SanitizationPlan) -> SDKResult<String> {
        let operation_id = uuid::Uuid::new_v4().to_string();
        let op_id_clone = operation_id.clone();

        let controller = self.controller.clone();
        let operations = self.active_operations.clone();
        let progress_tx = self.progress_tx.clone();

        // Spawn background task for sanitization
        tokio::spawn(async move {
            let progress_controller = SafeWipeController::new()
                .with_progress_callback({
                    let tx = progress_tx.clone();
                    let op_id = op_id_clone.clone();
                    move |progress| {
                        let event = ProgressEvent {
                            operation_id: op_id.clone(),
                            device_name: progress.device_path.clone(),
                            progress: progress.clone(),
                            timestamp: chrono::Utc::now(),
                        };
                        let _ = tx.send(event);
                    }
                });

            match progress_controller.execute_plan(plan).await {
                Ok(report) => {
                    operations.lock().unwrap().insert(op_id_clone, report);
                }
                Err(e) => {
                    eprintln!("Sanitization failed: {}", e);
                }
            }
        });

        Ok(operation_id)
    }

    /// Get the status of a running operation
    pub fn get_operation_status(&self, operation_id: &str) -> SDKResult<Option<SanitizationReport>> {
        let operations = self.active_operations.lock()
            .map_err(|e| SafeWipeError::Internal(anyhow::anyhow!("Lock error: {}", e)))?;

        Ok(operations.get(operation_id).cloned())
    }

    /// Get all operation statuses
    pub fn get_all_operations(&self) -> SDKResult<HashMap<String, SanitizationReport>> {
        let operations = self.active_operations.lock()
            .map_err(|e| SafeWipeError::Internal(anyhow::anyhow!("Lock error: {}", e)))?;

        Ok(operations.clone())
    }

    /// Wait for an operation to complete
    pub async fn wait_for_operation(&self, operation_id: &str) -> SDKResult<SanitizationReport> {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));

        for _ in 0..240 { // Wait up to 2 minutes
            interval.tick().await;

            if let Some(report) = self.get_operation_status(operation_id)? {
                if report.overall_success || !matches!(report.results.first().map(|r| &r.status), Some(crate::wipe::WipeStatus::InProgress) | Some(crate::wipe::WipeStatus::Starting)) {
                    return Ok(report);
                }
            }
        }

        Err(SafeWipeError::OperationNotFound(format!("Operation {} did not complete within timeout", operation_id)))
    }

    /// Cancel a running operation (if possible)
    pub async fn cancel_operation(&self, operation_id: &str) -> SDKResult<bool> {
        // Note: Actual cancellation would require more complex implementation
        // This is a placeholder for future enhancement
        let mut operations = self.active_operations.lock()
            .map_err(|e| SafeWipeError::Internal(anyhow::anyhow!("Lock error: {}", e)))?;

        if operations.contains_key(operation_id) {
            operations.remove(operation_id);
            Ok(true)
        } else {
            Err(SafeWipeError::OperationNotFound(operation_id.to_string()))
        }
    }

    /// Get system health and status
    pub async fn get_system_status(&self) -> SDKResult<SystemStatus> {
        let devices = self.scan_devices().await?;
        let operations = self.get_all_operations()?;

        let system_drives = devices.iter().filter(|d| d.is_system_drive).count();
        let sanitizable_drives = devices.iter().filter(|d| !d.is_system_drive).count();
        let active_operations = operations.len();

        Ok(SystemStatus {
            total_devices: devices.len(),
            system_drives,
            sanitizable_drives,
            active_operations,
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub total_devices: usize,
    pub system_drives: usize,
    pub sanitizable_drives: usize,
    pub active_operations: usize,
    pub sdk_version: String,
}

/// Convenience functions for common operations
impl SafeWipeSDK {
    /// Quick sanitize - sanitize a single device with recommended method
    pub async fn quick_sanitize(&self, device_name: &str) -> SDKResult<String> {
        let device = self.get_device(device_name).await?;

        if device.is_system_drive {
            return Err(SafeWipeError::SystemDriveProtected(device_name.to_string()));
        }

        // Use recommended method based on device type
        let method = match device.device_type {
            crate::device::DriveType::SSD => {
                if device.supports_secure_erase() {
                    SanitizationMethod::Purge
                } else {
                    SanitizationMethod::Clear
                }
            }
            crate::device::DriveType::HDD => {
                if device.supports_secure_erase() {
                    SanitizationMethod::Purge
                } else {
                    SanitizationMethod::Clear
                }
            }
            _ => SanitizationMethod::Clear,
        };

        self.sanitize_devices(&[device_name.to_string()], method).await
    }

    /// Batch sanitize - sanitize multiple devices with the same method
    pub async fn batch_sanitize(&self, device_names: &[String], method: SanitizationMethod) -> SDKResult<Vec<String>> {
        let mut operation_ids = Vec::new();

        for device_name in device_names {
            let op_id = self.sanitize_devices(&[device_name.clone()], method.clone()).await?;
            operation_ids.push(op_id);
        }

        Ok(operation_ids)
    }

    /// Generate compliance report for completed operations
    pub fn generate_compliance_report(&self, operation_ids: &[String]) -> SDKResult<ComplianceReport> {
        let operations = self.get_all_operations()?;
        let mut reports = Vec::new();

        for op_id in operation_ids {
            if let Some(report) = operations.get(op_id) {
                reports.push(report.clone());
            }
        }

        Ok(ComplianceReport {
            generated_at: chrono::Utc::now(),
            operation_count: reports.len(),
            reports,
            standard: "NIST SP 800-88 Rev. 1".to_string(),
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

/// Compliance report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub operation_count: usize,
    pub reports: Vec<SanitizationReport>,
    pub standard: String,
    pub sdk_version: String,
}

/// Builder pattern for SDK configuration
pub struct SafeWipeSDKBuilder {
    config: SDKConfig,
}

impl SafeWipeSDKBuilder {
    pub fn new() -> Self {
        Self {
            config: SDKConfig::default(),
        }
    }

    pub fn enable_progress_callbacks(mut self, enabled: bool) -> Self {
        self.config.enable_progress_callbacks = enabled;
        self
    }

    pub fn max_concurrent_operations(mut self, max: usize) -> Self {
        self.config.max_concurrent_operations = max;
        self
    }

    pub fn progress_buffer_size(mut self, size: usize) -> Self {
        self.config.progress_buffer_size = size;
        self
    }

    pub fn build(self) -> SafeWipeSDK {
        SafeWipeSDK::with_config(self.config)
    }
}

impl Default for SafeWipeSDKBuilder {
    fn default() -> Self {
        Self::new()
    }
}
