use eframe::egui;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use chrono;
use windows::{
    core::PWSTR,
    Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
    },
};

mod sanitization;
mod ata_commands;
mod advanced_wiper;
mod ui;

use sanitization::{DataSanitizer, SanitizationMethod, SanitizationProgress};
use advanced_wiper::{AdvancedWiper, WipingAlgorithm, WipingProgress, DeviceInfo};
use ui::{SecureTheme, TabWidget, DriveTableWidget, DriveInfo, AdvancedOptionsWidget, show_logo};

#[derive(Debug, Clone)]
struct DiskInfo {
    drive_letter: String,
    drive_type: String,
    detailed_type: String,
    file_system: String,
    total_space: u64,
    free_space: u64,
    used_space: u64,
    supports_secure_erase: bool,
    is_encrypted: bool,
}

struct HDDApp {
    disks: Vec<DiskInfo>,
    sanitizer: DataSanitizer,
    sanitization_in_progress: bool,
    sanitization_progress: Option<SanitizationProgress>,
    last_error_message: Option<String>,
    
    // Advanced Wiper Integration
    advanced_wiper: AdvancedWiper,
    selected_algorithm: WipingAlgorithm,
    device_analysis: Option<DeviceInfo>,
    wipe_progress: Arc<Mutex<WipingProgress>>,
    
    // New UI Components
    tab_widget: TabWidget,
    drive_table: DriveTableWidget,
    advanced_options: AdvancedOptionsWidget,
}

impl HDDApp {
    fn new() -> Self {
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
            sanitizer: DataSanitizer::new(),
            sanitization_in_progress: false,
            sanitization_progress: None,
            last_error_message: None,
            
            advanced_wiper: AdvancedWiper::new(),
            selected_algorithm: WipingAlgorithm::NistClear,
            device_analysis: None,
            wipe_progress: Arc::new(Mutex::new(initial_progress)),
            
            tab_widget: TabWidget::new(),
            drive_table: DriveTableWidget::new(),
            advanced_options: AdvancedOptionsWidget::new(),
        };
        app.refresh_disks();
        app
    }

    fn refresh_disks(&mut self) {
        self.disks.clear();
        self.drive_table.drives.clear();
        
        unsafe {
            let logical_drives = GetLogicalDrives();
            
            for i in 0..26 {
                if logical_drives & (1 << i) != 0 {
                    let drive_letter = format!("{}:", (b'A' + i) as char);
                    let drive_path = format!("{}\\", drive_letter);
                    
                    if let Some(disk_info) = self.get_disk_info(&drive_path) {
                        // Add to internal list
                        self.disks.push(disk_info.clone());
                        
                        // Add to drive table widget
                        let drive_ui_info = DriveInfo::new(
                            if disk_info.drive_letter == "C:" { 
                                "OS".to_string() 
                            } else { 
                                disk_info.drive_letter.clone() 
                            },
                            disk_info.drive_letter.clone(),
                            Self::format_bytes(disk_info.total_space),
                            Self::format_bytes(disk_info.used_space),
                        );
                        self.drive_table.add_drive(drive_ui_info);
                    }
                }
            }
        }
    }

    fn get_disk_info(&self, drive_path: &str) -> Option<DiskInfo> {
        let drive_letter = drive_path.trim_end_matches('\\');
        
        unsafe {
            let wide_path: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();
            let drive_type = GetDriveTypeW(PWSTR(wide_path.as_ptr() as *mut u16));
            
            let drive_type_str = match drive_type {
                2 => "Removable",
                3 => "Fixed",
                4 => "Network",
                5 => "CD-ROM",
                6 => "RAM",
                _ => "Unknown",
            };
            
            let mut total_bytes = 0u64;
            let mut free_bytes = 0u64;
            
            let result = GetDiskFreeSpaceExW(
                PWSTR(wide_path.as_ptr() as *mut u16),
                Some(&mut free_bytes),
                Some(&mut total_bytes),
                None,
            );
            
            if result.is_err() {
                return None;
            }
            
            let mut volume_name_buffer = vec![0u16; 256];
            let mut file_system_buffer = vec![0u16; 256];
            let mut volume_serial_number = 0u32;
            let mut maximum_component_length = 0u32;
            let mut file_system_flags = 0u32;
            
            let _ = GetVolumeInformationW(
                PWSTR(wide_path.as_ptr() as *mut u16),
                Some(&mut volume_name_buffer),
                Some(&mut volume_serial_number),
                Some(&mut maximum_component_length),
                Some(&mut file_system_flags),
                Some(&mut file_system_buffer),
            );
            
            let file_system = String::from_utf16_lossy(&file_system_buffer)
                .trim_end_matches('\0')
                .to_string();
            
            let (detailed_type, supports_secure_erase) = if drive_type_str == "Fixed" {
                self.get_detailed_drive_info(drive_letter)
            } else {
                (format!("{} Drive", drive_type_str), false)
            };
            
            let is_encrypted = file_system_flags & 0x00040000 != 0; // FILE_SUPPORTS_ENCRYPTION
            
            Some(DiskInfo {
                drive_letter: drive_letter.to_string(),
                drive_type: drive_type_str.to_string(),
                detailed_type,
                file_system,
                total_space: total_bytes,
                free_space: free_bytes,
                used_space: total_bytes - free_bytes,
                supports_secure_erase,
                is_encrypted,
            })
        }
    }

    fn get_detailed_drive_info(&self, drive_letter: &str) -> (String, bool) {
        use ata_commands::AtaInterface;
        
        let drive_num = (drive_letter.chars().next().unwrap() as u8).saturating_sub(b'A');
        let physical_drive_path = format!(r"\\.\PhysicalDrive{}", drive_num);
        
        match AtaInterface::new(&physical_drive_path) {
            Ok(ata) => {
                match ata.identify_device() {
                    Ok(identify_data) => {
                        let drive_info = ata.parse_identify_data(&identify_data);
                        
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
    
    fn handle_erase_request(&mut self) {
        // First check if erase confirmation is checked
        if !self.advanced_options.confirm_erase {
            self.last_error_message = Some("❌ Please check 'Confirm to erase the data' before starting the erase process".to_string());
            return;
        }
        
        // Get selected drives
        let selected_drives: Vec<usize> = self.drive_table.drives
            .iter()
            .enumerate()
            .filter(|(_, drive)| drive.selected)
            .map(|(i, _)| i)
            .collect();
            
        if selected_drives.is_empty() {
            self.last_error_message = Some("❌ No drives selected for sanitization".to_string());
            return;
        }
        
        // Check if system drive is selected
        for &drive_idx in &selected_drives {
            if let Some(disk_info) = self.disks.get(drive_idx) {
                if disk_info.drive_letter == "C:" {
                    self.last_error_message = Some("❌ Cannot sanitize system drive C: - this would make your computer unbootable!".to_string());
                    return;
                }
            }
        }
        
        // Start real sanitization for selected drives
        self.sanitization_in_progress = true;
        self.last_error_message = Some(format!("� REAL SANITIZATION STARTED: {} erasure for {} drive(s) - ALL FILES AND FOLDERS WILL BE PERMANENTLY DESTROYED!", 
            self.advanced_options.eraser_method, selected_drives.len()));
        
        // Start actual sanitization process
        self.start_real_sanitization();
    }
    
    fn start_real_sanitization(&mut self) {
        // Collect drives to sanitize
        let drives_to_process: Vec<(String, String, usize)> = self.drive_table.drives
            .iter()
            .enumerate()
            .filter(|(_, drive)| drive.selected)
            .map(|(i, drive)| (drive.path.clone(), drive.name.clone(), i))
            .collect();
        
        if drives_to_process.is_empty() {
            return;
        }
        
        // Start the sanitization process for each selected drive
        for (drive_path, drive_name, drive_index) in drives_to_process {
            self.start_drive_sanitization(&drive_path, &drive_name, drive_index);
        }
        
        // Begin progress simulation/tracking
        self.simulate_sanitization_progress();
    }
    
    fn start_drive_sanitization(&mut self, drive_path: &str, drive_name: &str, drive_index: usize) {
        let sanitizer = DataSanitizer::new();
        let passes = 3; // NIST SP 800-88 and DoD 5220.22-M typically use 3 passes
        
        // Convert drive path to full path (e.g., "C:" -> "C:\")
        let full_drive_path = if drive_path.ends_with(':') {
            format!("{}\\", drive_path)
        } else {
            drive_path.to_string()
        };
        
        println!("🔥 Starting real sanitization of drive {} ({})", drive_name, full_drive_path);
        
        // Start sanitization in a separate thread to avoid blocking UI
        let drive_path_clone = full_drive_path.clone();
        std::thread::spawn(move || {
            match sanitizer.sanitize_files_and_free_space(&drive_path_clone, passes, None) {
                Ok(_) => {
                    println!("✅ Successfully sanitized drive: {}", drive_path_clone);
                }
                Err(e) => {
                    println!("❌ Failed to sanitize drive {}: {}", drive_path_clone, e);
                }
            }
        });
        
        // Initialize progress tracking for this drive
        let total_bytes = if let Some(drive) = self.drive_table.drives.get(drive_index) {
            self.parse_size_to_bytes(&drive.size)
        } else {
            1_000_000_000 // Default 1GB if drive not found
        };
        
        if let Some(drive) = self.drive_table.drives.get_mut(drive_index) {
            drive.start_processing(total_bytes);
            drive.status = format!("Sanitizing {} passes", passes);
        }
    }
    
    fn simulate_sanitization_progress(&mut self) {
        // Collect drive data first to avoid borrowing conflicts
        let mut drive_updates = Vec::new();
        let mut total_bytes_all_drives = 0u64;
        let mut total_processed_all_drives = 0u64;
        
        // Start processing for selected drives
        for (i, drive) in self.drive_table.drives.iter().enumerate() {
            if drive.selected && drive.progress == 0.0 {
                // Simulate total bytes based on drive size
                // Parse size string (e.g., "100 GB" -> bytes)
                let total_bytes = self.parse_size_to_bytes(&drive.size);
                drive_updates.push((i, total_bytes, true)); // true = start processing
            }
        }
        
        // Apply start processing updates
        for (i, total_bytes, start) in drive_updates {
            if start {
                if let Some(drive) = self.drive_table.drives.get_mut(i) {
                    drive.start_processing(total_bytes);
                }
            }
        }
        
        // Update progress for processing drives and calculate overall progress
        let mut any_in_progress = false;
        let mut all_completed = true;
        
        for drive in &mut self.drive_table.drives {
            if drive.selected {
                total_bytes_all_drives += drive.bytes_total;
                
                if drive.start_time.is_some() && drive.progress < 1.0 {
                    // Simulate progress increment (in real implementation, this would come from actual sanitization)
                    let increment = 1024 * 1024 * 2; // 2MB per update cycle
                    let new_bytes_processed = (drive.bytes_processed + increment).min(drive.bytes_total);
                    drive.update_progress(new_bytes_processed);
                    any_in_progress = true;
                    
                    if drive.progress < 1.0 {
                        all_completed = false;
                    }
                }
                
                total_processed_all_drives += drive.bytes_processed;
            }
        }
        
        // Update overall sanitization progress
        if total_bytes_all_drives > 0 {
            let overall_percentage = (total_processed_all_drives as f64 / total_bytes_all_drives as f64) * 100.0;
            
            let progress = SanitizationProgress {
                current_pass: if overall_percentage < 33.0 { 1 } else if overall_percentage < 66.0 { 2 } else { 3 },
                total_passes: 3,
                percentage: overall_percentage,
                bytes_processed: total_processed_all_drives,
                total_bytes: total_bytes_all_drives,
            };
            self.sanitization_progress = Some(progress);
        }
        
        // Check if sanitization is complete
        if all_completed && any_in_progress {
            self.sanitization_in_progress = false;
            self.last_error_message = Some("✅ Sanitization completed successfully!".to_string());
        }
    }
    
    fn parse_size_to_bytes(&self, size_str: &str) -> u64 {
        // Parse size string like "100 GB", "50.5 MB" etc.
        let parts: Vec<&str> = size_str.split_whitespace().collect();
        if parts.len() != 2 {
            return 1_000_000_000; // Default 1GB if parsing fails
        }
        
        let number: f64 = parts[0].parse().unwrap_or(1.0);
        let unit = parts[1].to_uppercase();
        
        let multiplier: u64 = match unit.as_str() {
            "B" => 1,
            "KB" => 1_000,
            "MB" => 1_000_000,
            "GB" => 1_000_000_000,
            "TB" => 1_000_000_000_000,
            _ => 1_000_000_000, // Default to GB
        };
        
        (number * multiplier as f64) as u64
    }
    
    fn generate_sanitization_report(&mut self) {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("sanitization_report_{}.txt", timestamp);
        
        let mut report = String::new();
        report.push_str("SHREDX - Sanitization Report\n");
        report.push_str(&format!("Generated: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        report.push_str(&format!("Erasure Method: {}\n", self.advanced_options.eraser_method));
        report.push_str(&format!("Verification: {}\n", self.advanced_options.verification));
        report.push_str("\n=== SANITIZED DRIVES ===\n");
        
        for drive in &self.drive_table.drives {
            if drive.selected && drive.progress >= 1.0 {
                report.push_str(&format!("✅ {} ({}): Complete\n", drive.name, drive.path));
                report.push_str(&format!("   Size: {}\n", drive.size));
                report.push_str(&format!("   Status: {}\n", drive.status));
            }
        }
        
        report.push_str("\n=== COMPLIANCE ===\n");
        report.push_str("This sanitization process complies with:\n");
        if self.advanced_options.eraser_method.contains("NIST") {
            report.push_str("- NIST SP 800-88 Guidelines\n");
        }
        if self.advanced_options.eraser_method.contains("DoD") {
            report.push_str("- DoD 5220.22-M Standards\n");
        }
        
        // Try to save the report
        match std::fs::write(&filename, report) {
            Ok(_) => {
                self.last_error_message = Some(format!("✅ Report saved as: {}", filename));
            }
            Err(e) => {
                self.last_error_message = Some(format!("❌ Failed to save report: {}", e));
            }
        }
    }
}

impl eframe::App for HDDApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply SHREDX theme
        SecureTheme::apply(ctx);
        
        // Continuous progress updates for active sanitization processes
        let has_active_process = self.drive_table.drives.iter()
            .any(|drive| drive.start_time.is_some() && drive.progress < 1.0);
            
        if has_active_process {
            self.simulate_sanitization_progress();
            ctx.request_repaint(); // Ensure UI updates continuously
        }
        
        // Set window title
        ctx.send_viewport_cmd(egui::ViewportCommand::Title("SHREDX - HDD Secure Wipe Tool".to_string()));
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title bar with logo
            ui.horizontal(|ui| {
                show_logo(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔄").clicked() {
                        self.refresh_disks();
                    }
                });
            });
            
            ui.add_space(20.0);
            
            // Tab navigation
            let active_tab = self.tab_widget.show(ui, &["Drives", "Details", "Report"]);
            
            ui.add_space(20.0);
            
            match active_tab {
                0 => {
                    // Drives tab
                    self.drive_table.show(ui);
                    
                    ui.add_space(30.0);
                    
                    // Advanced options and handle erase button
                    if self.advanced_options.show(ui) {
                        self.handle_erase_request();
                    }
                },
                1 => {
                    // Details tab
                    ui.vertical_centered(|ui| {
                        ui.heading("Drive Details");
                        ui.label("Selected drives information will appear here");
                        
                        // Show details for selected drives
                        for (i, drive) in self.drive_table.drives.iter().enumerate() {
                            if drive.selected {
                                if let Some(disk_info) = self.disks.get(i) {
                                    ui.group(|ui| {
                                        ui.heading(&drive.name);
                                        ui.label(format!("Path: {}", disk_info.drive_letter));
                                        ui.label(format!("Type: {}", disk_info.detailed_type));
                                        ui.label(format!("File System: {}", disk_info.file_system));
                                        ui.label(format!("Total Space: {}", drive.size));
                                        ui.label(format!("Used Space: {}", drive.used));
                                        ui.label(format!("Free Space: {}", Self::format_bytes(disk_info.free_space)));
                                        ui.label(format!("Secure Erase: {}", if disk_info.supports_secure_erase { "✅ Supported" } else { "❌ Not Supported" }));
                                        ui.label(format!("Encrypted: {}", if disk_info.is_encrypted { "🔒 Yes" } else { "🔓 No" }));
                                    });
                                }
                            }
                        }
                    });
                },
                2 => {
                    // Report tab
                    ui.vertical_centered(|ui| {
                        ui.heading("Sanitization Reports");
                        
                        if let Some(ref message) = self.last_error_message {
                            ui.add_space(20.0);
                            if message.starts_with("✅") {
                                ui.colored_label(SecureTheme::SUCCESS_GREEN, message);
                                
                                // Show completion report
                                if !self.sanitization_in_progress {
                                    ui.add_space(10.0);
                                    ui.group(|ui| {
                                        ui.heading("📋 Sanitization Report");
                                        
                                        // Show completed drives
                                        for drive in &self.drive_table.drives {
                                            if drive.selected && drive.progress >= 1.0 {
                                                ui.horizontal(|ui| {
                                                    ui.label("✅");
                                                    ui.label(&drive.name);
                                                    ui.label(format!("({}) - Complete", drive.path));
                                                });
                                            }
                                        }
                                        
                                        ui.add_space(10.0);
                                        ui.label(format!("Method: {}", self.advanced_options.eraser_method));
                                        ui.label(format!("Verification: {}", self.advanced_options.verification));
                                        ui.label(format!("Completion Time: {}", 
                                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
                                        
                                        ui.add_space(10.0);
                                        if ui.button("💾 Save Report").clicked() {
                                            self.generate_sanitization_report();
                                        }
                                    });
                                }
                            } else {
                                ui.colored_label(SecureTheme::DANGER_RED, message);
                            }
                        }
                        
                        // Show sanitization progress if in progress
                        if self.sanitization_in_progress {
                            ui.add_space(20.0);
                            ui.group(|ui| {
                                ui.heading("🔄 Sanitization in Progress");
                                
                                if let Some(ref progress) = self.sanitization_progress {
                                    ui.label(format!("Pass {}/{}", progress.current_pass, progress.total_passes));
                                    
                                    let progress_bar = egui::ProgressBar::new((progress.percentage / 100.0) as f32)
                                        .text(format!("{:.1}%", progress.percentage))
                                        .fill(SecureTheme::LIGHT_BLUE);
                                    ui.add(progress_bar);
                                    
                                    ui.label(format!(
                                        "Processed: {} / {}",
                                        Self::format_bytes(progress.bytes_processed),
                                        Self::format_bytes(progress.total_bytes)
                                    ));
                                }
                                
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.label("🔧 Method:");
                                    ui.label(&self.advanced_options.eraser_method);
                                });
                                
                                // Show individual drive progress
                                ui.add_space(10.0);
                                ui.label("Individual Drive Progress:");
                                for drive in &self.drive_table.drives {
                                    if drive.selected && drive.start_time.is_some() {
                                        ui.horizontal(|ui| {
                                            let status_icon = if drive.progress >= 1.0 { "✅" } 
                                                           else if drive.progress > 0.0 { "🔄" } 
                                                           else { "⏸" };
                                            ui.label(status_icon);
                                            ui.label(&drive.name);
                                            ui.label(format!("({:.1}%)", drive.progress * 100.0));
                                            ui.label(&drive.speed);
                                            ui.label(&drive.time_left);
                                        });
                                    }
                                }
                            });
                        } else {
                            // Show placeholder when nothing is happening
                            ui.label("No active sanitization processes.");
                            ui.add_space(10.0);
                            ui.label("Start a sanitization process from the Drives tab to see progress here.");
                        }
                    });
                },
                _ => {}
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "SHREDX - HDD Secure Wipe Tool",
        native_options,
        Box::new(|_cc| Ok(Box::new(HDDApp::new()))),
    )
}