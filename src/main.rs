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
    sanitizer: DataSanitizer,
    sanitization_in_progress: bool,
    sanitization_progress: Option<SanitizationProgress>,
}

impl HDDApp {
    fn new() -> Self {
        let mut app = Self { 
            disks: Vec::new(),
            show_popup: false,
            selected_disk: None,
            selected_disk_index: None,
            sanitizer: DataSanitizer::new(),
            sanitization_in_progress: false,
            sanitization_progress: None,
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
                drive_letter: drive_path[..drive_path.len()-1].to_string(),
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

    fn start_sanitization(&mut self, method: SanitizationMethod, disk: DiskInfo) {
        println!("⚠️ Starting {:?} sanitization for drive {}", method, disk.drive_letter);
        
        // Calculate estimated time for this specific drive
        let estimated_minutes = self.estimate_sanitization_time(&disk, &method);
        println!("⏱️ Estimated time: {} minutes", estimated_minutes);
        
        // Start the actual sanitization process
        self.sanitization_in_progress = true;
        self.sanitization_progress = Some(SanitizationProgress {
            bytes_processed: 0,
            total_bytes: disk.total_space,
            current_pass: 1,
            total_passes: match method {
                SanitizationMethod::Clear => 1,
                SanitizationMethod::Purge => 3,
            },
            percentage: 0.0,
        });

        // Execute the sanitization in a separate thread (simulation for demo)
        // In real implementation, this would call the actual sanitization methods
        let drive_path = format!("{}:", disk.drive_letter);
        match method {
            SanitizationMethod::Clear => {
                println!("🗑️ Executing NIST 800-88 Clear method on {}", drive_path);
                // In production: self.sanitizer.clear(&drive_path);
                self.simulate_sanitization_progress();
            },
            SanitizationMethod::Purge => {
                println!("🔥 Executing NIST 800-88 Purge method on {}", drive_path);
                // In production: self.sanitizer.purge(&drive_path);
                self.simulate_sanitization_progress();
            },
        }
    }

    fn simulate_sanitization_progress(&mut self) {
        // This simulates progress - in real implementation, this would be called by the sanitization thread
        if let Some(ref mut progress) = self.sanitization_progress {
            if progress.percentage < 100.0 {
                progress.percentage += 0.5; // Increment progress
                progress.bytes_processed = (progress.total_bytes as f64 * progress.percentage / 100.0) as u64;
                
                if progress.percentage >= 100.0 {
                    if progress.current_pass < progress.total_passes {
                        progress.current_pass += 1;
                        progress.percentage = 0.0;
                        progress.bytes_processed = 0;
                    } else {
                        println!("✅ Sanitization completed successfully!");
                        self.sanitization_in_progress = false;
                        self.sanitization_progress = None;
                    }
                }
            }
        }
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
        let drive_path = format!("{}:", disk.drive_letter);
        
        match method {
            SanitizationMethod::Clear => {
                match self.sanitizer.clear(&drive_path) {
                    Ok(_) => {
                        println!("✅ Clear sanitization completed for {}", drive_path);
                        Ok(())
                    },
                    Err(e) => {
                        let error_msg = format!("❌ Clear sanitization failed for {}: {}", drive_path, e);
                        println!("{}", error_msg);
                        Err(error_msg)
                    }
                }
            },
            SanitizationMethod::Purge => {
                match self.sanitizer.purge(&drive_path) {
                    Ok(_) => {
                        println!("✅ Purge sanitization completed for {}", drive_path);
                        Ok(())
                    },
                    Err(e) => {
                        let error_msg = format!("❌ Purge sanitization failed for {}: {}", drive_path, e);
                        println!("{}", error_msg);
                        Err(error_msg)
                    }
                }
            }
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
                        
                        // Make each cell selectable and highlight if selected
                        let drive_clicked = ui.selectable_label(is_selected, &disk.drive_letter).clicked();
                        let type_clicked = ui.selectable_label(is_selected, &disk.drive_type).clicked();
                        let fs_clicked = ui.selectable_label(is_selected, &disk.file_system).clicked();
                        let total_clicked = ui.selectable_label(is_selected, Self::format_bytes(disk.total_space)).clicked();
                        let free_clicked = ui.selectable_label(is_selected, Self::format_bytes(disk.free_space)).clicked();
                        let used_clicked = ui.selectable_label(is_selected, Self::format_bytes(disk.used_space)).clicked();
                        
                        // If any part of the row is clicked, show popup and select this row
                        if drive_clicked || type_clicked || fs_clicked || total_clicked || free_clicked || used_clicked {
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
                                    
                                    // Simulate progress updates
                                    self.simulate_sanitization_progress();
                                }
                            } else {
                                ui.label("⚠️ NIST 800-88 Data Sanitization");
                                ui.colored_label(egui::Color32::RED, "⚠️ WARNING: This will permanently erase ALL data!");
                                
                                ui.separator();
                                
                                // NIST 800-88 Clear method
                                if ui.add_sized([200.0, 30.0], egui::Button::new("🗑️ CLEAR (Single Pass)")).clicked() {
                                    self.start_sanitization(SanitizationMethod::Clear, disk.clone());
                                }
                                ui.label("NIST 800-88 Clear: Single pass overwrite");
                                
                                ui.separator();
                                
                                // NIST 800-88 Purge method
                                if ui.add_sized([200.0, 30.0], egui::Button::new("🔥 PURGE (Multi Pass)")).clicked() {
                                    self.start_sanitization(SanitizationMethod::Purge, disk.clone());
                                }
                                ui.label("NIST 800-88 Purge: 3-pass DoD method");
                                
                                ui.separator();
                                
                                // Real sanitization buttons (for actual implementation)
                                ui.label("🚨 REAL SANITIZATION (DANGER!)");
                                ui.horizontal(|ui| {
                                    if ui.add_sized([90.0, 25.0], egui::Button::new("🔥 REAL CLEAR")).clicked() {
                                        if let Err(e) = self.execute_real_sanitization(SanitizationMethod::Clear, &disk) {
                                            println!("Error: {}", e);
                                        }
                                    }
                                    if ui.add_sized([90.0, 25.0], egui::Button::new("💀 REAL PURGE")).clicked() {
                                        if let Err(e) = self.execute_real_sanitization(SanitizationMethod::Purge, &disk) {
                                            println!("Error: {}", e);
                                        }
                                    }
                                });
                                
                                ui.separator();
                                
                                // New Task button (original functionality)
                                if ui.add_sized([120.0, 30.0], egui::Button::new("📝 New Task")).clicked() {
                                    println!("New Task clicked for drive {}", disk.drive_letter);
                                    open = false; // Close window
                                }
                            }
                            
                            ui.separator();
                            
                            // Close button
                            if ui.add_sized([120.0, 25.0], egui::Button::new("❌ Close")).clicked() {
                                if !self.sanitization_in_progress {
                                    open = false; // Close window
                                }
                            }
                        }
                    });
                
                // Update popup state based on window state
                if !open {
                    self.show_popup = false;
                    self.selected_disk = None;
                    self.selected_disk_index = None;
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