use eframe::egui;
use windows::{
    core::PWSTR,
    Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
    },
};

mod sanitization;
use sanitization::{DataSanitizer, SanitizationMethod, SanitizationProgress};

#[derive(Debug, Clone)]
struct DiskInfo {
    drive_letter: String,
    drive_type: String,
    file_system: String,
    total_space: u64,
    free_space: u64,
    used_space: u64,
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
}

impl HDDApp {
    fn new() -> Self {
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

            // Get drive type
            let drive_type_raw = GetDriveTypeW(drive_path_pwstr);
            let drive_type = match drive_type_raw {
                3 => "Fixed Drive (HDD/SSD)",    // DRIVE_FIXED
                2 => "Removable Drive",          // DRIVE_REMOVABLE
                4 => "Network Drive",            // DRIVE_REMOTE
                5 => "CD-ROM Drive",             // DRIVE_CDROM
                6 => "RAM Disk",                 // DRIVE_RAMDISK
                1 => "Unknown",                  // DRIVE_UNKNOWN
                0 => "Cannot Determine",         // DRIVE_NO_ROOT_DIR
                _ => "Other",
            }.to_string();

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

            let used_space = total_bytes.saturating_sub(free_bytes);

            Some(DiskInfo {
                drive_letter: drive_path[..2].to_string(), // Get just "E:" instead of "E"
                drive_type,
                file_system,
                total_space: total_bytes,
                free_space: free_bytes,
                used_space,
            })
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
        };
        
        let size_mb = disk.total_space / (1024 * 1024);
        let time_seconds = (size_mb / write_speed_mbs) * passes;
        time_seconds / 60 // Convert to minutes
    }

    fn execute_real_sanitization(&mut self, method: SanitizationMethod, disk: &DiskInfo) -> Result<(), String> {
        println!("🔄 Starting {} sanitization for drive {}", 
                 match method { SanitizationMethod::Clear => "CLEAR", SanitizationMethod::Purge => "PURGE" }, 
                 disk.drive_letter);
        
        // Check if this is the system drive
        if disk.drive_letter == "C" {
            let error_msg = format!("❌ Cannot sanitize system drive {} - this would make your computer unbootable!", disk.drive_letter);
            println!("{}", error_msg);
            return Err(error_msg);
        }
        
        // Warning for data drives
        println!("⚠️  WARNING: About to permanently erase all data on drive {}", disk.drive_letter);
        println!("⚠️  Drive contains: {} total space", Self::format_bytes(disk.total_space));
        println!("⚠️  Note: This requires Administrator privileges and the drive must not be in use");
        
        // Try different path formats for Windows device access
        let device_paths = vec![
            format!("\\\\.\\{}", disk.drive_letter),            // Standard raw device path (E:)
        ];
        
        let mut last_error = String::new();
        
        // First try direct device access
        for device_path in device_paths.iter() {
            println!("🔧 Attempting direct device access: {}", device_path);
            
            let result = match method {
                SanitizationMethod::Clear => {
                    self.sanitizer.clear(device_path, sanitization::SanitizationPattern::Random, None)
                },
                SanitizationMethod::Purge => {
                    self.sanitizer.purge(device_path, None)
                }
            };
            
            match result {
                Ok(_) => {
                    println!("✅ Direct {} sanitization completed for {}", 
                             match method { SanitizationMethod::Clear => "Clear", SanitizationMethod::Purge => "Purge" },
                             disk.drive_letter);
                    return Ok(());
                },
                Err(e) => {
                    last_error = format!("Direct access failed: {}", e);
                    println!("❌ Direct device access failed: {}", e);
                }
            }
        }
        
        // If direct access failed, try file-level sanitization
        println!("🔧 Falling back to file-level sanitization...");
        let drive_root = format!("{}\\", disk.drive_letter); // Now disk.drive_letter is "E:" so this becomes "E:\"
        let passes = match method {
            SanitizationMethod::Clear => 1,
            SanitizationMethod::Purge => 3,
        };
        
        println!("🔧 Using drive root path: {}", drive_root);
        
        match self.sanitizer.sanitize_files_and_free_space(&drive_root, passes, None) {
            Ok(_) => {
                println!("✅ File-level {} sanitization completed for drive {}", 
                         match method { SanitizationMethod::Clear => "Clear", SanitizationMethod::Purge => "Purge" },
                         disk.drive_letter);
                return Ok(());
            },
            Err(e) => {
                last_error = format!("{}; File-level sanitization also failed: {}", last_error, e);
                println!("❌ File-level sanitization error: {}", e);
            }
        }
        
        // If all paths failed, return comprehensive error
        let error_msg = format!(
            "❌ All sanitization attempts failed for drive {}:\n{}\n\n💡 Solutions to try:\n\
            • Right-click app and 'Run as Administrator'\n\
            • Ensure no files are open on the drive\n\
            • Try ejecting and reinserting USB drive\n\
            • Use 'diskpart' command line tool as alternative\n\
            • Some drives may have write protection",
            disk.drive_letter, last_error
        );
        
        println!("{}", error_msg);
        Err(error_msg)
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
                            egui::Button::new(&disk.drive_type)
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
                            
                            ui.label(format!("Drive Type: {}", disk.drive_type));
                            ui.label(format!("File System: {}", disk.file_system));
                            ui.label(format!("Total Space: {}", Self::format_bytes(disk.total_space)));
                            
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
                                
                                // Real NIST 800-88 Clear method
                                if ui.add_sized([200.0, 30.0], egui::Button::new("🗑️ CLEAR (Single Pass)")).clicked() {
                                    match self.execute_real_sanitization(SanitizationMethod::Clear, &disk) {
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
                                    match self.execute_real_sanitization(SanitizationMethod::Purge, &disk) {
                                        Ok(_) => {
                                            self.last_error_message = Some("✅ Sanitization completed successfully!".to_string());
                                        },
                                        Err(e) => {
                                            self.last_error_message = Some(e);
                                        }
                                    }
                                }
                                ui.label("NIST 800-88 Purge: 3-pass DoD method");
                                
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