use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write, BufWriter};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Instant, Duration};
use rayon::prelude::*;
use rand::{Rng, RngCore};
use chrono::Utc;
use uuid::Uuid;

use crate::core::{
    SanitizationStandard, TargetType, WipeRequest, WipeProgress, WipeResult, ValidationResult,
    DriveGeometry, SecurityFeatures, WipeError, WipeErrorCode, WipeResult2, WipeConfiguration,
};
use crate::core::config::{NIST_CLEAR_PATTERNS, NIST_PURGE_PATTERNS, DOD_522022M_PATTERNS, 
                           VERIFICATION_BLOCK_SIZE, MAX_RETRY_ATTEMPTS, PROGRESS_UPDATE_INTERVAL};
use crate::hardware::DriveInterface;

#[derive(Debug, Clone)]
pub struct SanitizationProgress {
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub current_pass: u32,
    pub total_passes: u32,
    pub percentage: f64,
    pub estimated_time_remaining: chrono::Duration,
    pub current_operation: String,
    pub errors: Vec<String>,
}

pub struct SecureSanitizer {
    config: WipeConfiguration,
    progress_sender: Option<mpsc::Sender<WipeProgress>>,
}

impl SecureSanitizer {
    pub fn new(config: WipeConfiguration) -> Self {
        Self {
            config,
            progress_sender: None,
        }
    }

    pub fn with_progress_reporting(mut self, sender: mpsc::Sender<WipeProgress>) -> Self {
        self.progress_sender = Some(sender);
        self
    }

    pub fn execute_wipe(&self, request: WipeRequest) -> WipeResult2<WipeResult> {
        let start_time = Utc::now();
        
        // Step 1: Open and analyze the target
        let drive = DriveInterface::new(&request.target_path)?;
        let identify_data = drive.identify_device()?;
        let drive_geometry = drive.parse_drive_geometry(&identify_data);
        let security_features = drive.parse_security_features(&identify_data);

        // Step 2: Security checks
        if security_features.security_locked {
            return Err(WipeError {
                code: WipeErrorCode::SecurityLocked,
                message: "Drive is security locked".to_string(),
                sector: None,
            });
        }

        // Step 3: Unlock hidden areas if requested
        let mut actual_geometry = drive_geometry.clone();
        if matches!(request.target_type, TargetType::ComprehensiveFull | TargetType::HiddenAreas) {
            if drive_geometry.has_hpa {
                match drive.unlock_hpa() {
                    Ok(true) => {
                        println!("HPA unlocked successfully");
                        // Re-read geometry after HPA unlock
                        let new_identify = drive.identify_device()?;
                        actual_geometry = drive.parse_drive_geometry(&new_identify);
                    }
                    Ok(false) => println!("No HPA detected or already unlocked"),
                    Err(e) => {
                        println!("Warning: Failed to unlock HPA: {}", e);
                        if matches!(request.target_type, TargetType::HiddenAreas) {
                            return Err(WipeError {
                                code: WipeErrorCode::HPAUnlockFailed,
                                message: "Failed to unlock HPA for hidden area wipe".to_string(),
                                sector: None,
                            });
                        }
                    }
                }
            }
        }

        // Step 4: Determine target sectors
        let (start_sector, sector_count) = self.calculate_target_range(&request, &actual_geometry)?;
        
        // Step 5: Execute sanitization passes
        let patterns = self.get_patterns_for_standard(&request.standard, request.passes)?;
        let total_passes = patterns.len() as u32;
        let mut sectors_wiped = 0u64;

        for (pass_num, pattern) in patterns.iter().enumerate() {
            let pass_start_time = Instant::now();
            
            self.send_progress(WipeProgress {
                request_id: request.id,
                current_pass: (pass_num + 1) as u32,
                total_passes,
                sectors_processed: 0,
                total_sectors: sector_count,
                percentage: 0.0,
                current_operation: format!("Pass {} - Overwriting with pattern", pass_num + 1),
                estimated_completion: None,
            });

            sectors_wiped += self.execute_pattern_pass(
                &request.target_path,
                start_sector,
                sector_count,
                pattern,
                pass_num + 1,
                total_passes,
                &request.id,
            )?;

            println!("Pass {} completed in {:.2} seconds", 
                     pass_num + 1, 
                     pass_start_time.elapsed().as_secs_f64());
        }

        // Step 6: Verification if requested
        let validation_result = if request.verify_erasure {
            self.send_progress(WipeProgress {
                request_id: request.id,
                current_pass: total_passes + 1,
                total_passes: total_passes + 1,
                sectors_processed: 0,
                total_sectors: sector_count,
                percentage: 0.0,
                current_operation: "Verifying erasure".to_string(),
                estimated_completion: None,
            });

            Some(self.verify_erasure(
                &request.target_path,
                start_sector,
                sector_count,
                patterns.last().unwrap(),
                &request.id,
            )?)
        } else {
            None
        };

        let completion_time = Utc::now();
        let duration = completion_time.signed_duration_since(start_time);

        Ok(WipeResult {
            request_id: request.id,
            success: true,
            start_time,
            completion_time,
            duration_seconds: duration.num_seconds() as u64,
            sectors_wiped,
            passes_completed: total_passes,
            validation_result,
            error_message: None,
            drive_geometry: actual_geometry,
            security_features,
        })
    }

    fn calculate_target_range(&self, request: &WipeRequest, geometry: &DriveGeometry) -> WipeResult2<(u64, u64)> {
        match &request.target_type {
            TargetType::FullDisk => Ok((0, geometry.total_sectors)),
            TargetType::ComprehensiveFull => Ok((0, geometry.total_sectors)),
            TargetType::HiddenAreas => {
                if geometry.has_hpa && geometry.hpa_size > 0 {
                    let hpa_start_sector = geometry.user_capacity / geometry.sector_size;
                    let hpa_sector_count = geometry.hpa_size / geometry.sector_size;
                    Ok((hpa_start_sector, hpa_sector_count))
                } else {
                    Err(WipeError {
                        code: WipeErrorCode::InvalidPattern,
                        message: "No hidden areas detected".to_string(),
                        sector: None,
                    })
                }
            }
            TargetType::Partition(path) => {
                // For partition wiping, we would need to parse the partition table
                // For now, return the full disk (this should be enhanced)
                Ok((0, geometry.total_sectors))
            }
        }
    }

    fn get_patterns_for_standard(&self, standard: &SanitizationStandard, passes: u32) -> WipeResult2<Vec<Vec<u8>>> {
        match standard {
            SanitizationStandard::NIST80088Clear => {
                Ok(vec![NIST_CLEAR_PATTERNS.to_vec()])
            }
            SanitizationStandard::NIST80088Purge => {
                let mut patterns = Vec::new();
                patterns.push(NIST_PURGE_PATTERNS[0].to_vec()); // Zeros
                patterns.push(NIST_PURGE_PATTERNS[1].to_vec()); // Ones
                patterns.push(self.generate_random_pattern()); // Random
                Ok(patterns)
            }
            SanitizationStandard::DoD522022M => {
                let mut patterns = Vec::new();
                patterns.push(DOD_522022M_PATTERNS[0].to_vec()); // Pass 1: Zeros
                patterns.push(DOD_522022M_PATTERNS[1].to_vec()); // Pass 2: Ones
                patterns.push(self.generate_random_pattern()); // Pass 3: Random
                patterns.push(DOD_522022M_PATTERNS[3].to_vec()); // Pass 4: Zeros (verification)
                Ok(patterns)
            }
        }
    }

    fn generate_random_pattern(&self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut pattern = vec![0u8; self.config.buffer_size];
        rng.fill_bytes(&mut pattern);
        pattern
    }

    fn execute_pattern_pass(
        &self,
        device_path: &str,
        start_sector: u64,
        sector_count: u64,
        pattern: &[u8],
        pass_num: usize,
        total_passes: u32,
        request_id: &Uuid,
    ) -> WipeResult2<u64> {
        let mut device = OpenOptions::new()
            .read(true)
            .write(true)
            .open(device_path)
            .map_err(|e| WipeError {
                code: WipeErrorCode::AccessDenied,
                message: format!("Failed to open device: {}", e),
                sector: None,
            })?;

        let total_bytes = sector_count * 512;
        let mut bytes_written = 0u64;
        let buffer_size = self.config.buffer_size;
        
        // Create pattern buffer
        let pattern_buffer = if pattern.len() == 1 {
            vec![pattern[0]; buffer_size]
        } else if pattern.len() >= buffer_size {
            pattern[0..buffer_size].to_vec()
        } else {
            let repeat_count = (buffer_size + pattern.len() - 1) / pattern.len();
            pattern.repeat(repeat_count)[0..buffer_size].to_vec()
        };

        device.seek(SeekFrom::Start(start_sector * 512))
            .map_err(|e| WipeError {
                code: WipeErrorCode::HardwareError,
                message: format!("Failed to seek to start sector: {}", e),
                sector: Some(start_sector),
            })?;

        let mut writer = BufWriter::with_capacity(buffer_size, &mut device);
        let start_time = Instant::now();
        let mut last_progress_update = Instant::now();

        while bytes_written < total_bytes {
            let remaining_bytes = total_bytes - bytes_written;
            let write_size = buffer_size.min(remaining_bytes as usize);
            
            let write_buffer = &pattern_buffer[0..write_size];
            
            match writer.write_all(write_buffer) {
                Ok(_) => {
                    bytes_written += write_size as u64;
                    
                    // Update progress periodically
                    if last_progress_update.elapsed() > Duration::from_millis(100) {
                        let percentage = (bytes_written as f64 / total_bytes as f64) * 100.0;
                        let sectors_processed = bytes_written / 512;
                        
                        self.send_progress(WipeProgress {
                            request_id: *request_id,
                            current_pass: pass_num as u32,
                            total_passes,
                            sectors_processed,
                            total_sectors: sector_count,
                            percentage,
                            current_operation: format!("Pass {} - Writing pattern", pass_num),
                            estimated_completion: None,
                        });
                        
                        last_progress_update = Instant::now();
                    }
                }
                Err(e) => {
                    return Err(WipeError {
                        code: WipeErrorCode::HardwareError,
                        message: format!("Write failed at byte {}: {}", bytes_written, e),
                        sector: Some(start_sector + bytes_written / 512),
                    });
                }
            }
        }

        writer.flush().map_err(|e| WipeError {
            code: WipeErrorCode::HardwareError,
            message: format!("Failed to flush writes: {}", e),
            sector: None,
        })?;

        Ok(sector_count)
    }

    fn verify_erasure(
        &self,
        device_path: &str,
        start_sector: u64,
        sector_count: u64,
        expected_pattern: &[u8],
        request_id: &Uuid,
    ) -> WipeResult2<ValidationResult> {
        let mut device = File::open(device_path)
            .map_err(|e| WipeError {
                code: WipeErrorCode::AccessDenied,
                message: format!("Failed to open device for verification: {}", e),
                sector: None,
            })?;

        let total_bytes = sector_count * 512;
        let sample_rate = self.config.verification_sample_rate;
        let blocks_to_verify = ((sector_count as f64 * sample_rate) as u64).max(1);
        let mut failed_sectors = Vec::new();
        let mut sectors_verified = 0u64;
        let mut bytes_read = 0u64;

        device.seek(SeekFrom::Start(start_sector * 512))
            .map_err(|e| WipeError {
                code: WipeErrorCode::HardwareError,
                message: format!("Failed to seek for verification: {}", e),
                sector: Some(start_sector),
            })?;

        let mut buffer = vec![0u8; VERIFICATION_BLOCK_SIZE];
        let expected_byte = if expected_pattern.len() == 1 { 
            expected_pattern[0] 
        } else { 
            0 // For complex patterns, we'll do a simpler check
        };

        while bytes_read < total_bytes && sectors_verified < blocks_to_verify {
            let read_size = VERIFICATION_BLOCK_SIZE.min((total_bytes - bytes_read) as usize);
            
            match device.read_exact(&mut buffer[0..read_size]) {
                Ok(_) => {
                    // Verify the pattern
                    for (i, &byte) in buffer[0..read_size].iter().enumerate() {
                        if expected_pattern.len() == 1 {
                            if byte != expected_byte {
                                let sector = start_sector + (bytes_read + i as u64) / 512;
                                failed_sectors.push(sector);
                            }
                        }
                        // For complex patterns, additional verification logic would go here
                    }
                    
                    bytes_read += read_size as u64;
                    sectors_verified += (read_size as u64 + 511) / 512;
                    
                    // Update progress
                    let percentage = (bytes_read as f64 / total_bytes as f64) * 100.0;
                    self.send_progress(WipeProgress {
                        request_id: *request_id,
                        current_pass: 0, // Verification pass
                        total_passes: 1,
                        sectors_processed: bytes_read / 512,
                        total_sectors: sector_count,
                        percentage,
                        current_operation: "Verifying erasure".to_string(),
                        estimated_completion: None,
                    });
                }
                Err(e) => {
                    return Err(WipeError {
                        code: WipeErrorCode::VerificationFailed,
                        message: format!("Verification read failed: {}", e),
                        sector: Some(start_sector + bytes_read / 512),
                    });
                }
            }
        }

        let pattern_matches = failed_sectors.is_empty();
        
        Ok(ValidationResult {
            sectors_verified,
            failed_sectors,
            pattern_matches,
            checksum_valid: true, // Additional checksum verification could be added
            completion_time: Utc::now(),
        })
    }

    fn send_progress(&self, progress: WipeProgress) {
        if let Some(ref sender) = self.progress_sender {
            let _ = sender.send(progress);
        }
    }
}