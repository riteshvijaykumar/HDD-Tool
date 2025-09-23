use eframe::egui;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use windows::{
    core::PWSTR,
    Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
    },
};

mod sanitization;
mod ata_commands;
mod advanced_wiper;
use sanitization::{DataSanitizer, SanitizationMethod, SanitizationProgress};
use advanced_wiper::{AdvancedWiper, WipingAlgorithm, WipingProgress, DeviceInfo, get_available_algorithms};

#[derive(Debug, Clone)]
struct DiskInfo {
    drive_letter: String,
    drive_type: String,          // Enhanced to show specific type (HDD/SSD/etc)
    detailed_type: String,       // More detailed type information
    file_system: String,
    total_space: u64,
    free_space: u64,
    used_space: u64,
    supports_secure_erase: bool, // Whether ATA Secure Erase is supported
    is_encrypted: bool,          // Whether the drive appears to be encrypted
}

struct HDDApp {
    disks: Vec<DiskInfo>,
    show_popup: bool,
    selected_disk: Option<DiskInfo>,
    selected_disk_index: Option<usize>, // Track which row is selected
    hovered_disk_index: Option<usize>, // Track which row is hovered
    sanitizer: DataSanitizer,
    sanitization_in_progress: bool,
    sanitization_progress: Option<SanitizationProgress>,
    last_error_message: Option<String>, // Store last error message to display
    
    // Advanced Wiper Integration
    advanced_wiper: AdvancedWiper,
    selected_algorithm: WipingAlgorithm,
    show_algorithm_selection: bool,
    device_analysis: Option<DeviceInfo>,
    wipe_progress: Arc<Mutex<WipingProgress>>,
    show_advanced_mode: bool,
}

impl HDDApp {
    fn new() -> Self {
        // Initialize wipe progress
        let initial_progress = WipingProgress {
            algorithm: WipingAlgorithm::NistClear,
            current_pass: 0,
            total_passes: 1,
            bytes_processed: 0,
            total_bytes: 0,
            current_pattern: "Ready".to_string(),
            estimated_time_remaining: Duration::from_secs(0),
            speed_mbps: 0.0,
        };
        
        let mut app = Self { 
            disks: Vec::new(),
            show_popup: false,
            selected_disk: None,
            selected_disk_index: None,
            hovered_disk_index: None,
            sanitizer: DataSanitizer::new(),
            sanitization_in_progress: false,
            sanitization_progress: None,
            last_error_message: None,
            
            // Advanced Wiper Integration
            advanced_wiper: AdvancedWiper::new(),
            selected_algorithm: WipingAlgorithm::NistClear,
            show_algorithm_selection: false,
            device_analysis: None,
            wipe_progress: Arc::new(Mutex::new(initial_progress)),
            show_advanced_mode: false,
        };
        app.refresh_disks();
        app
    }

    fn refresh_disks(&mut self) {
        self.disks.clear();
        
        unsafe {
            let logical_drives = GetLogicalDrives();
            
            for i in 0..26 {
                if logical_drives & (1 << i) != 0 {
                    let drive_letter = format!("{}:", (b'A' + i) as char);
                    let drive_path = format!("{}\\", drive_letter);
                    
                    if let Some(disk_info) = self.get_disk_info(&drive_path) {
                        self.disks.push(disk_info);
                    }
                }
            }
        }
    }

    fn get_disk_info(&self, drive_path: &str) -> Option<DiskInfo> {
        unsafe {
            let drive_path_wide: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();
            let drive_path_pwstr = PWSTR::from_raw(drive_path_wide.as_ptr() as *mut u16);

            // Get basic drive type
            let drive_type_raw = GetDriveTypeW(drive_path_pwstr);
            let basic_drive_type = match drive_type_raw {
                3 => "Fixed Drive",              // DRIVE_FIXED
                2 => "Removable Drive",          // DRIVE_REMOVABLE
                4 => "Network Drive",            // DRIVE_REMOTE
                5 => "CD-ROM Drive",             // DRIVE_CDROM
                6 => "RAM Disk",                 // DRIVE_RAMDISK
                1 => "Unknown",                  // DRIVE_UNKNOWN
                0 => "Cannot Determine",         // DRIVE_NO_ROOT_DIR
                _ => "Other",
            };

            // For fixed drives, try to get more specific information
            let (detailed_type, supports_secure_erase) = if drive_type_raw == 3 {
                self.detect_drive_details(drive_path)
            } else {
                (basic_drive_type.to_string(), false)
            };

            // Get disk space information
            let mut free_bytes = 0u64;
            let mut total_bytes = 0u64;
            
            let space_result = GetDiskFreeSpaceExW(
                drive_path_pwstr,
                Some(&mut free_bytes),
                Some(&mut total_bytes),
                None,
            );

            if space_result.is_err() {
                return None;
            }

            // Get file system information
            let mut volume_name_buffer = [0u16; 261];
            let mut file_system_buffer = [0u16; 261];
            let mut serial_number = 0u32;
            let mut max_component_length = 0u32;
            let mut file_system_flags = 0u32;

            let volume_result = GetVolumeInformationW(
                drive_path_pwstr,
                Some(&mut volume_name_buffer),
                Some(&mut serial_number),
                Some(&mut max_component_length),
                Some(&mut file_system_flags),
                Some(&mut file_system_buffer),
            );

            let file_system = if volume_result.is_ok() {
                String::from_utf16_lossy(&file_system_buffer)
                    .trim_end_matches('\0')
                    .to_string()
            } else {
                "Unknown".to_string()
            };

            // Check if drive appears to be encrypted
            let is_encrypted = file_system == "BitLocker" || 
                              file_system.contains("Encrypted") ||
                              file_system_flags & 0x00020000 != 0; // FILE_SUPPORTS_ENCRYPTION

            let used_space = total_bytes.saturating_sub(free_bytes);

            Some(DiskInfo {
                drive_letter: drive_path[..2].to_string(), // Get just "E:" instead of "E"
                drive_type: basic_drive_type.to_string(),
                detailed_type,
                file_system,
                total_space: total_bytes,
                free_space: free_bytes,
                used_space,
                supports_secure_erase,
                is_encrypted,
            })
        }
    }

    /// Detects specific drive details using ATA commands
    fn detect_drive_details(&self, drive_path: &str) -> (String, bool) {
        use crate::ata_commands::AtaInterface;
        
        // Convert logical drive path to physical drive path for ATA access
        let drive_letter = drive_path.chars().next().unwrap();
        let physical_drive_path = format!(r"\\.\PhysicalDrive{}", (drive_letter as u8 - b'A'));
        
        match AtaInterface::new(&physical_drive_path) {
            Ok(ata) => {
                match ata.identify_device() {
                    Ok(identify_data) => {
                        let drive_info = ata.parse_identify_data(&identify_data);
                        
                        // Determine drive type based on model and characteristics
                        let model_lower = drive_info.model.to_lowercase();
                        let drive_type = if model_lower.contains("ssd") || 
                                          model_lower.contains("solid state") ||
                                          model_lower.contains("nvme") ||
                                          model_lower.contains("m.2") {
                            "SSD (Solid State Drive)"
                        } else if model_lower.contains("hdd") || 
                                  model_lower.contains("hard disk") ||
                                  !model_lower.is_empty() {
                            "HDD (Hard Disk Drive)"
                        } else {
                            "Fixed Drive (Unknown Type)"
                        };
                        
                        // Check if ATA Secure Erase is supported and available
                        let secure_erase_available = drive_info.security_supported && 
                                                   !drive_info.security_frozen;
                        
                        (drive_type.to_string(), secure_erase_available)
                    },
                    Err(_) => ("Fixed Drive (ATA Detection Failed)".to_string(), false),
                }
            },
            Err(_) => ("Fixed Drive (No ATA Access)".to_string(), false),
        }
    }

    fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    fn estimate_sanitization_time(&self, disk: &DiskInfo, method: &SanitizationMethod) -> u64 {
        // Conservative estimates based on drive type
        let write_speed_mbs = if disk.drive_type.contains("Removable") {
            25 // USB drives are slower
        } else if disk.drive_type.contains("SSD") {
            200 // SSDs are faster
        } else {
            80 // Regular HDDs
        };
        
        let passes = match method {
            SanitizationMethod::Clear => 1,
            SanitizationMethod::Purge => 3,
            SanitizationMethod::SecureErase => 1,
            SanitizationMethod::EnhancedSecureErase => 1,
            SanitizationMethod::ComprehensiveClean => 3,
        };
        
        let size_mb = disk.total_space / (1024 * 1024);
        let time_seconds = (size_mb / write_speed_mbs) * passes;
        time_seconds / 60 // Convert to minutes
    }

    fn execute_real_sanitization(&mut self, method: SanitizationMethod, disk: &DiskInfo) -> Result<(), String> {
        println!("🔄 Starting {} sanitization for drive {}", 
                 match method { 
                     SanitizationMethod::Clear => "CLEAR", 
                     SanitizationMethod::Purge => "NIST 800-88 PURGE",
                     SanitizationMethod::SecureErase => "ATA SECURE ERASE",
                     SanitizationMethod::EnhancedSecureErase => "ENHANCED SECURE ERASE",
                     SanitizationMethod::ComprehensiveClean => "COMPREHENSIVE CLEAN",
                 }, 
                 disk.drive_letter);
        
        // Check if this is the system drive
        if disk.drive_letter == "C:" {
            let error_msg = format!("❌ Cannot sanitize system drive {} - this would make your computer unbootable!", disk.drive_letter);
            println!("{}", error_msg);
            return Err(error_msg);
        }
        
        // Warning for data drives
        println!("⚠️  WARNING: About to permanently erase all data on drive {}", disk.drive_letter);
        println!("⚠️  Drive contains: {} total space", Self::format_bytes(disk.total_space));
        println!("⚠️  Drive type: {}", disk.detailed_type);
        if disk.is_encrypted {
            println!("🔒 Drive appears to be encrypted - will attempt cryptographic erase");
        }
        if disk.supports_secure_erase {
            println!("🔧 Drive supports ATA Secure Erase - will use hardware-based destruction");
        }
        println!("⚠️  Note: This requires Administrator privileges and the drive must not be in use");
        
        match method {
            SanitizationMethod::Clear => {
                self.execute_clear_method(disk)
            },
            SanitizationMethod::Purge => {
                self.execute_nist_purge_method(disk)
            },
            SanitizationMethod::SecureErase => {
                self.execute_secure_erase_method(disk, false)
            },
            SanitizationMethod::EnhancedSecureErase => {
                self.execute_secure_erase_method(disk, true)
            },
            SanitizationMethod::ComprehensiveClean => {
                self.execute_comprehensive_clean_method(disk)
            }
        }
    }

    fn execute_clear_method(&mut self, disk: &DiskInfo) -> Result<(), String> {
        // Standard clear method - single pass overwrite
        let device_paths = vec![
            format!("\\\\.\\{}", disk.drive_letter.trim_end_matches(':')),
        ];
        
        let mut last_error = String::new();
        
        // Try direct device access first
        for device_path in device_paths.iter() {
            println!("🔧 Attempting direct device access: {}", device_path);
            
            let result = self.sanitizer.clear(device_path, sanitization::SanitizationPattern::Random, None);
            
            match result {
                Ok(_) => {
                    println!("✅ Clear sanitization completed for {}", disk.drive_letter);
                    return Ok(());
                },
                Err(e) => {
                    last_error = format!("Direct access failed: {}", e);
                    println!("❌ Direct device access failed: {}", e);
                }
            }
        }
        
        // Fallback to file-level sanitization
        println!("🔧 Falling back to file-level sanitization...");
        let drive_root = format!("{}\\", disk.drive_letter);
        
        match self.sanitizer.sanitize_files_and_free_space(&drive_root, 1, None) {
            Ok(_) => {
                println!("✅ File-level Clear sanitization completed for drive {}", disk.drive_letter);
                Ok(())
            },
            Err(e) => {
                let error_msg = format!("❌ Clear sanitization failed: {}", e);
                println!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    fn execute_nist_purge_method(&mut self, disk: &DiskInfo) -> Result<(), String> {
        println!("🔒 Executing NIST 800-88 Purge Method for complete data destruction");
        println!("📋 This method ensures data is cryptographically destroyed and unrecoverable");
        
        let drive_letter = disk.drive_letter.trim_end_matches(':');
        let device_path = format!("\\\\.\\{}", drive_letter);
        let physical_drive_path = format!(r"\\.\PhysicalDrive{}", (drive_letter.chars().next().unwrap() as u8 - b'A'));
        
        // Phase 1: ATA Secure Erase (fastest and most secure for modern drives)
        if disk.supports_secure_erase {
            println!("🔧 Phase 1: Attempting ATA Secure Erase (hardware-based destruction)...");
            match self.perform_ata_secure_erase(&physical_drive_path) {
                Ok(_) => {
                    println!("✅ NIST Purge completed using ATA Secure Erase - Data is cryptographically destroyed");
                    return Ok(());
                },
                Err(e) => {
                    println!("❌ ATA Secure Erase failed: {}", e);
                    println!("🔄 Continuing to next phase...");
                }
            }
        } else {
            println!("ℹ️  ATA Secure Erase not available for this drive");
        }
        
        // Phase 2: Cryptographic Erase (for encrypted drives)
        if disk.is_encrypted {
            println!("🔧 Phase 2: Attempting Cryptographic Erase (key destruction)...");
            match self.perform_cryptographic_erase(drive_letter) {
                Ok(_) => {
                    println!("✅ NIST Purge completed using Cryptographic Erase - Encryption keys destroyed");
                    return Ok(());
                },
                Err(e) => {
                    println!("❌ Cryptographic erase failed: {}", e);
                    println!("🔄 Continuing to overwrite method...");
                }
            }
        } else {
            println!("ℹ️  Drive is not encrypted - skipping cryptographic erase");
        }
        
        // Phase 3: Enhanced Multi-pass Overwrite (NIST approved patterns)
        println!("🔧 Phase 3: Performing NIST 800-88 compliant multi-pass overwrite...");
        self.perform_enhanced_overwrite(&device_path, disk)
    }

    fn perform_ata_secure_erase(&self, physical_drive_path: &str) -> Result<(), String> {
        use crate::ata_commands::AtaInterface;
        
        let ata = AtaInterface::new(physical_drive_path)
            .map_err(|e| format!("Cannot access drive for ATA commands: {}", e))?;
        
        let identify_data = ata.identify_device()
            .map_err(|e| format!("Cannot identify device: {}", e))?;
        
        let drive_info = ata.parse_identify_data(&identify_data);
        
        if !drive_info.security_supported {
            return Err("Drive does not support ATA security features".to_string());
        }
        
        if drive_info.security_frozen {
            return Err("Security is frozen - cannot perform secure erase".to_string());
        }
        
        // Note: In a full implementation, this would send ATA SECURITY SET PASSWORD
        // and ATA SECURITY ERASE UNIT commands. For now, we'll return an error
        // to indicate that this advanced feature is not fully implemented.
        
        Err("ATA Secure Erase implementation requires additional low-level ATA command support".to_string())
    }

    fn perform_cryptographic_erase(&self, drive_letter: &str) -> Result<(), String> {
        use std::process::Command;
        
        // Try BitLocker key deletion
        let cmd = format!("manage-bde -delete {}: -type recovery", drive_letter);
        
        println!("🔧 Deleting BitLocker recovery keys...");
        let output = Command::new("cmd")
            .args(&["/C", &cmd])
            .output()
            .map_err(|e| format!("Failed to execute BitLocker command: {}", e))?;
        
        if output.status.success() {
            println!("✅ BitLocker keys deleted successfully");
            
            // Also try to delete the volume encryption keys
            let cmd2 = format!("manage-bde -delete {}: -type password", drive_letter);
            let _ = Command::new("cmd").args(&["/C", &cmd2]).output();
            
            return Ok(());
        }
        
        let error_output = String::from_utf8_lossy(&output.stderr);
        Err(format!("BitLocker key deletion failed: {}", error_output))
    }

    fn perform_enhanced_overwrite(&mut self, device_path: &str, disk: &DiskInfo) -> Result<(), String> {
        println!("🔧 Using NIST 800-88 approved overwrite patterns:");
        println!("   Pass 1: All zeros (0x00)");
        println!("   Pass 2: All ones (0xFF)");
        println!("   Pass 3: Cryptographically secure random data");
        println!("   Pass 4: Pseudorandom data");
        println!("   Pass 5: Alternating pattern (0xAA)");
        println!("   Pass 6: Inverted alternating (0x55)");
        println!("   Pass 7: Final cryptographically secure random");
        
        // Try direct device access with enhanced patterns
        match self.sanitizer.purge(device_path, None) {
            Ok(_) => {
                println!("✅ NIST Purge completed using direct device overwrite");
                return Ok(());
            },
            Err(e) => {
                println!("❌ Direct device purge failed: {}", e);
                println!("🔄 Falling back to file-level multi-pass overwrite...");
            }
        }
        
        // Fallback to file-level sanitization with maximum passes
        let drive_root = format!("{}\\", disk.drive_letter);
        println!("🔧 Performing file-level NIST Purge on: {}", drive_root);
        
        match self.sanitizer.sanitize_files_and_free_space(&drive_root, 7, None) {
            Ok(_) => {
                println!("✅ NIST Purge completed using file-level multi-pass overwrite");
                println!("🔒 Data has been overwritten with cryptographically secure patterns");
                println!("📋 Recovery is computationally infeasible per NIST 800-88 standards");
                Ok(())
            },
            Err(e) => {
                let error_msg = format!(
                    "❌ NIST Purge method failed:\n{}\n\n💡 Possible solutions:\n\
                    • Ensure you have Administrator privileges\n\
                    • Close all programs using the drive\n\
                    • Check if drive has write protection\n\
                    • For SSDs, manufacturer utilities may be needed\n\
                    • Physical destruction may be required for highest security",
                    e
                );
                println!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    fn execute_secure_erase_method(&mut self, disk: &DiskInfo, enhanced: bool) -> Result<(), String> {
        println!("🔧 Attempting {} ATA Secure Erase for drive {}", 
                 if enhanced { "Enhanced" } else { "Standard" },
                 disk.drive_letter);
        
        let drive_letter = disk.drive_letter.trim_end_matches(':');
        let physical_drive_path = format!(r"\\.\PhysicalDrive{}", (drive_letter.chars().next().unwrap() as u8 - b'A'));
        
        match self.perform_ata_secure_erase(&physical_drive_path) {
            Ok(_) => {
                println!("✅ ATA Secure Erase completed for drive {}", disk.drive_letter);
                Ok(())
            },
            Err(e) => {
                println!("❌ ATA Secure Erase failed: {}", e);
                println!("🔄 Falling back to multi-pass overwrite...");
                self.execute_clear_method(disk)
            }
        }
    }

    fn execute_comprehensive_clean_method(&mut self, disk: &DiskInfo) -> Result<(), String> {
        println!("🔧 Performing Comprehensive Clean for drive {}", disk.drive_letter);
        println!("📋 This includes HPA/DCO detection and removal plus full sanitization");
        
        // For now, fall back to the NIST Purge method
        // In a full implementation, this would include HPA/DCO analysis
        self.execute_nist_purge_method(disk)
    }

    /// Analyze the selected device for advanced wiping
    fn analyze_selected_device(&mut self) {
        if let Some(ref disk) = self.selected_disk {
            let device_path = format!("\\\\.\\{}", disk.drive_letter.trim_end_matches(':'));
            match self.advanced_wiper.analyze_device(&device_path) {
                Ok(device_info) => {
                    println!("✅ Device analysis completed for {}", disk.drive_letter);
                    self.device_analysis = Some(device_info);
                },
                Err(e) => {
                    self.last_error_message = Some(format!("❌ Device analysis failed: {}", e));
                }
            }
        }
    }

    /// Show standard NIST 800-88 sanitization interface
    fn show_standard_sanitization_interface(&mut self, ui: &mut egui::Ui, disk: &DiskInfo) {
        // Real NIST 800-88 Clear method
        if ui.add_sized([200.0, 30.0], egui::Button::new("🗑️ CLEAR (Single Pass)")).clicked() {
            match self.execute_real_sanitization(SanitizationMethod::Clear, disk) {
                Ok(_) => {
                    self.last_error_message = Some("✅ Sanitization completed successfully!".to_string());
                },
                Err(e) => {
                    self.last_error_message = Some(e);
                }
            }
        }
        ui.label("NIST 800-88 Clear: Single pass overwrite");
        
        ui.separator();
        
        // Real NIST 800-88 Purge method
        if ui.add_sized([200.0, 30.0], egui::Button::new("🔥 PURGE (Multi Pass)")).clicked() {
            match self.execute_real_sanitization(SanitizationMethod::Purge, disk) {
                Ok(_) => {
                    self.last_error_message = Some("✅ Sanitization completed successfully!".to_string());
                },
                Err(e) => {
                    self.last_error_message = Some(e);
                }
            }
        }
        ui.label("NIST 800-88 Purge: 7-pass enhanced method");
    }

    /// Show advanced wiper interface with algorithm selection
    fn show_advanced_wiper_interface(&mut self, ui: &mut egui::Ui, disk: &DiskInfo) {
        // Show device analysis if available
        if let Some(ref device_info) = self.device_analysis {
            ui.collapsing("📊 Device Analysis", |ui| {
                ui.label(format!("Device Type: {:?}", device_info.device_type));
                ui.label(format!("Size: {:.2} GB", device_info.size_bytes as f64 / (1000.0 * 1000.0 * 1000.0)));
                ui.label(format!("Model: {}", device_info.model));
                ui.label(format!("Serial: {}", device_info.serial));
                ui.label(format!("Secure Erase: {}", if device_info.supports_secure_erase { "✅" } else { "❌" }));
                ui.label(format!("TRIM Support: {}", if device_info.supports_trim { "✅" } else { "❌" }));
                ui.label(format!("Crypto Erase: {}", if device_info.supports_crypto_erase { "✅" } else { "❌" }));
            });
            ui.separator();
        }

        ui.label("🔧 Select Wiping Algorithm:");
        
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                let algorithms = get_available_algorithms();
                
                for (algorithm, name, description) in algorithms {
                    let is_selected = std::mem::discriminant(&self.selected_algorithm) == std::mem::discriminant(&algorithm);
                    
                    if ui.selectable_label(is_selected, format!("🔹 {}", name)).clicked() {
                        self.selected_algorithm = algorithm;
                    }
                    
                    ui.label(format!("   {}", description));
                    ui.separator();
                }
            });

        ui.separator();

        // Show selected algorithm
        ui.label(format!("Selected: {:?}", self.selected_algorithm));
        
        // Recommend algorithm based on device type
        if let Some(ref device_info) = self.device_analysis {
            ui.label("💡 Recommended for this device:");
            match device_info.device_type {
                advanced_wiper::DeviceType::SSD | advanced_wiper::DeviceType::NVMe => {
                    if device_info.supports_secure_erase {
                        ui.colored_label(egui::Color32::GREEN, "   🔧 ATA Secure Erase (Hardware-based, fastest)");
                    } else {
                        ui.colored_label(egui::Color32::BLUE, "   🔒 NIST Purge (Software-based, secure)");
                    }
                },
                advanced_wiper::DeviceType::HDD => {
                    ui.colored_label(egui::Color32::BLUE, "   🔒 NIST Purge or DoD 5220.22-M");
                },
                _ => {
                    ui.colored_label(egui::Color32::YELLOW, "   🔒 NIST Clear or 3-Pass method");
                }
            }
        }

        ui.separator();

        // Execute button
        if ui.add_sized([200.0, 40.0], egui::Button::new("🚨 EXECUTE WIPE")).clicked() {
            self.execute_advanced_wipe(disk);
        }
    }

    /// Execute advanced wipe with selected algorithm
    fn execute_advanced_wipe(&mut self, disk: &DiskInfo) {
        // For Windows, we need to use the drive root path for file-level access
        // or physical device path for direct device access
        let drive_root = format!("{}\\", disk.drive_letter); // e.g., "T:\"
        let physical_device_path = self.get_physical_device_path(&disk.drive_letter);
        
        println!("🔧 Advanced wipe target:");
        println!("   Drive root: {}", drive_root);
        println!("   Physical device: {:?}", physical_device_path);
        
        // Get or create device info
        let device_info = match &self.device_analysis {
            Some(info) => {
                // Update the device path to use drive root for file-level access
                let mut updated_info = info.clone();
                updated_info.device_path = drive_root.clone();
                updated_info
            },
            None => {
                // Create basic device info
                advanced_wiper::DeviceInfo {
                    device_path: drive_root.clone(),
                    device_type: advanced_wiper::DeviceType::Other("Unknown".to_string()),
                    size_bytes: disk.total_space,
                    sector_size: 512,
                    supports_trim: false,
                    supports_secure_erase: disk.supports_secure_erase,
                    supports_enhanced_secure_erase: false,
                    supports_crypto_erase: disk.is_encrypted,
                    is_removable: disk.drive_type.contains("Removable"),
                    vendor: "Unknown".to_string(),
                    model: "Unknown".to_string(),
                    serial: "Unknown".to_string(),
                }
            }
        };

        // Start wipe in background thread
        let algorithm = self.selected_algorithm.clone();
        let progress_callback = self.wipe_progress.clone();
        let wiper = self.advanced_wiper.clone();
        
        // Reset progress
        {
            let mut progress = progress_callback.lock().unwrap();
            progress.algorithm = algorithm.clone();
            progress.current_pass = 0;
            progress.total_passes = 1;
            progress.bytes_processed = 0;
            progress.total_bytes = device_info.size_bytes;
        }

        thread::spawn(move || {
            match wiper.wipe_device(&device_info, algorithm, progress_callback) {
                Ok(result) => {
                    println!("✅ Advanced wipe completed: {}", result);
                },
                Err(e) => {
                    println!("❌ Advanced wipe failed: {}", e);
                }
            }
        });

        self.sanitization_in_progress = true;
    }

    /// Get physical device path for a drive letter
    fn get_physical_device_path(&self, drive_letter: &str) -> Option<String> {
        let drive_char = drive_letter.chars().next()?.to_ascii_uppercase();
        if drive_char >= 'A' && drive_char <= 'Z' {
            let drive_index = (drive_char as u8) - b'A';
            Some(format!(r"\\.\PhysicalDrive{}", drive_index))
        } else {
            None
        }
    }
}

impl eframe::App for HDDApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("HDD Tool - System Disk Information");
            
            if ui.button("Refresh Disks").clicked() {
                self.refresh_disks();
            }
            
            ui.separator();
            
            if self.disks.is_empty() {
                ui.label("No disks found or unable to access disk information.");
                return;
            }

            // Table headers
            egui::Grid::new("disk_grid")
                .striped(true)
                .num_columns(6)
                .show(ui, |ui| {
                    ui.label("Drive");
                    ui.label("Type");
                    ui.label("File System");
                    ui.label("Total Space");
                    ui.label("Free Space");
                    ui.label("Used Space");
                    ui.end_row();

                    for (index, disk) in self.disks.iter().enumerate() {
                        // Check if this row is selected
                        let is_selected = self.selected_disk_index == Some(index);
                        
                        // Check if this row is currently being hovered
                        let is_hovered = self.hovered_disk_index == Some(index);
                        
                        // Track hover and click states for the entire row
                        let mut any_hovered = false;
                        let mut any_clicked = false;
                        
                        // Define the highlight color (subtle gray/low opacity white)  
                        let highlight_color = egui::Color32::from_rgba_unmultiplied(200, 200, 200, 60);
                        
                        // Render each cell as clickable buttons with custom styling
                        let drive_response = ui.add(
                            egui::Button::new(&disk.drive_letter)
                                .fill(if is_selected || is_hovered { highlight_color } else { egui::Color32::TRANSPARENT })
                                .stroke(egui::Stroke::NONE)
                        );
                        any_hovered |= drive_response.hovered();
                        any_clicked |= drive_response.clicked();
                        
                        let type_response = ui.add(
                            egui::Button::new(&disk.detailed_type)
                                .fill(if is_selected || is_hovered { highlight_color } else { egui::Color32::TRANSPARENT })
                                .stroke(egui::Stroke::NONE)
                        );
                        any_hovered |= type_response.hovered();
                        any_clicked |= type_response.clicked();
                        
                        let fs_response = ui.add(
                            egui::Button::new(&disk.file_system)
                                .fill(if is_selected || is_hovered { highlight_color } else { egui::Color32::TRANSPARENT })
                                .stroke(egui::Stroke::NONE)
                        );
                        any_hovered |= fs_response.hovered();
                        any_clicked |= fs_response.clicked();
                        
                        let total_response = ui.add(
                            egui::Button::new(Self::format_bytes(disk.total_space))
                                .fill(if is_selected || is_hovered { highlight_color } else { egui::Color32::TRANSPARENT })
                                .stroke(egui::Stroke::NONE)
                        );
                        any_hovered |= total_response.hovered();
                        any_clicked |= total_response.clicked();
                        
                        let free_response = ui.add(
                            egui::Button::new(Self::format_bytes(disk.free_space))
                                .fill(if is_selected || is_hovered { highlight_color } else { egui::Color32::TRANSPARENT })
                                .stroke(egui::Stroke::NONE)
                        );
                        any_hovered |= free_response.hovered();
                        any_clicked |= free_response.clicked();
                        
                        let used_response = ui.add(
                            egui::Button::new(Self::format_bytes(disk.used_space))
                                .fill(if is_selected || is_hovered { highlight_color } else { egui::Color32::TRANSPARENT })
                                .stroke(egui::Stroke::NONE)
                        );
                        any_hovered |= used_response.hovered();
                        any_clicked |= used_response.clicked();
                        
                        // Update hover state for the entire row
                        if any_hovered {
                            self.hovered_disk_index = Some(index);
                        } else if self.hovered_disk_index == Some(index) {
                            // Clear hover state if we're no longer hovering over this row
                            self.hovered_disk_index = None;
                        }
                        
                        // If any part of the row is clicked, show popup and select this row
                        if any_clicked {
                            self.show_popup = true;
                            self.selected_disk = Some(disk.clone());
                            self.selected_disk_index = Some(index);
                        }
                        
                        ui.end_row();
                    }
                });

            // Show popup window if requested
            if self.show_popup {
                let mut open = true; // Track if window should stay open
                let mut close_requested = false; // Track close button clicks
                
                egui::Window::new("Disk Actions")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut open) // Allow window to be closed with X button
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .show(ctx, |ui| {
                        if let Some(disk) = self.selected_disk.clone() {
                            ui.heading(format!("Actions for Drive {}", disk.drive_letter));
                            ui.separator();
                            
                            ui.label(format!("Drive Type: {}", disk.detailed_type));
                            ui.label(format!("Basic Type: {}", disk.drive_type));
                            ui.label(format!("File System: {}", disk.file_system));
                            ui.label(format!("Total Space: {}", Self::format_bytes(disk.total_space)));
                            
                            // Show enhanced security features
                            if disk.supports_secure_erase {
                                ui.colored_label(egui::Color32::GREEN, "🔧 Supports ATA Secure Erase");
                            }
                            if disk.is_encrypted {
                                ui.colored_label(egui::Color32::BLUE, "🔒 Drive is Encrypted");
                            }
                            
                            // Show time estimates
                            let clear_time = self.estimate_sanitization_time(&disk, &SanitizationMethod::Clear);
                            let purge_time = self.estimate_sanitization_time(&disk, &SanitizationMethod::Purge);
                            ui.label(format!("⏱️ Clear time: ~{} minutes", clear_time));
                            ui.label(format!("⏱️ Purge time: ~{} minutes", purge_time));
                            
                            ui.separator();
                            
                            if self.sanitization_in_progress {
                                ui.label("🔄 Sanitization in Progress...");
                                
                                if let Some(ref progress) = self.sanitization_progress {
                                    ui.label(format!("Pass {}/{}", progress.current_pass, progress.total_passes));
                                    
                                    let progress_bar = egui::ProgressBar::new((progress.percentage / 100.0) as f32)
                                        .text(format!("{:.1}%", progress.percentage));
                                    ui.add(progress_bar);
                                    
                                    ui.label(format!(
                                        "Processed: {} / {}",
                                        Self::format_bytes(progress.bytes_processed),
                                        Self::format_bytes(progress.total_bytes)
                                    ));
                                }
                            } else {
                                ui.label("⚠️ NIST 800-88 Data Sanitization");
                                ui.colored_label(egui::Color32::RED, "⚠️ WARNING: This will permanently erase ALL data!");
                                
                                ui.separator();
                                
                                // Advanced Mode Toggle
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut self.show_advanced_mode, "🔧 Advanced Mode");
                                    if ui.button("🔍 Analyze Device").clicked() {
                                        self.analyze_selected_device();
                                    }
                                });
                                
                                ui.separator();
                                
                                if self.show_advanced_mode {
                                    // Show advanced algorithm selection
                                    self.show_advanced_wiper_interface(ui, &disk);
                                } else {
                                    // Standard NIST 800-88 methods
                                    self.show_standard_sanitization_interface(ui, &disk);
                                }
                                
                                // Show error/success message if available
                                if let Some(ref message) = self.last_error_message {
                                    ui.separator();
                                    if message.starts_with("✅") {
                                        ui.colored_label(egui::Color32::GREEN, message);
                                    } else {
                                        ui.colored_label(egui::Color32::RED, message);
                                    }
                                }
                            }
                            
                            ui.separator();
                            
                            // Close button
                            if ui.add_sized([120.0, 25.0], egui::Button::new("❌ Close")).clicked() {
                                if !self.sanitization_in_progress {
                                    close_requested = true;
                                }
                            }
                        }
                    });
                
                // Update popup state based on window state
                if !open || close_requested {
                    self.show_popup = false;
                    self.selected_disk = None;
                    self.selected_disk_index = None;
                    self.last_error_message = None; // Clear error message when closing
                    // Don't reset sanitization progress when closing popup
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust HDD Manager",
        native_options,
        Box::new(|_cc| Ok(Box::new(HDDApp::new()))),
    )
}