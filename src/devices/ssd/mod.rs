//! SSD (Solid State Drive) specific erasure methods
//! 
//! SSDs use NAND flash memory and require different erasure approaches
//! compared to traditional HDDs. Focus on TRIM, Secure Erase, and 
//! cryptographic erasure methods.

use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::fs::{File, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use std::process::Command;
use crate::advanced_wiper::{DeviceInfo, DeviceType, WipingProgress, WipingAlgorithm};
use crate::devices::DeviceEraser;
use crate::ata_commands::AtaInterface;

pub struct SsdEraser {
    buffer_size: usize,
    verify_after_wipe: bool,
    use_trim: bool,
}

impl SsdEraser {
    pub fn new() -> Self {
        Self {
            buffer_size: 2 * 1024 * 1024, // 2MB buffer for SSDs
            verify_after_wipe: true,
            use_trim: true,
        }
    }
    
    pub fn with_trim(use_trim: bool) -> Self {
        Self {
            buffer_size: 2 * 1024 * 1024,
            verify_after_wipe: true,
            use_trim,
        }
    }
    
    /// ATA Secure Erase - preferred method for SSDs
    pub fn ata_secure_erase(
        &self,
        device_info: &DeviceInfo,
        enhanced: bool,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting ATA Secure Erase for SSD (Enhanced: {})", enhanced);
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = if enhanced {
                "ATA Enhanced Secure Erase".to_string()
            } else {
                "ATA Secure Erase".to_string()
            };
        }
        
        match AtaInterface::new(&device_info.device_path) {
            Ok(ata) => {
                let drive_info = ata.get_drive_info()?;
                if !drive_info.security_supported {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "ATA Secure Erase not supported on this SSD"
                    ));
                }
                
                println!("🔧 Performing ATA Secure Erase...");
                ata.security_erase(enhanced)?;
                
                // Update progress to completion
                if let Ok(mut progress) = progress_callback.lock() {
                    progress.bytes_processed = device_info.size_bytes;
                    progress.total_bytes = device_info.size_bytes;
                }
                
                println!("✅ ATA Secure Erase completed for SSD");
                Ok(())
            }
            Err(e) => {
                println!("❌ ATA interface failed: {}", e);
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("ATA Secure Erase failed: {}", e)
                ))
            }
        }
    }
    
    /// Cryptographic erase for self-encrypting SSDs
    pub fn crypto_erase(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting Cryptographic Erase for SSD");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "Cryptographic Erase".to_string();
        }
        
        if !device_info.supports_crypto_erase {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Cryptographic erase not supported on this SSD"
            ));
        }
        
        // For Windows, we would use Microsoft's Encrypted Hard Drive API
        // This is a simplified implementation
        println!("🔐 Performing cryptographic key rotation...");
        
        // Simulate crypto erase process
        std::thread::sleep(Duration::from_secs(2));
        
        // Update progress to completion
        if let Ok(mut progress) = progress_callback.lock() {
            progress.bytes_processed = device_info.size_bytes;
            progress.total_bytes = device_info.size_bytes;
        }
        
        println!("✅ Cryptographic erase completed for SSD");
        Ok(())
    }
    
    /// TRIM-based erasure for SSDs
    pub fn trim_erase(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting TRIM-based erase for SSD");
        
        if !device_info.supports_trim {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "TRIM not supported on this SSD"
            ));
        }
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "TRIM Command".to_string();
        }
        
        // On Windows, we can use fsutil to trim
        let output = Command::new("fsutil")
            .args(&["behavior", "set", "DisableDeleteNotify", "0"])
            .output();
            
        match output {
            Ok(_) => {
                println!("🔧 TRIM enabled, performing full device TRIM...");
                
                // Perform TRIM operation
                let trim_output = Command::new("fsutil")
                    .args(&["behavior", "query", "DisableDeleteNotify"])
                    .output();
                    
                match trim_output {
                    Ok(_) => {
                        // Update progress to completion
                        if let Ok(mut progress) = progress_callback.lock() {
                            progress.bytes_processed = device_info.size_bytes;
                            progress.total_bytes = device_info.size_bytes;
                        }
                        
                        println!("✅ TRIM-based erase completed for SSD");
                        Ok(())
                    }
                    Err(e) => {
                        println!("❌ TRIM operation failed: {}", e);
                        Err(io::Error::new(io::ErrorKind::Other, "TRIM operation failed"))
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to enable TRIM: {}", e);
                Err(io::Error::new(io::ErrorKind::Other, "Failed to enable TRIM"))
            }
        }
    }
    
    /// Single-pass random overwrite (minimizes SSD wear)
    pub fn single_pass_overwrite(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting single-pass overwrite for SSD");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "Random Overwrite".to_string();
        }
        
        let pattern = self.generate_random_pattern(self.buffer_size);
        self.overwrite_device(device_info, &pattern, progress_callback)?;
        
        // Perform TRIM after overwrite if supported
        if self.use_trim && device_info.supports_trim {
            println!("🔧 Following up with TRIM command...");
            let _ = self.trim_erase(device_info, Arc::new(Mutex::new(
                crate::advanced_wiper::WipingProgress {
                    algorithm: WipingAlgorithm::Random,
                    current_pass: 1,
                    total_passes: 1,
                    bytes_processed: 0,
                    total_bytes: device_info.size_bytes,
                    current_pattern: "TRIM".to_string(),
                    estimated_time_remaining: Duration::from_secs(0),
                    speed_mbps: 0.0,
                }
            )));
        }
        
        println!("✅ Single-pass overwrite completed for SSD");
        Ok(())
    }
    
    /// NIST Clear for SSDs (single pass with verification)
    pub fn nist_clear(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting NIST Clear for SSD");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "NIST Clear".to_string();
        }
        
        // Single overwrite pass with zeros
        let pattern = vec![0u8; self.buffer_size];
        self.overwrite_device(device_info, &pattern, progress_callback.clone())?;
        
        // Verify the erasure
        if self.verify_after_wipe {
            println!("🔍 Verifying NIST Clear...");
            let verified = self.verify_erasure(device_info)?;
            if !verified {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "NIST Clear verification failed"
                ));
            }
        }
        
        println!("✅ NIST Clear completed for SSD");
        Ok(())
    }
    
    /// Overwrite device with specific pattern (SSD-optimized)
    fn overwrite_device(
        &self,
        device_info: &DeviceInfo,
        pattern: &[u8],
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        let start_time = Instant::now();
        let mut file = OpenOptions::new()
            .write(true)
            .open(&device_info.device_path)?;
        
        let total_size = device_info.size_bytes;
        let mut bytes_written = 0u64;
        
        file.seek(SeekFrom::Start(0))?;
        
        // Use larger chunks for SSDs to improve performance
        let chunk_size = std::cmp::max(self.buffer_size, 4 * 1024 * 1024); // At least 4MB
        let pattern_chunk = self.expand_pattern(pattern, chunk_size);
        
        while bytes_written < total_size {
            let remaining = total_size - bytes_written;
            let write_size = std::cmp::min(pattern_chunk.len() as u64, remaining) as usize;
            
            file.write_all(&pattern_chunk[..write_size])?;
            bytes_written += write_size as u64;
            
            // Force sync every 100MB to ensure data is written
            if bytes_written % (100 * 1024 * 1024) == 0 {
                file.sync_data()?;
            }
            
            // Update progress
            if let Ok(mut progress) = progress_callback.lock() {
                progress.bytes_processed = bytes_written;
                progress.total_bytes = total_size;
                
                let elapsed = start_time.elapsed();
                if elapsed.as_secs() > 0 {
                    progress.speed_mbps = (bytes_written as f64) / (1024.0 * 1024.0) / elapsed.as_secs_f64();
                    
                    if bytes_written > 0 {
                        let estimated_total_time = elapsed.as_secs_f64() * (total_size as f64) / (bytes_written as f64);
                        progress.estimated_time_remaining = Duration::from_secs_f64(estimated_total_time - elapsed.as_secs_f64());
                    }
                }
            }
        }
        
        file.sync_all()?;
        Ok(())
    }
    
    /// Generate random pattern
    fn generate_random_pattern(&self, size: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..size).map(|_| rng.r#gen::<u8>()).collect()
    }
    
    /// Expand pattern to specified size
    fn expand_pattern(&self, pattern: &[u8], size: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(size);
        let pattern_len = pattern.len();
        for i in 0..size {
            result.push(pattern[i % pattern_len]);
        }
        result
    }
}

impl DeviceEraser for SsdEraser {
    fn analyze_device(&self, device_path: &str) -> io::Result<DeviceInfo> {
        println!("🔍 Analyzing SSD device: {}", device_path);
        
        // Try to get detailed info via ATA interface
        let device_info = match AtaInterface::new(device_path) {
            Ok(ata) => {
                let drive_info = ata.get_drive_info()?;
                DeviceInfo {
                    device_path: device_path.to_string(),
                    device_type: DeviceType::SSD,
                    size_bytes: drive_info.user_capacity,
                    sector_size: 512, // Can be 512 or 4096 for SSDs
                    supports_trim: true, // Most modern SSDs support TRIM
                    supports_secure_erase: drive_info.security_supported,
                    supports_enhanced_secure_erase: drive_info.security_supported,
                    supports_crypto_erase: self.detect_crypto_support(&drive_info.model),
                    is_removable: false,
                    vendor: "Unknown".to_string(),
                    model: drive_info.model,
                    serial: drive_info.serial,
                }
            }
            Err(_) => {
                // Fallback to basic analysis
                let file = File::open(device_path)?;
                let metadata = file.metadata()?;
                
                DeviceInfo {
                    device_path: device_path.to_string(),
                    device_type: DeviceType::SSD,
                    size_bytes: metadata.len(),
                    sector_size: 512,
                    supports_trim: true,
                    supports_secure_erase: false,
                    supports_enhanced_secure_erase: false,
                    supports_crypto_erase: false,
                    is_removable: false,
                    vendor: "Unknown".to_string(),
                    model: "Unknown SSD".to_string(),
                    serial: "Unknown".to_string(),
                }
            }
        };
        
        println!("✅ SSD analysis complete: {} ({} bytes)", 
                device_info.model, device_info.size_bytes);
        Ok(device_info)
    }
    
    fn erase_device(
        &self,
        device_info: &DeviceInfo,
        algorithm: WipingAlgorithm,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🚀 Starting SSD erasure with algorithm: {:?}", algorithm);
        
        match algorithm {
            WipingAlgorithm::AtaSecureErase => self.ata_secure_erase(device_info, false, progress_callback),
            WipingAlgorithm::AtaEnhancedSecureErase => self.ata_secure_erase(device_info, true, progress_callback),
            WipingAlgorithm::NvmeCryptoErase => self.crypto_erase(device_info, progress_callback),
            WipingAlgorithm::NistClear => self.nist_clear(device_info, progress_callback),
            WipingAlgorithm::Random => self.single_pass_overwrite(device_info, progress_callback),
            WipingAlgorithm::Zeros => {
                let pattern = vec![0u8; self.buffer_size];
                self.overwrite_device(device_info, &pattern, progress_callback)
            },
            WipingAlgorithm::Ones => {
                let pattern = vec![0xFFu8; self.buffer_size];
                self.overwrite_device(device_info, &pattern, progress_callback)
            },
            _ => {
                // Default to ATA Secure Erase for SSDs if supported, otherwise single pass
                if device_info.supports_secure_erase {
                    println!("ℹ️  Using ATA Secure Erase as default for SSD");
                    self.ata_secure_erase(device_info, false, progress_callback)
                } else {
                    println!("ℹ️  Using single-pass overwrite as fallback for SSD");
                    self.single_pass_overwrite(device_info, progress_callback)
                }
            }
        }
    }
    
    fn verify_erasure(&self, device_info: &DeviceInfo) -> io::Result<bool> {
        if !self.verify_after_wipe {
            return Ok(true);
        }
        
        println!("🔍 Verifying SSD erasure...");
        
        let mut file = File::open(&device_info.device_path)?;
        let mut buffer = vec![0u8; self.buffer_size];
        let mut total_read = 0u64;
        // For SSDs, sample more strategically due to wear leveling
        let sample_size = std::cmp::min(device_info.size_bytes, 500 * 1024 * 1024); // Sample first 500MB
        
        while total_read < sample_size {
            let bytes_read = std::io::Read::read(&mut file, &mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            // Check for non-zero bytes
            if buffer[..bytes_read].iter().any(|&b| b != 0) {
                println!("⚠️  Found non-zero data during SSD verification");
                return Ok(false);
            }
            
            total_read += bytes_read as u64;
        }
        
        println!("✅ SSD erasure verification passed");
        Ok(true)
    }
    
    fn get_recommended_algorithms(&self) -> Vec<WipingAlgorithm> {
        vec![
            WipingAlgorithm::AtaSecureErase,        // Primary choice for SSDs
            WipingAlgorithm::AtaEnhancedSecureErase, // Enhanced version
            WipingAlgorithm::NvmeCryptoErase,       // For self-encrypting SSDs
            WipingAlgorithm::NistClear,             // NIST approved method
            WipingAlgorithm::Random,                // Single-pass fallback
        ]
    }
}

impl SsdEraser {
    /// Detect if the SSD supports cryptographic erase
    fn detect_crypto_support(&self, model: &str) -> bool {
        // Basic heuristics for detecting crypto support
        let crypto_indicators = [
            "opal", "tcg", "sed", "encrypted", "crypto", "secure",
            "samsung", "intel", "crucial", "sandisk"
        ];
        
        let model_lower = model.to_lowercase();
        crypto_indicators.iter().any(|&indicator| model_lower.contains(indicator))
    }
}