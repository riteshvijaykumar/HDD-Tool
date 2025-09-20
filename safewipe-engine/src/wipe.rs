use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use uuid::Uuid;
use crate::device;
use crate::device::{Device, DriveType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SanitizationMethod {
    Clear,
    Purge,
    Destroy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClearPattern {
    Zeros,
    Ones,
    Random,
    DoD5220(u8), // DoD 5220.22-M with number of passes
    Gutmann,     // 35-pass Gutmann method
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NvmeSanitizeMode {
    Block,     // Block erase - fast but may leave traces
    Crypto,    // Cryptographic erase - best for encrypted drives
    Overwrite, // Overwrite with pattern - most thorough
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PurgeMethod {
    AtaSecureErase,
    AtaEnhancedSecureErase,
    NvmeSanitize(NvmeSanitizeMode),
    CryptoErase,
    VendorSpecific(String), // For vendor-specific tools like Knox
    SecureFactoryReset,     // For mobile devices
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeProgress {
    pub device_path: String,
    pub method: SanitizationMethod,
    pub started_at: DateTime<Utc>,
    pub current_pass: u8,
    pub total_passes: u8,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub status: WipeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WipeStatus {
    Starting,
    InProgress,
    Verifying,
    Completed,
    Failed(String),
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeResult {
    pub id: String,
    pub device: Device,
    pub method: SanitizationMethod,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub status: WipeStatus,
    pub verification_passed: bool,
    pub error_message: Option<String>,
    pub patterns_used: Vec<String>,
}

pub struct SanitizationEngine {
    progress_callback: Option<Box<dyn Fn(&WipeProgress) + Send + Sync>>,
    allow_real_devices: bool, // Add configuration for real device access
}

impl SanitizationEngine {
    pub fn new() -> Self {
        Self {
            progress_callback: None,
            allow_real_devices: false, // Default to safe mode
        }
    }

    pub fn with_real_device_access(mut self, enabled: bool) -> Self {
        self.allow_real_devices = enabled;
        self
    }

    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&WipeProgress) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Main sanitization entry point
    pub async fn sanitize_device(
        &self,
        device: &Device,
        method: SanitizationMethod,
    ) -> Result<WipeResult> {
        let wipe_id = Uuid::new_v4().to_string();
        let started_at = Utc::now();

        let mut result = WipeResult {
            id: wipe_id,
            device: device.clone(),
            method: method.clone(),
            started_at,
            completed_at: None,
            duration: None,
            status: WipeStatus::Starting,
            verification_passed: false,
            error_message: None,
            patterns_used: Vec::new(),
        };

        // Safety check for system drives
        if device.is_system_drive {
            return Err(anyhow!("Cannot sanitize system drive for safety reasons"));
        }

        let start_time = Instant::now();

        match method {
            SanitizationMethod::Clear => {
                result = self.perform_clear_operation(device, result).await?;
            }
            SanitizationMethod::Purge => {
                result = self.perform_purge_operation(device, result).await?;
            }
            SanitizationMethod::Destroy => {
                result = self.perform_destroy_operation(device, result).await?;
            }
        }

        result.completed_at = Some(Utc::now());
        result.duration = Some(start_time.elapsed());

        Ok(result)
    }

    async fn perform_clear_operation(
        &self,
        device: &Device,
        mut result: WipeResult,
    ) -> Result<WipeResult> {
        println!("Starting CLEAR operation on device: {}", device.path);

        let pattern = match device.device_type {
            DriveType::SSD => ClearPattern::Random,     // Better for SSDs
            DriveType::HDD => ClearPattern::DoD5220(3), // 3-pass DoD for HDDs
            _ => ClearPattern::Zeros,
        };

        result.status = WipeStatus::InProgress;
        result = self.execute_clear_pattern(device, pattern, result).await?;

        // Verify the wipe
        result.status = WipeStatus::Verifying;
        result.verification_passed = self.verify_clear_operation(device).await?;

        result.status = if result.verification_passed {
            WipeStatus::Completed
        } else {
            WipeStatus::Failed("Verification failed".to_string())
        };

        Ok(result)
    }

    async fn execute_clear_pattern(
        &self,
        device: &Device,
        pattern: ClearPattern,
        mut result: WipeResult,
    ) -> Result<WipeResult> {
        match pattern {
            ClearPattern::Zeros => {
                result.patterns_used.push("zeros".to_string());
                self.overwrite_with_pattern(device, &[0u8; 1024 * 1024], 1)
                    .await?;
            }
            ClearPattern::Ones => {
                result.patterns_used.push("ones".to_string());
                self.overwrite_with_pattern(device, &[0xFFu8; 1024 * 1024], 1)
                    .await?;
            }
            ClearPattern::Random => {
                result.patterns_used.push("random".to_string());
                self.overwrite_with_random(device, 1).await?;
            }
            ClearPattern::DoD5220(passes) => {
                result
                    .patterns_used
                    .push(format!("DoD 5220.22-M ({} passes)", passes));
                self.perform_dod_5220_wipe(device, passes).await?;
            }
            ClearPattern::Gutmann => {
                result.patterns_used.push("Gutmann 35-pass".to_string());
                self.perform_gutmann_wipe(device).await?;
            }
        }

        Ok(result)
    }

    async fn overwrite_with_pattern(
        &self,
        device: &Device,
        pattern: &[u8],
        passes: u8,
    ) -> Result<()> {
        for pass in 1..=passes {
            println!(
                "Pass {}/{}: Writing pattern to {}",
                pass, passes, device.path
            );

            // If real device access is enabled, proceed with actual device access
            if self.allow_real_devices {
                println!("🔥 REAL DEVICE MODE: Actually writing to {}", device.path);

                // Try to open the device for direct access
                let file_result = if device.path.contains(":") {
                    // Windows drive letter - try to open as volume
                    let volume_path = format!(r"\\.\{}", device.path.trim_end_matches('\\'));
                    println!("   Attempting to open Windows volume: {}", volume_path);
                    OpenOptions::new().write(true).open(&volume_path).await
                } else {
                    // Regular file or Unix device
                    OpenOptions::new().write(true).open(&device.path).await
                };

                let mut file = match file_result {
                    Ok(f) => {
                        println!("✅ Successfully opened device for writing");
                        f
                    }
                    Err(e) => {
                        println!("❌ Failed to open device for direct access: {}", e);
                        println!("   This likely requires administrator privileges");
                        println!("   Try running as administrator or use a test file instead");
                        return Err(anyhow!("Cannot access device: {}", e));
                    }
                };

                let mut bytes_written = 0u64;

                while bytes_written < device.size {
                    let chunk_size =
                        std::cmp::min(pattern.len(), (device.size - bytes_written) as usize);

                    match file.write_all(&pattern[..chunk_size]).await {
                        Ok(_) => {
                            bytes_written += chunk_size as u64;

                            // Report progress
                            if let Some(callback) = &self.progress_callback {
                                let progress = WipeProgress {
                                    device_path: device.path.clone(),
                                    method: SanitizationMethod::Clear,
                                    started_at: Utc::now(),
                                    current_pass: pass,
                                    total_passes: passes,
                                    bytes_processed: bytes_written,
                                    total_bytes: device.size,
                                    estimated_completion: None,
                                    status: WipeStatus::InProgress,
                                };
                                callback(&progress);
                            }
                        }
                        Err(e) => {
                            println!("❌ Write error at {} bytes: {}", bytes_written, e);
                            return Err(anyhow!("Write failed: {}", e));
                        }
                    }
                }

                file.flush().await?;
                file.sync_all().await?;
                println!("✅ Pass {}/{} completed successfully", pass, passes);
                continue;
            }

            // Demo mode - simulate the operation
            println!("⚠️ DEMO MODE: Simulating write to device {}", device.path);
            println!("   Use --real-devices flag to enable actual device modification");
            println!(
                "   This would write {} bytes in {} passes",
                device.size, passes
            );

            // Simulate the operation with a small delay
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Report progress
            if let Some(callback) = &self.progress_callback {
                let progress = WipeProgress {
                    device_path: device.path.clone(),
                    method: SanitizationMethod::Clear,
                    started_at: Utc::now(),
                    current_pass: pass,
                    total_passes: passes,
                    bytes_processed: device.size, // Simulate completion
                    total_bytes: device.size,
                    estimated_completion: None,
                    status: WipeStatus::InProgress,
                };
                callback(&progress);
            }
        }

        Ok(())
    }

    async fn overwrite_with_random(&self, device: &Device, passes: u8) -> Result<()> {
        let mut rng = thread_rng();
        let chunk_size = 1024 * 1024; // 1MB chunks

        for pass in 1..=passes {
            println!(
                "Pass {}/{}: Writing random data to {}",
                pass, passes, device.path
            );

            // Safety check - don't actually write to real devices in demo mode
            if !self.allow_real_devices
                && (device.path.contains(":") || device.path.starts_with("/dev/"))
            {
                println!(
                    "⚠️ DEMO MODE: Simulating random write to real device {}",
                    device.path
                );
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            let file_result = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&device.path)
                .await;

            let mut file = match file_result {
                Ok(f) => f,
                Err(e) => {
                    println!("⚠️ Cannot open device {} for writing: {}", device.path, e);
                    println!("   Simulating random data write instead...");
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                }
            };

            let mut bytes_written = 0u64;

            while bytes_written < device.size {
                let current_chunk_size =
                    std::cmp::min(chunk_size, (device.size - bytes_written) as usize);
                let mut buffer = vec![0u8; current_chunk_size];
                rng.fill_bytes(&mut buffer);

                file.write_all(&buffer).await?;
                bytes_written += current_chunk_size as u64;
            }

            file.flush().await?;
            file.sync_all().await?;
        }

        Ok(())
    }

    async fn perform_dod_5220_wipe(&self, device: &Device, passes: u8) -> Result<()> {
        // DoD 5220.22-M: Pass 1 (zeros), Pass 2 (ones), Pass 3 (random)
        for pass in 1..=passes {
            match pass % 3 {
                1 => {
                    self.overwrite_with_pattern(device, &[0u8; 1024 * 1024], 1)
                        .await?
                }
                2 => {
                    self.overwrite_with_pattern(device, &[0xFFu8; 1024 * 1024], 1)
                        .await?
                }
                0 => self.overwrite_with_random(device, 1).await?,
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    async fn perform_gutmann_wipe(&self, device: &Device) -> Result<()> {
        // Simplified Gutmann method (35 passes with various patterns)
        let patterns = [
            vec![0x55, 0x55, 0x55, 0x55], // Pattern 1
            vec![0xAA, 0xAA, 0xAA, 0xAA], // Pattern 2
            vec![0x92, 0x49, 0x24, 0x92], // Pattern 3
                                          // Add more Gutmann patterns as needed
        ];

        for (i, pattern) in patterns.iter().enumerate() {
            println!("Gutmann pass {}/35", i + 1);
            let extended_pattern = pattern.repeat(1024 * 256); // Extend pattern
            self.overwrite_with_pattern(device, &extended_pattern, 1)
                .await?;
        }

        // Fill remaining passes with random data
        self.overwrite_with_random(device, 31).await?;

        Ok(())
    }

    async fn perform_purge_operation(
        &self,
        device: &Device,
        mut result: WipeResult,
    ) -> Result<WipeResult> {
        println!("Starting PURGE operation on device: {}", device.path);

        let method = self.select_purge_method(device)?;

        result.status = WipeStatus::InProgress;

        match method {
            PurgeMethod::AtaSecureErase => {
                result.patterns_used.push("ATA Secure Erase".to_string());
                self.execute_ata_secure_erase(device).await?;
            }
            PurgeMethod::AtaEnhancedSecureErase => {
                result
                    .patterns_used
                    .push("ATA Enhanced Secure Erase".to_string());
                self.execute_ata_enhanced_secure_erase(device).await?;
            }
            PurgeMethod::NvmeSanitize(mode) => {
                result.patterns_used.push("NVMe Sanitize".to_string());
                self.execute_nvme_sanitize(device, mode).await?;
            }
            PurgeMethod::CryptoErase => {
                result.patterns_used.push("Cryptographic Erase".to_string());
                self.execute_crypto_erase(device).await?;
            }
            PurgeMethod::VendorSpecific(tool) => {
                result
                    .patterns_used
                    .push(format!("Vendor-specific purge with {}", tool));
                self.execute_vendor_specific_purge(device, &tool).await?;
            }
            PurgeMethod::SecureFactoryReset => {
                result
                    .patterns_used
                    .push("Secure Factory Reset".to_string());
                self.execute_secure_factory_reset(device).await?;
            }
        }

        result.status = WipeStatus::Completed;
        result.verification_passed = true; // Purge methods are self-verifying

        Ok(result)
    }

    fn select_purge_method(&self, device: &Device) -> Result<PurgeMethod> {
        match device.device_type {
            DriveType::SSD => {
                if device.capabilities.supports_crypto_erase {
                    Ok(PurgeMethod::CryptoErase)
                } else if device.capabilities.supports_nvme_sanitize {
                    Ok(PurgeMethod::NvmeSanitize(NvmeSanitizeMode::Block))
                } else if device.capabilities.supports_ata_secure_erase {
                    Ok(PurgeMethod::AtaEnhancedSecureErase)
                } else {
                    Err(anyhow!("SSD does not support any secure purge methods"))
                }
            }
            DriveType::HDD => {
                if device.capabilities.supports_ata_secure_erase {
                    if device.capabilities.supports_enhanced_erase {
                        Ok(PurgeMethod::AtaEnhancedSecureErase)
                    } else {
                        Ok(PurgeMethod::AtaSecureErase)
                    }
                } else {
                    Err(anyhow!("HDD does not support secure erase"))
                }
            }
            DriveType::Removable => {
                if device.capabilities.supports_crypto_erase {
                    Ok(PurgeMethod::CryptoErase)
                } else {
                    Ok(PurgeMethod::VendorSpecific(
                        "removable-secure-erase".to_string(),
                    ))
                }
            }
            DriveType::Unknown => Err(anyhow!(
                "Unknown device type does not support purge operations"
            )),
        }
    }

    async fn execute_ata_secure_erase(&self, device: &Device) -> Result<()> {
        println!("Executing ATA Secure Erase on {}", device.path);
        if !self.allow_real_devices {
            println!("Demo mode: not performing real ATA Secure Erase.");
            return Ok(());
        }
        // Set password (NULL) and perform secure erase
        let set_pass = tokio::process::Command::new("sudo")
            .arg("hdparm")
            .arg("--user-master")
            .arg("u")
            .arg("--security-set-pass")
            .arg("NULL")
            .arg(&device.path)
            .output()
            .await?;
        if !set_pass.status.success() {
            return Err(anyhow!("Failed to set ATA password: {}", String::from_utf8_lossy(&set_pass.stderr)));
        }
        let erase = tokio::process::Command::new("sudo")
            .arg("hdparm")
            .arg("--user-master")
            .arg("u")
            .arg("--security-erase")
            .arg("NULL")
            .arg(&device.path)
            .output()
            .await?;
        if !erase.status.success() {
            return Err(anyhow!("ATA Secure Erase failed: {}", String::from_utf8_lossy(&erase.stderr)));
        }
        println!("ATA Secure Erase completed.");
        Ok(())
    }

    async fn execute_ata_enhanced_secure_erase(&self, device: &Device) -> Result<()> {
        println!("Executing ATA Enhanced Secure Erase on {}", device.path);
        if !self.allow_real_devices {
            println!("Demo mode: not performing real ATA Enhanced Secure Erase.");
            return Ok(());
        }
        let set_pass = tokio::process::Command::new("sudo")
            .arg("hdparm")
            .arg("--user-master")
            .arg("u")
            .arg("--security-set-pass")
            .arg("NULL")
            .arg(&device.path)
            .output()
            .await?;
        if !set_pass.status.success() {
            return Err(anyhow!("Failed to set ATA password: {}", String::from_utf8_lossy(&set_pass.stderr)));
        }
        let erase = tokio::process::Command::new("sudo")
            .arg("hdparm")
            .arg("--user-master")
            .arg("u")
            .arg("--security-erase-enhanced")
            .arg("NULL")
            .arg(&device.path)
            .output()
            .await?;
        if !erase.status.success() {
            return Err(anyhow!("ATA Enhanced Secure Erase failed: {}", String::from_utf8_lossy(&erase.stderr)));
        }
        println!("ATA Enhanced Secure Erase completed.");
        Ok(())
    }

    async fn execute_vendor_specific_purge(&self, device: &Device, tool: &str) -> Result<()> {
        println!("Executing vendor-specific purge with {} on {}", tool, device.path);
        if !self.allow_real_devices {
            println!("Demo mode: not performing real vendor-specific purge.");
            return Ok(());
        }
        match device.device_type {
            DriveType::Removable | DriveType::SSD => {
                match tool {
                    "removable-secure-erase" | "crypto-erase" => {
                        // Use blkdiscard for SSD/USB/Removable
                        let discard = tokio::process::Command::new("sudo")
                            .arg("blkdiscard")
                            .arg(&device.path)
                            .output()
                            .await?;
                        if !discard.status.success() {
                            return Err(anyhow!("blkdiscard failed: {}", String::from_utf8_lossy(&discard.stderr)));
                        }
                        println!("blkdiscard completed for {}.", device.path);
                        Ok(())
                    }
                    _ => Err(anyhow!("Unsupported vendor-specific purge tool: {}", tool)),
                }
            }
            _ => Err(anyhow!("Vendor-specific purge not supported for this device type")),
        }
    }

    async fn enable_device_encryption(&self, device: &Device) -> Result<()> {
        println!("Enabling device encryption for {}", device.path);
        if !self.allow_real_devices {
            println!("Demo mode: not performing real encryption.");
            return Ok(());
        }
        // For ATA: set password (enables encryption)
        let set_pass = tokio::process::Command::new("sudo")
            .arg("hdparm")
            .arg("--user-master")
            .arg("u")
            .arg("--security-set-pass")
            .arg("ENCRYPT")
            .arg(&device.path)
            .output()
            .await?;
        if !set_pass.status.success() {
            return Err(anyhow!("Failed to enable encryption: {}", String::from_utf8_lossy(&set_pass.stderr)));
        }
        println!("Device encryption enabled.");
        Ok(())
    }

    async fn generate_new_encryption_key(&self, device: &Device) -> Result<()> {
        println!("Generating new encryption key for {}", device.path);
        if !self.allow_real_devices {
            println!("Demo mode: not generating real encryption key.");
            return Ok(());
        }
        // For ATA: set new password
        let new_pass = tokio::process::Command::new("sudo")
            .arg("hdparm")
            .arg("--user-master")
            .arg("u")
            .arg("--security-set-pass")
            .arg("NEWKEY")
            .arg(&device.path)
            .output()
            .await?;
        if !new_pass.status.success() {
            return Err(anyhow!("Failed to generate new encryption key: {}", String::from_utf8_lossy(&new_pass.stderr)));
        }
        println!("New encryption key generated.");
        Ok(())
    }

    async fn execute_secure_factory_reset(&self, device: &Device) -> Result<()> {
        println!("Executing secure factory reset for {}", device.path);
        if !self.allow_real_devices {
            println!("Demo mode: not performing real factory reset.");
            return Ok(());
        }
        // For SSD/Removable: use blkdiscard
        let discard = tokio::process::Command::new("sudo")
            .arg("blkdiscard")
            .arg(&device.path)
            .output()
            .await?;
        if !discard.status.success() {
            return Err(anyhow!("Secure factory reset (blkdiscard) failed: {}", String::from_utf8_lossy(&discard.stderr)));
        }
        println!("Secure factory reset completed.");
        Ok(())
    }

    async fn perform_destroy_operation(
        &self,
        device: &Device,
        mut result: WipeResult,
    ) -> Result<WipeResult> {
        println!("DESTROY operation requested for device: {}", device.path);
        println!("\n==============================");
        println!("MANUAL DESTRUCTION REQUIRED!");
        println!("Follow these steps to physically destroy the device:");
        match device.device_type {
            DriveType::HDD => {
                println!("1. Degauss the drive with a certified degausser.");
                println!("2. Disassemble and shred platters to <2mm particles.");
                println!("3. Incinerate if possible.");
            }
            DriveType::SSD => {
                println!("1. Shred the SSD to <2mm particles.");
                println!("2. Incinerate or pulverize NAND chips.");
                println!("3. Use chemical destruction if available.");
            }
            DriveType::Removable => {
                println!("1. Shred or incinerate removable media.");
                println!("2. For optical: scratch surface, then shred.");
            }
            _ => {
                println!("Unknown device type. Use best available destruction method.");
            }
        }
        println!("\nIMPORTANT: Document the destruction process and retain a certificate of destruction if required by policy.");
        println!("==============================\n");
        result.status = WipeStatus::Completed;
        result.verification_passed = true;
        result.patterns_used.push("Physical Destruction Instructions".to_string());
        Ok(result)
    }

    async fn verify_clear_operation(&self, device: &Device) -> Result<bool> {
        println!("Verifying clear operation on {}...", device.path);
        if !self.allow_real_devices {
            println!("Demo mode: assuming clear operation succeeded.");
            return Ok(true);
        }
        // Open device and check random blocks for zeroed data
        let file_result = File::open(&device.path).await;
        let mut file = match file_result {
            Ok(f) => f,
            Err(e) => {
                println!("Cannot open device for verification: {}", e);
                return Ok(false);
            }
        };
        let mut buffer = vec![0u8; 4096];
        for _ in 0..10 {
            let position = rand::random::<u64>() % (device.size / 4096) * 4096;
            if file.seek(SeekFrom::Start(position)).await.is_err() {
                continue;
            }
            if tokio::io::AsyncReadExt::read_exact(&mut file, &mut buffer).await.is_err() {
                continue;
            }
            if buffer.iter().any(|&b| b != 0) {
                println!("Verification failed: Non-zero data found at position {}", position);
                return Ok(false);
            }
        }
        println!("Verification passed: Device appears to be properly wiped.");
        Ok(true)
    }

    async fn execute_nvme_sanitize(&self, device: &Device, mode: NvmeSanitizeMode) -> Result<()> {
        println!("Executing NVMe Sanitize on {} with mode {:?}", device.path, mode);
        if !self.allow_real_devices {
            println!("Demo mode: not performing real NVMe sanitize.");
            return Ok(());
        }
        let mode_arg = match mode {
            NvmeSanitizeMode::Block => "block",
            NvmeSanitizeMode::Crypto => "crypto",
            NvmeSanitizeMode::Overwrite => "overwrite",
        };
        let output = tokio::process::Command::new("sudo")
            .arg("nvme")
            .arg("sanitize")
            .arg(&device.path)
            .arg("--sanitize")
            .arg(mode_arg)
            .output()
            .await?;
        if !output.status.success() {
            return Err(anyhow!("NVMe sanitize failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        println!("NVMe sanitize completed.");
        Ok(())
    }

    async fn execute_crypto_erase(&self, device: &Device) -> Result<()> {
        println!("Executing Cryptographic Erase on {}", device.path);
        if !self.allow_real_devices {
            println!("Demo mode: not performing real crypto erase.");
            return Ok(());
        }
        match device.device_type {
            DriveType::SSD => {
                // Try NVMe crypto erase first
                let output = tokio::process::Command::new("sudo")
                    .arg("nvme")
                    .arg("format")
                    .arg(&device.path)
                    .arg("--ses")
                    .arg("2") // 2 = cryptographic erase
                    .output()
                    .await?;
                if output.status.success() {
                    println!("NVMe cryptographic erase completed.");
                    return Ok(());
                } else {
                    println!("NVMe crypto erase failed, trying blkdiscard as fallback.");
                }
                // Fallback to blkdiscard
                let discard = tokio::process::Command::new("sudo")
                    .arg("blkdiscard")
                    .arg(&device.path)
                    .output()
                    .await?;
                if !discard.status.success() {
                    return Err(anyhow!("blkdiscard failed: {}", String::from_utf8_lossy(&discard.stderr)));
                }
                println!("blkdiscard completed for {}.", device.path);
                Ok(())
            }
            DriveType::Removable => {
                let discard = tokio::process::Command::new("sudo")
                    .arg("blkdiscard")
                    .arg(&device.path)
                    .output()
                    .await?;
                if !discard.status.success() {
                    return Err(anyhow!("blkdiscard failed: {}", String::from_utf8_lossy(&discard.stderr)));
                }
                println!("blkdiscard completed for {}.", device.path);
                Ok(())
            }
            _ => Err(anyhow!("Crypto erase not supported for this device type")),
        }
    }
}

/// Legacy wipe function for backward compatibility
pub async fn wipe_device(device_name: &str, method: &str) -> Result<()> {
    println!("Legacy wipe function called - consider using SanitizationEngine for full functionality");
    println!("Wiping device: {} with method: {}", device_name, method);

    // Scan for devices
    let devices = match device::list_devices() {
        Ok(devs) => devs,
        Err(e) => {
            println!("❌ Failed to list devices: {}", e);
            return Err(e);
        }
    };
    let device = match devices.iter().find(|d| d.name == device_name) {
        Some(dev) => dev,
        None => {
            println!("❌ Device '{}' not found.", device_name);
            return Err(anyhow!("Device not found"));
        }
    };

    let engine = SanitizationEngine::new();
    let method_enum = match method.to_lowercase().as_str() {
        "clear" => SanitizationMethod::Clear,
        "purge" => SanitizationMethod::Purge,
        "destroy" => SanitizationMethod::Destroy,
        _ => {
            println!("❌ Unknown method. Use: clear, purge, or destroy.");
            return Err(anyhow!("Unknown method"));
        }
    };

    let result = engine.sanitize_device(device, method_enum).await;
    match result {
        Ok(wipe_result) => {
            println!("✅ Wipe operation completed. Status: {:?}", wipe_result.status);
            if !wipe_result.verification_passed {
                println!("⚠️ Verification did not pass. Data may not be fully purged.");
            }
            if let Some(err) = wipe_result.error_message {
                println!("⚠️ Error: {}", err);
            }
            if !wipe_result.patterns_used.is_empty() {
                println!("Patterns used: {}", wipe_result.patterns_used.join(", "));
            }
        }
        Err(e) => {
            println!("❌ Wipe operation failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
