//! HDD (Hard Disk Drive) specific erasure methods
//! 
//! Traditional magnetic storage drives require multiple overwrite passes
//! to ensure data cannot be recovered through magnetic force microscopy.

use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::fs::{File, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use crate::advanced_wiper::{DeviceInfo, DeviceType, WipingProgress, WipingAlgorithm};
use crate::devices::DeviceEraser;
use crate::ata_commands::AtaInterface;

pub struct HddEraser {
    buffer_size: usize,
    verify_after_wipe: bool,
}

impl HddEraser {
    pub fn new() -> Self {
        Self {
            buffer_size: 1024 * 1024, // 1MB buffer
            verify_after_wipe: true,
        }
    }
    
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self {
            buffer_size,
            verify_after_wipe: true,
        }
    }
    
    /// DoD 5220.22-M standard erasure (3-pass)
    pub fn dod_5220_22m_erase(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting DoD 5220.22-M (3-pass) erasure for HDD");
        
        let patterns = [
            vec![0x00; self.buffer_size], // Pass 1: All zeros
            vec![0xFF; self.buffer_size], // Pass 2: All ones
            self.generate_random_pattern(self.buffer_size), // Pass 3: Random
        ];
        
        for (pass, pattern) in patterns.iter().enumerate() {
            let pass_num = pass + 1;
            println!("🔄 HDD DoD Pass {}/3", pass_num);
            
            // Update progress
            if let Ok(mut progress) = progress_callback.lock() {
                progress.current_pass = pass_num as u32;
                progress.total_passes = 3;
                progress.current_pattern = match pass {
                    0 => "Zeros (0x00)".to_string(),
                    1 => "Ones (0xFF)".to_string(),
                    2 => "Random".to_string(),
                    _ => "Unknown".to_string(),
                };
            }
            
            self.overwrite_device(device_info, pattern, progress_callback.clone())?;
        }
        
        println!("✅ DoD 5220.22-M erasure completed for HDD");
        Ok(())
    }
    
    /// Gutmann 35-pass method for maximum security
    pub fn gutmann_erase(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting Gutmann 35-pass erasure for HDD");
        
        // Gutmann patterns for magnetic drives
        let gutmann_patterns = self.get_gutmann_patterns();
        
        for (pass, pattern_data) in gutmann_patterns.iter().enumerate() {
            let pass_num = pass + 1;
            println!("🔄 HDD Gutmann Pass {}/35: {}", pass_num, pattern_data.1);
            
            // Update progress
            if let Ok(mut progress) = progress_callback.lock() {
                progress.current_pass = pass_num as u32;
                progress.total_passes = 35;
                progress.current_pattern = pattern_data.1.clone();
            }
            
            let pattern = self.expand_pattern(&pattern_data.0, self.buffer_size);
            self.overwrite_device(device_info, &pattern, progress_callback.clone())?;
        }
        
        println!("✅ Gutmann 35-pass erasure completed for HDD");
        Ok(())
    }
    
    /// Multi-pass random erasure
    pub fn multi_pass_random_erase(
        &self,
        device_info: &DeviceInfo,
        passes: u32,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting {}-pass random erasure for HDD", passes);
        
        for pass in 1..=passes {
            println!("🔄 HDD Random Pass {}/{}", pass, passes);
            
            // Update progress
            if let Ok(mut progress) = progress_callback.lock() {
                progress.current_pass = pass;
                progress.total_passes = passes;
                progress.current_pattern = "Random".to_string();
            }
            
            let pattern = self.generate_random_pattern(self.buffer_size);
            self.overwrite_device(device_info, &pattern, progress_callback.clone())?;
        }
        
        println!("✅ {}-pass random erasure completed for HDD", passes);
        Ok(())
    }
    
    /// ATA Secure Erase for compatible HDDs
    pub fn ata_secure_erase(
        &self,
        device_info: &DeviceInfo,
        enhanced: bool,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting ATA Secure Erase for HDD (Enhanced: {})", enhanced);
        
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
                // Check if secure erase is supported
                let drive_info = ata.get_drive_info()?;
                if !drive_info.security_supported {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "ATA Secure Erase not supported on this drive"
                    ));
                }
                
                // Perform secure erase
                ata.security_erase(enhanced)?;
                println!("✅ ATA Secure Erase completed for HDD");
                Ok(())
            }
            Err(e) => {
                println!("❌ ATA interface failed, falling back to software erasure");
                // Fallback to software-based erasure
                self.dod_5220_22m_erase(device_info, progress_callback)
            }
        }
    }
    
    /// Overwrite device with specific pattern
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
        
        while bytes_written < total_size {
            let remaining = total_size - bytes_written;
            let write_size = std::cmp::min(pattern.len() as u64, remaining) as usize;
            
            file.write_all(&pattern[..write_size])?;
            bytes_written += write_size as u64;
            
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
    
    /// Get Gutmann patterns
    fn get_gutmann_patterns(&self) -> Vec<(Vec<u8>, String)> {
        vec![
            // Random passes
            (self.generate_random_pattern(3), "Random 1".to_string()),
            (self.generate_random_pattern(3), "Random 2".to_string()),
            (self.generate_random_pattern(3), "Random 3".to_string()),
            (self.generate_random_pattern(3), "Random 4".to_string()),
            // Specific patterns for magnetic drives
            (vec![0x55, 0x55, 0x55], "Pattern 0x555555".to_string()),
            (vec![0xAA, 0xAA, 0xAA], "Pattern 0xAAAAAA".to_string()),
            (vec![0x92, 0x49, 0x24], "Pattern 0x924924".to_string()),
            (vec![0x49, 0x24, 0x92], "Pattern 0x492492".to_string()),
            (vec![0x24, 0x92, 0x49], "Pattern 0x249249".to_string()),
            (vec![0x00, 0x00, 0x00], "Pattern 0x000000".to_string()),
            (vec![0x11, 0x11, 0x11], "Pattern 0x111111".to_string()),
            (vec![0x22, 0x22, 0x22], "Pattern 0x222222".to_string()),
            (vec![0x33, 0x33, 0x33], "Pattern 0x333333".to_string()),
            (vec![0x44, 0x44, 0x44], "Pattern 0x444444".to_string()),
            (vec![0x55, 0x55, 0x55], "Pattern 0x555555".to_string()),
            (vec![0x66, 0x66, 0x66], "Pattern 0x666666".to_string()),
            (vec![0x77, 0x77, 0x77], "Pattern 0x777777".to_string()),
            (vec![0x88, 0x88, 0x88], "Pattern 0x888888".to_string()),
            (vec![0x99, 0x99, 0x99], "Pattern 0x999999".to_string()),
            (vec![0xAA, 0xAA, 0xAA], "Pattern 0xAAAAAA".to_string()),
            (vec![0xBB, 0xBB, 0xBB], "Pattern 0xBBBBBB".to_string()),
            (vec![0xCC, 0xCC, 0xCC], "Pattern 0xCCCCCC".to_string()),
            (vec![0xDD, 0xDD, 0xDD], "Pattern 0xDDDDDD".to_string()),
            (vec![0xEE, 0xEE, 0xEE], "Pattern 0xEEEEEE".to_string()),
            (vec![0xFF, 0xFF, 0xFF], "Pattern 0xFFFFFF".to_string()),
            (vec![0x92, 0x49, 0x24], "Pattern 0x924924".to_string()),
            (vec![0x49, 0x24, 0x92], "Pattern 0x492492".to_string()),
            (vec![0x24, 0x92, 0x49], "Pattern 0x249249".to_string()),
            (vec![0x6D, 0xB6, 0xDB], "Pattern 0x6DB6DB".to_string()),
            (vec![0xB6, 0xDB, 0x6D], "Pattern 0xB6DB6D".to_string()),
            (vec![0xDB, 0x6D, 0xB6], "Pattern 0xDB6DB6".to_string()),
            // Final random passes
            (self.generate_random_pattern(3), "Random 32".to_string()),
            (self.generate_random_pattern(3), "Random 33".to_string()),
            (self.generate_random_pattern(3), "Random 34".to_string()),
            (self.generate_random_pattern(3), "Random 35".to_string()),
        ]
    }
    
    /// Expand pattern to buffer size
    fn expand_pattern(&self, pattern: &[u8], size: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(size);
        let pattern_len = pattern.len();
        for i in 0..size {
            result.push(pattern[i % pattern_len]);
        }
        result
    }
}

impl DeviceEraser for HddEraser {
    fn analyze_device(&self, device_path: &str) -> io::Result<DeviceInfo> {
        println!("🔍 Analyzing HDD device: {}", device_path);
        
        // Try to get detailed info via ATA interface
        let device_info = match AtaInterface::new(device_path) {
            Ok(ata) => {
                let drive_info = ata.get_drive_info()?;
                DeviceInfo {
                    device_path: device_path.to_string(),
                    device_type: DeviceType::HDD,
                    size_bytes: drive_info.user_capacity,
                    sector_size: 512, // Standard for HDDs
                    supports_trim: false, // HDDs don't support TRIM
                    supports_secure_erase: drive_info.security_supported,
                    supports_enhanced_secure_erase: drive_info.security_supported,
                    supports_crypto_erase: false, // HDDs typically don't have hardware encryption
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
                    device_type: DeviceType::HDD,
                    size_bytes: metadata.len(),
                    sector_size: 512,
                    supports_trim: false,
                    supports_secure_erase: false,
                    supports_enhanced_secure_erase: false,
                    supports_crypto_erase: false,
                    is_removable: false,
                    vendor: "Unknown".to_string(),
                    model: "Unknown HDD".to_string(),
                    serial: "Unknown".to_string(),
                }
            }
        };
        
        println!("✅ HDD analysis complete: {} ({} bytes)", 
                device_info.model, device_info.size_bytes);
        Ok(device_info)
    }
    
    fn erase_device(
        &self,
        device_info: &DeviceInfo,
        algorithm: WipingAlgorithm,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🚀 Starting HDD erasure with algorithm: {:?}", algorithm);
        
        match algorithm {
            WipingAlgorithm::DoD522022M => self.dod_5220_22m_erase(device_info, progress_callback),
            WipingAlgorithm::Gutmann => self.gutmann_erase(device_info, progress_callback),
            WipingAlgorithm::AtaSecureErase => self.ata_secure_erase(device_info, false, progress_callback),
            WipingAlgorithm::AtaEnhancedSecureErase => self.ata_secure_erase(device_info, true, progress_callback),
            WipingAlgorithm::ThreePass => self.multi_pass_random_erase(device_info, 3, progress_callback),
            WipingAlgorithm::SevenPass => self.multi_pass_random_erase(device_info, 7, progress_callback),
            _ => {
                // Default to DoD 5220.22-M for other algorithms
                println!("ℹ️  Using DoD 5220.22-M as default for HDD");
                self.dod_5220_22m_erase(device_info, progress_callback)
            }
        }
    }
    
    fn verify_erasure(&self, device_info: &DeviceInfo) -> io::Result<bool> {
        if !self.verify_after_wipe {
            return Ok(true);
        }
        
        println!("🔍 Verifying HDD erasure...");
        
        let mut file = File::open(&device_info.device_path)?;
        let mut buffer = vec![0u8; self.buffer_size];
        let mut total_read = 0u64;
        let sample_size = std::cmp::min(device_info.size_bytes, 100 * 1024 * 1024); // Sample first 100MB
        
        while total_read < sample_size {
            let bytes_read = std::io::Read::read(&mut file, &mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            // Check for non-zero bytes (indicating potential data remnants)
            if buffer[..bytes_read].iter().any(|&b| b != 0) {
                println!("⚠️  Found non-zero data during verification");
                return Ok(false);
            }
            
            total_read += bytes_read as u64;
        }
        
        println!("✅ HDD erasure verification passed");
        Ok(true)
    }
    
    fn get_recommended_algorithms(&self) -> Vec<WipingAlgorithm> {
        vec![
            WipingAlgorithm::DoD522022M,      // Standard 3-pass
            WipingAlgorithm::Gutmann,         // Maximum security 35-pass
            WipingAlgorithm::AtaSecureErase,  // Hardware-based if supported
            WipingAlgorithm::SevenPass,       // Enhanced multi-pass
            WipingAlgorithm::ThreePass,       // Basic multi-pass
        ]
    }
}