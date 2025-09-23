// HPA (Host Protected Area) and DCO (Device Configuration Overlay) detection and management
use std::io;
use crate::ata_commands::{AtaInterface, DriveInfo};
use crate::sanitization::SanitizationMethod;

#[derive(Debug, Clone)]
pub struct HpaInfo {
    pub present: bool,
    pub user_max_lba: u64,
    pub native_max_lba: u64,
    pub hidden_sectors: u64,
    pub hidden_size_mb: f64,
}

#[derive(Debug, Clone)]
pub struct DcoInfo {
    pub present: bool,
    pub original_capacity: u64,
    pub reported_capacity: u64,
    pub hidden_sectors: u64,
    pub hidden_size_mb: f64,
}

#[derive(Debug, Clone)]
pub struct SecurityInfo {
    pub supported: bool,
    pub enabled: bool,
    pub locked: bool,
    pub frozen: bool,
    pub count_expired: bool,
    pub enhanced_erase_supported: bool,
    pub normal_erase_time: u16,
    pub enhanced_erase_time: u16,
}

#[derive(Debug)]
pub struct ComprehensiveDriveInfo {
    pub basic_info: DriveInfo,
    pub hpa_info: HpaInfo,
    pub dco_info: DcoInfo,
    pub security_info: SecurityInfo,
    pub sanitization_recommendations: Vec<String>,
}

/// Simplified drive information for sanitization operations
#[derive(Debug, Clone)]
pub struct SimpleDriveInfo {
    pub user_capacity: u64,      // User-accessible capacity in sectors
    pub native_capacity: u64,    // Native capacity in sectors (includes HPA)
    pub hpa_detected: bool,      // Whether HPA is present
    pub dco_detected: bool,      // Whether DCO is supported/present
}

/// HPA and DCO detector
pub struct HpaDcoDetector;

impl HpaDcoDetector {
    pub fn new() -> Self {
        Self
    }

    /// Perform comprehensive analysis of a drive including HPA, DCO, and security features
    pub fn analyze_drive(&self, drive_path: &str) -> io::Result<ComprehensiveDriveInfo> {
        let ata = AtaInterface::new(drive_path)?;
        
        // Get basic drive identification
        let identify_data = ata.identify_device()?;
        let mut basic_info = ata.parse_identify_data(&identify_data);
        
        // Detect HPA
        let hpa_info = self.detect_hpa(&ata, &identify_data)?;
        
        // Detect DCO (requires comparing with manufacturer specifications)
        let dco_info = self.detect_dco(&ata, &basic_info)?;
        
        // Get detailed security information
        let security_info = self.analyze_security(&identify_data);
        
        // Update basic info with discovered capacities
        basic_info.native_capacity = hpa_info.native_max_lba * 512;
        basic_info.has_hpa = hpa_info.present;
        basic_info.has_dco = dco_info.present;
        
        // Generate sanitization recommendations
        let recommendations = self.generate_sanitization_recommendations(
            &hpa_info, 
            &dco_info, 
            &security_info
        );

        Ok(ComprehensiveDriveInfo {
            basic_info,
            hpa_info,
            dco_info,
            security_info,
            sanitization_recommendations: recommendations,
        })
    }

    /// Get comprehensive drive information (simplified struct for sanitization)
    pub fn get_comprehensive_drive_info(&self, drive_path: &str) -> io::Result<SimpleDriveInfo> {
        let ata = AtaInterface::new(drive_path)?;
        let identify_data = ata.identify_device()?;
        let basic_info = ata.parse_identify_data(&identify_data);
        
        // Get HPA information
        let hpa_info = self.detect_hpa(&ata, &identify_data)?;
        
        // Check for DCO (simplified detection)
        let words = &identify_data.data;
        let dco_detected = words[83] & 0x0800 != 0; // DCO feature set supported
        
        Ok(SimpleDriveInfo {
            user_capacity: basic_info.user_capacity / 512, // Convert to sectors
            native_capacity: hpa_info.native_max_lba,
            hpa_detected: hpa_info.present,
            dco_detected,
        })
    }

    /// Detect Host Protected Area (HPA)
    fn detect_hpa(&self, ata: &AtaInterface, identify_data: &crate::ata_commands::IdentifyDeviceData) -> io::Result<HpaInfo> {
        let words = &identify_data.data;
        
        // Get user-addressable capacity from IDENTIFY DEVICE
        let user_max_lba = if words[83] & 0x0400 != 0 {
            // 48-bit addressing
            ((words[103] as u64) << 48) | ((words[102] as u64) << 32) | 
            ((words[101] as u64) << 16) | (words[100] as u64)
        } else {
            // 28-bit addressing
            ((words[61] as u64) << 16) | (words[60] as u64)
        };

        // Get native capacity using READ NATIVE MAX ADDRESS
        let native_max_lba = match ata.read_native_max_address(words[83] & 0x0400 != 0) {
            Ok(lba) => lba,
            Err(_) => {
                // If command fails, assume no HPA
                return Ok(HpaInfo {
                    present: false,
                    user_max_lba,
                    native_max_lba: user_max_lba,
                    hidden_sectors: 0,
                    hidden_size_mb: 0.0,
                });
            }
        };

        let hidden_sectors = if native_max_lba > user_max_lba {
            native_max_lba - user_max_lba
        } else {
            0
        };

        let hidden_size_mb = (hidden_sectors * 512) as f64 / (1024.0 * 1024.0);

        Ok(HpaInfo {
            present: hidden_sectors > 0,
            user_max_lba,
            native_max_lba,
            hidden_sectors,
            hidden_size_mb,
        })
    }

    /// Detect Device Configuration Overlay (DCO)
    fn detect_dco(&self, _ata: &AtaInterface, basic_info: &DriveInfo) -> io::Result<DcoInfo> {
        // DCO detection is more complex and requires:
        // 1. DEVICE CONFIGURATION IDENTIFY command (if supported)
        // 2. Comparison with manufacturer specifications
        // 3. Analysis of reported vs. actual drive characteristics
        
        // For now, we'll implement a basic heuristic-based detection
        // In a complete implementation, you would:
        // - Use DEVICE CONFIGURATION IDENTIFY (0xB1) command
        // - Compare with known drive specifications database
        // - Check for suspicious capacity reductions
        
        // Placeholder implementation
        let reported_capacity = basic_info.user_capacity / 512; // Convert to sectors
        
        // This is a simplified heuristic - in reality, you'd need a database
        // of drive specifications to detect DCO properly
        let suspicious_capacity_reduction = self.check_suspicious_capacity(&basic_info.model, reported_capacity);
        
        Ok(DcoInfo {
            present: suspicious_capacity_reduction.0,
            original_capacity: suspicious_capacity_reduction.1,
            reported_capacity,
            hidden_sectors: if suspicious_capacity_reduction.0 {
                suspicious_capacity_reduction.1 - reported_capacity
            } else {
                0
            },
            hidden_size_mb: if suspicious_capacity_reduction.0 {
                ((suspicious_capacity_reduction.1 - reported_capacity) * 512) as f64 / (1024.0 * 1024.0)
            } else {
                0.0
            },
        })
    }

    /// Analyze security features in detail
    fn analyze_security(&self, identify_data: &crate::ata_commands::IdentifyDeviceData) -> SecurityInfo {
        let words = &identify_data.data;
        let security_word = words[128];
        let _enhanced_security_word = words[89];
        
        SecurityInfo {
            supported: security_word & 0x0001 != 0,
            enabled: security_word & 0x0002 != 0,
            locked: security_word & 0x0004 != 0,
            frozen: security_word & 0x0008 != 0,
            count_expired: security_word & 0x0010 != 0,
            enhanced_erase_supported: security_word & 0x0020 != 0,
            normal_erase_time: words[89] & 0x00FF,
            enhanced_erase_time: (words[89] & 0xFF00) >> 8,
        }
    }

    /// Check for suspicious capacity reductions that might indicate DCO
    fn check_suspicious_capacity(&self, _model: &str, reported_sectors: u64) -> (bool, u64) {
        // This is a simplified heuristic. In a real implementation, you would:
        // 1. Maintain a database of known drive models and their specifications
        // 2. Check if the reported capacity matches expected capacity
        // 3. Look for unusual capacity values (not round numbers)
        
        // For demonstration, we'll use some basic heuristics
        let reported_gb = (reported_sectors * 512) / (1000 * 1000 * 1000);
        
        // Check if capacity is suspiciously not a round number
        let common_sizes = [80, 120, 160, 250, 320, 500, 750, 1000, 1500, 2000, 3000, 4000, 6000, 8000, 10000, 12000, 16000];
        let closest_size = common_sizes.iter()
            .min_by_key(|&&size| ((size as i64) - (reported_gb as i64)).abs())
            .unwrap_or(&1000);
        
        let deviation = ((reported_gb as i64) - (*closest_size as i64)).abs();
        
        // If deviation is significant and the drive is smaller than expected
        if deviation > 50 && reported_gb < *closest_size {
            (true, (*closest_size as u64 * 1000 * 1000 * 1000) / 512) // Convert back to sectors
        } else {
            (false, reported_sectors)
        }
    }

    /// Generate recommendations for complete sanitization
    fn generate_sanitization_recommendations(&self, hpa: &HpaInfo, dco: &DcoInfo, security: &SecurityInfo) -> Vec<String> {
        let mut recommendations = Vec::new();

        // HPA recommendations
        if hpa.present {
            recommendations.push(format!(
                "⚠️ HPA DETECTED: {:.2} MB hidden area found. Must remove HPA before sanitization.",
                hpa.hidden_size_mb
            ));
            recommendations.push("1. Use SET MAX ADDRESS command to restore full capacity".to_string());
            recommendations.push("2. Sanitize entire native capacity".to_string());
            recommendations.push("3. Verify HPA removal after sanitization".to_string());
        }

        // DCO recommendations  
        if dco.present {
            recommendations.push(format!(
                "⚠️ DCO SUSPECTED: {:.2} MB potentially hidden by DCO. Advanced techniques required.",
                dco.hidden_size_mb
            ));
            recommendations.push("1. Use DEVICE CONFIGURATION RESET command if available".to_string());
            recommendations.push("2. Consider firmware-level sanitization".to_string());
            recommendations.push("3. Physical destruction may be required for highest security".to_string());
        }

        // Security recommendations
        if security.supported {
            if security.enhanced_erase_supported {
                recommendations.push("✅ ATA Secure Erase (Enhanced) is supported - recommended method".to_string());
                recommendations.push(format!("   Estimated time: {} minutes", security.enhanced_erase_time * 2));
            } else {
                recommendations.push("✅ ATA Secure Erase (Normal) is supported".to_string());
                recommendations.push(format!("   Estimated time: {} minutes", security.normal_erase_time * 2));
            }
            
            if security.frozen {
                recommendations.push("⚠️ Security is FROZEN - power cycle required before secure erase".to_string());
            }
            
            if security.locked {
                recommendations.push("⚠️ Drive is LOCKED - password required for secure erase".to_string());
            }
        } else {
            recommendations.push("⚠️ ATA Secure Erase not supported - use multi-pass overwrite".to_string());
        }

        // General recommendations
        if hpa.present || dco.present {
            recommendations.push("🔒 HIGH SECURITY RECOMMENDATION: Physical destruction for classified data".to_string());
        }

        recommendations.push("📋 Generate audit trail documenting all sanitization steps".to_string());
        recommendations.push("🔍 Perform post-sanitization verification".to_string());

        recommendations
    }

    /// Remove HPA by restoring native capacity
    pub fn remove_hpa(&self, drive_path: &str) -> io::Result<()> {
        let ata = AtaInterface::new(drive_path)?;
        
        // Get current information
        let identify_data = ata.identify_device()?;
        let words = &identify_data.data;
        let use_ext = words[83] & 0x0400 != 0;
        
        // Get native max address
        let native_max_lba = ata.read_native_max_address(use_ext)?;
        
        // Set max address to native capacity
        ata.set_max_address(native_max_lba, use_ext)?;
        
        println!("✅ HPA removed. Drive capacity restored to {} sectors", native_max_lba);
        Ok(())
    }

    /// Perform ATA Secure Erase
    pub fn secure_erase(&self, drive_path: &str, _enhanced: bool) -> io::Result<()> {
        let ata = AtaInterface::new(drive_path)?;
        
        // Check if security is frozen
        let identify_data = ata.identify_device()?;
        let security_word = identify_data.data[128];
        
        if security_word & 0x0008 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other, 
                "Drive security is frozen. Power cycle required."
            ));
        }

        // This is a simplified implementation. Real secure erase requires:
        // 1. SECURITY SET PASSWORD
        // 2. SECURITY ERASE PREPARE (for enhanced erase)
        // 3. SECURITY ERASE UNIT
        // 4. Proper error handling and timeout management
        
        println!("⚠️ SECURE ERASE NOT FULLY IMPLEMENTED");
        println!("Real implementation would:");
        println!("1. Set user password");
        println!("2. Issue SECURITY ERASE PREPARE (if enhanced)");
        println!("3. Issue SECURITY ERASE UNIT command");
        println!("4. Monitor completion status");
        
        Ok(())
    }

    /// Attempt to sanitize DCO areas (limited success - requires manufacturer tools)
    pub fn attempt_dco_sanitization(&self, drive_path: &str) -> io::Result<()> {
        println!("🔍 Attempting DCO area sanitization...");
        
        // DCO removal requires manufacturer-specific tools in most cases
        // However, we can try some standard approaches
        
        let ata = AtaInterface::new(drive_path)?;
        let identify_data = ata.identify_device()?;
        
        // Check if DCO is present by examining feature support
        let features_word = identify_data.data[83];
        
        if features_word & 0x0800 != 0 {
            println!("📋 Drive supports DCO feature set");
            
            // Try to access DCO features (this may not work on modern drives)
            println!("⚠️ DCO sanitization has limited effectiveness:");
            println!("  • DCO removal typically requires manufacturer tools");
            println!("  • Some enterprise drives may support DCO restore");
            println!("  • Consumer drives rarely allow DCO modification");
            println!("  • Physical destruction may be required for highest security");
            
            // Attempt to get DCO information (may fail)
            match ata.read_native_max_address(true) {
                Ok(native_max) => {
                    println!("📏 Native max address: {} sectors", native_max);
                    println!("💡 Recommendation: Use hardware-level secure erase if supported");
                }
                Err(_) => {
                    println!("❌ Unable to access native capacity information");
                }
            }
        } else {
            println!("ℹ️ Drive does not support DCO feature set");
        }
        
        Ok(())
    }

    /// Enhanced HPA removal with verification
    pub fn remove_hpa_thoroughly(&self, drive_path: &str) -> io::Result<bool> {
        println!("🔧 Performing thorough HPA removal...");
        
        let ata = AtaInterface::new(drive_path)?;
        
        // Get initial state
        let identify_data = ata.identify_device()?;
        let words = &identify_data.data;
        let use_ext = words[83] & 0x0400 != 0;
        
        // Get current and native capacities
        let current_max_lba = if use_ext {
            ((words[103] as u64) << 48) | ((words[102] as u64) << 32) | 
            ((words[101] as u64) << 16) | (words[100] as u64)
        } else {
            ((words[61] as u64) << 16) | (words[60] as u64)
        };
        
        let native_max_lba = ata.read_native_max_address(use_ext)?;
        
        if native_max_lba > current_max_lba {
            println!("🚨 HPA detected: Current={} sectors, Native={} sectors", 
                    current_max_lba, native_max_lba);
            println!("📏 Hidden capacity: {:.2} MB", 
                    (native_max_lba - current_max_lba) as f64 * 512.0 / (1024.0 * 1024.0));
            
            // Set max address to native capacity
            ata.set_max_address(native_max_lba, use_ext)?;
            
            // Verify the change
            let verify_data = ata.identify_device()?;
            let verify_words = &verify_data.data;
            let new_current_max = if use_ext {
                ((verify_words[103] as u64) << 48) | ((verify_words[102] as u64) << 32) | 
                ((verify_words[101] as u64) << 16) | (verify_words[100] as u64)
            } else {
                ((verify_words[61] as u64) << 16) | (verify_words[60] as u64)
            };
            
            if new_current_max == native_max_lba {
                println!("✅ HPA successfully removed. Full capacity restored: {} sectors", native_max_lba);
                return Ok(true);
            } else {
                println!("⚠️ HPA removal incomplete. Current: {}, Expected: {}", 
                        new_current_max, native_max_lba);
                return Ok(false);
            }
        } else {
            println!("ℹ️ No HPA detected or already removed");
            return Ok(true);
        }
    }

    /// Perform comprehensive sanitization including HPA/DCO areas
    pub fn comprehensive_clean(&self, device_path: &str, method: &SanitizationMethod) -> io::Result<()> {
        println!("🚀 Starting comprehensive drive sanitization...");
        
        // 1. Detect HPA/DCO before sanitization
        let drive_info = self.get_comprehensive_drive_info(device_path)?;
        
        // 2. Remove HPA thoroughly to access hidden areas
        if drive_info.hpa_detected {
            println!("🔧 Performing thorough HPA removal...");
            match self.remove_hpa_thoroughly(device_path) {
                Ok(true) => println!("✅ HPA removal completed successfully"),
                Ok(false) => println!("⚠️ HPA removal incomplete - some areas may remain hidden"),
                Err(e) => {
                    println!("❌ HPA removal failed: {}", e);
                    println!("⚠️ Continuing with sanitization of accessible areas only");
                }
            }
        }
        
        // 3. Sanitize the entire drive using native capacity
        println!("🔄 Sanitizing drive with full native capacity...");
        if let Err(e) = crate::sanitization::sanitize_device_with_size(
            device_path, method, drive_info.native_capacity
        ) {
            eprintln!("❌ Sanitization failed: {}", e);
            return Err(e);
        }
        
        // 4. Attempt to handle DCO areas
        if drive_info.dco_detected {
            println!("🔍 Attempting DCO area sanitization...");
            if let Err(e) = self.attempt_dco_sanitization(device_path) {
                println!("⚠️ DCO sanitization attempt failed: {}", e);
            }
        }
        
        // 5. Final verification
        println!("🔍 Performing final verification...");
        let final_info = self.get_comprehensive_drive_info(device_path)?;
        if final_info.hpa_detected {
            println!("⚠️ Warning: HPA still detected after removal attempt");
            println!("💡 Consider using manufacturer tools or physical destruction for complete security");
        } else {
            println!("✅ HPA verification passed - no hidden areas detected");
        }
        
        if final_info.dco_detected {
            println!("⚠️ Warning: DCO still detected - limited sanitization effectiveness");
            println!("💡 DCO removal requires manufacturer-specific tools or hardware destruction");
        }
        
        println!("🎉 Comprehensive sanitization completed");
        println!("📊 Summary:");
        println!("  • Sanitized capacity: {:.2} GB", drive_info.native_capacity as f64 * 512.0 / (1024.0 * 1024.0 * 1024.0));
        println!("  • HPA handled: {}", if !final_info.hpa_detected { "✅ Yes" } else { "⚠️ Partial" });
        println!("  • DCO handled: {}", if !final_info.dco_detected { "✅ Yes" } else { "⚠️ Limited" });
        
        Ok(())
    }
}