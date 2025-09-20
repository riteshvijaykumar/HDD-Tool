use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::device::{Device, DriveDetector, DriveType};
use crate::wipe::{SanitizationEngine, SanitizationMethod, WipeResult, WipeProgress};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationPlan {
    pub devices: Vec<Device>,
    pub method: SanitizationMethod,
    pub estimated_duration: std::time::Duration,
    pub safety_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationReport {
    pub plan: SanitizationPlan,
    pub results: Vec<WipeResult>,
    pub overall_success: bool,
    pub total_duration: std::time::Duration,
    pub summary: String,
}

pub struct SafeWipeController {
    detector: DriveDetector,
    engine: SanitizationEngine,
}

impl SafeWipeController {
    pub fn new() -> Self {
        Self {
            detector: DriveDetector::new(),
            engine: SanitizationEngine::new(),
        }
    }

    pub fn with_progress_callback<F>(mut self, callback: F) -> Self 
    where 
        F: Fn(&WipeProgress) + Send + Sync + 'static 
    {
        self.engine = self.engine.with_progress_callback(callback);
        self
    }

    /// Scan and list all available drives
    pub async fn scan_drives(&mut self) -> Result<Vec<Device>> {
        println!("🔍 Scanning for storage devices...");
        let devices = self.detector.list_devices()?;
        
        println!("Found {} storage device(s):", devices.len());
        for device in &devices {
            println!("  📦 {} ({}) - {} - {:.2} GB", 
                device.name, 
                device.path,
                format!("{:?}", device.device_type),
                device.size as f64 / 1_000_000_000.0
            );
            
            if device.is_system_drive {
                println!("    ⚠️  SYSTEM DRIVE - Cannot be sanitized");
            }
            
            println!("    🔧 Capabilities:");
            println!("      ATA Secure Erase: {}", device.capabilities.supports_ata_secure_erase);
            println!("      NVMe Sanitize: {}", device.capabilities.supports_nvme_sanitize);
            println!("      Crypto Erase: {}", device.capabilities.supports_crypto_erase);
            println!("    📋 Recommended method: {}", device.recommended_sanitization_method());
            println!();
        }
        
        Ok(devices)
    }

    /// Create a sanitization plan for selected devices
    pub fn create_sanitization_plan(
        &self, 
        devices: Vec<Device>, 
        method: SanitizationMethod
    ) -> Result<SanitizationPlan> {
        let mut safety_warnings = Vec::new();
        let mut estimated_duration = std::time::Duration::new(0, 0);

        // Check for safety issues and estimate duration
        for device in &devices {
            if device.is_system_drive {
                safety_warnings.push(format!("⚠️ {} is a system drive and will be skipped", device.name));
                continue;
            }

            // Estimate duration based on device type and method
            let device_duration = match (&method, &device.device_type) {
                (SanitizationMethod::Clear, DriveType::HDD) => {
                    // HDD clear: ~50 MB/s throughput
                    std::time::Duration::from_secs(device.size / (50 * 1024 * 1024))
                }
                (SanitizationMethod::Clear, DriveType::SSD) => {
                    // SSD clear: ~200 MB/s throughput
                    std::time::Duration::from_secs(device.size / (200 * 1024 * 1024))
                }
                (SanitizationMethod::Clear, DriveType::Removable) => {
                    // Removable: ~20 MB/s throughput
                    std::time::Duration::from_secs(device.size / (20 * 1024 * 1024))
                }
                (SanitizationMethod::Clear, DriveType::Unknown) => {
                    // Conservative estimate for unknown types
                    std::time::Duration::from_secs(device.size / (10 * 1024 * 1024))
                }
                (SanitizationMethod::Purge, _) => {
                    // Purge operations are typically faster
                    std::time::Duration::from_secs(60) // Estimate 1 minute
                }
                (SanitizationMethod::Destroy, _) => {
                    // Destroy just generates instructions
                    std::time::Duration::from_secs(5)
                }
            };

            estimated_duration += device_duration;

            // Check method compatibility
            match method {
                SanitizationMethod::Purge => {
                    if !device.supports_secure_erase() {
                        safety_warnings.push(format!("⚠️ {} does not support secure purge methods", device.name));
                    }
                }
                _ => {}
            }
        }

        // Add general safety warnings
        match method {
            SanitizationMethod::Clear | SanitizationMethod::Purge => {
                safety_warnings.push("⚠️ This operation will PERMANENTLY destroy all data".to_string());
                safety_warnings.push("⚠️ Ensure you have backups of any important data".to_string());
                safety_warnings.push("⚠️ This operation cannot be undone".to_string());
            }
            SanitizationMethod::Destroy => {
                safety_warnings.push("⚠️ Physical destruction requires manual intervention".to_string());
                safety_warnings.push("⚠️ Follow all safety protocols when handling storage media".to_string());
            }
        }

        Ok(SanitizationPlan {
            devices,
            method,
            estimated_duration,
            safety_warnings,
        })
    }

    /// Execute a sanitization plan
    pub async fn execute_plan(&self, plan: SanitizationPlan) -> Result<SanitizationReport> {
        println!("🚀 Starting sanitization operation...");
        println!("Method: {:?}", plan.method);
        println!("Estimated duration: {:?}", plan.estimated_duration);
        println!();

        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        let mut overall_success = true;

        for device in &plan.devices {
            if device.is_system_drive {
                println!("⏭️ Skipping system drive: {}", device.name);
                continue;
            }

            println!("🔄 Processing device: {} ({})", device.name, device.path);
            
            match self.engine.sanitize_device(device, plan.method.clone()).await {
                Ok(result) => {
                    let success = matches!(result.status, crate::wipe::WipeStatus::Completed);
                    println!("✅ Device {} sanitization completed: {}", device.name, 
                        if success { "SUCCESS" } else { "FAILED" });
                    
                    if !success {
                        overall_success = false;
                    }
                    
                    results.push(result);
                }
                Err(e) => {
                    println!("❌ Device {} sanitization failed: {}", device.name, e);
                    overall_success = false;
                    
                    // Create a failed result
                    let failed_result = WipeResult {
                        id: uuid::Uuid::new_v4().to_string(),
                        device: device.clone(),
                        method: plan.method.clone(),
                        started_at: chrono::Utc::now(),
                        completed_at: Some(chrono::Utc::now()),
                        duration: Some(std::time::Duration::from_secs(0)),
                        status: crate::wipe::WipeStatus::Failed(e.to_string()),
                        verification_passed: false,
                        error_message: Some(e.to_string()),
                        patterns_used: Vec::new(),
                    };
                    results.push(failed_result);
                }
            }
            println!();
        }

        let total_duration = start_time.elapsed();
        
        let summary = if overall_success {
            format!("✅ All {} device(s) successfully sanitized in {:?}", 
                results.len(), total_duration)
        } else {
            let successful = results.iter()
                .filter(|r| matches!(r.status, crate::wipe::WipeStatus::Completed))
                .count();
            format!("⚠️ {}/{} device(s) successfully sanitized in {:?}", 
                successful, results.len(), total_duration)
        };

        println!("{}", summary);

        Ok(SanitizationReport {
            plan,
            results,
            overall_success,
            total_duration,
            summary,
        })
    }

    /// Get device recommendations
    pub fn get_recommendations(&self, devices: &[Device]) -> HashMap<String, String> {
        let mut recommendations = HashMap::new();

        for device in devices {
            let recommendation = if device.is_system_drive {
                "❌ System drive - cannot sanitize safely".to_string()
            } else {
                match device.device_type {
                    DriveType::SSD => {
                        if device.capabilities.supports_crypto_erase {
                            "🔒 Use PURGE with Crypto Erase for fastest, most secure sanitization".to_string()
                        } else if device.capabilities.supports_nvme_sanitize {
                            "⚡ Use PURGE with NVMe Sanitize for fast, secure sanitization".to_string()
                        } else {
                            "🔄 Use CLEAR with random pattern to minimize wear on SSD".to_string()
                        }
                    }
                    DriveType::HDD => {
                        if device.capabilities.supports_ata_secure_erase {
                            "⚡ Use PURGE with ATA Secure Erase for fastest sanitization".to_string()
                        } else {
                            "🔄 Use CLEAR with DoD 5220.22-M (3-pass) for thorough sanitization".to_string()
                        }
                    }
                    DriveType::Removable => {
                        "🔄 Use CLEAR with single-pass for removable media".to_string()
                    }
                    DriveType::Unknown => {
                        "⚠️ Use CLEAR with conservative settings for unknown device type".to_string()
                    }
                }
            };

            recommendations.insert(device.name.clone(), recommendation);
        }

        recommendations
    }
}
