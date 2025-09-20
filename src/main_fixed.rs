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
        println!("⚠️ This is a SIMULATION - actual implementation would:");
        println!("   1. Require administrator privileges");
        println!("   2. Unmount the drive");
        println!("   3. Access raw device (e.g., \\\\.\\PhysicalDrive0)");
        println!("   4. Perform actual overwrite operations");
        
        // In a real implementation, you would:
        // 1. Check for admin privileges
        // 2. Get the physical drive path (e.g., \\.\PhysicalDrive0)
        // 3. Ensure the drive is unmounted
        // 4. Start sanitization in a background thread
        
        // For demonstration, we'll simulate the process
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
        
        // In real implementation, this would be:
        // let drive_path = format!("\\\\.\\{}", disk.drive_letter);
        // std::thread::spawn(move || {
        //     match method {
        //         SanitizationMethod::Clear => {
        //             sanitizer.clear(&drive_path, SanitizationPattern::Random, Some(progress_callback))
        //         }
        //         SanitizationMethod::Purge => {
        //             sanitizer.purge(&drive_path, Some(progress_callback))
        //         }
        //     }
        // });
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

                    for disk in &self.disks {
                        // Make each cell clickable by using selectable labels
                        let drive_clicked = ui.selectable_label(false, &disk.drive_letter).clicked();
                        let type_clicked = ui.selectable_label(false, &disk.drive_type).clicked();
                        let fs_clicked = ui.selectable_label(false, &disk.file_system).clicked();
                        let total_clicked = ui.selectable_label(false, Self::format_bytes(disk.total_space)).clicked();
                        let free_clicked = ui.selectable_label(false, Self::format_bytes(disk.free_space)).clicked();
                        let used_clicked = ui.selectable_label(false, Self::format_bytes(disk.used_space)).clicked();
                        
                        // If any part of the row is clicked, show popup
                        if drive_clicked || type_clicked || fs_clicked || total_clicked || free_clicked || used_clicked {
                            self.show_popup = true;
                            self.selected_disk = Some(disk.clone());
                        }
                        
                        ui.end_row();
                    }
                });

            // Show popup window if requested
            if self.show_popup {
                egui::Window::new("Disk Actions")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .show(ctx, |ui| {
                        if let Some(ref disk) = self.selected_disk {
                            ui.heading(format!("Actions for Drive {}", disk.drive_letter));
                            ui.separator();
                            
                            ui.label(format!("Drive Type: {}", disk.drive_type));
                            ui.label(format!("File System: {}", disk.file_system));
                            ui.label(format!("Total Space: {}", Self::format_bytes(disk.total_space)));
                            
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
                                ui.label("⚠️ WARNING: This will permanently erase ALL data!");
                                
                                ui.separator();
                                
                                // NIST 800-88 Clear method
                                let disk_for_clear = disk.clone();
                                if ui.add_sized([200.0, 30.0], egui::Button::new("🗑️ CLEAR (Single Pass)")).clicked() {
                                    self.start_sanitization(SanitizationMethod::Clear, disk_for_clear);
                                }
                                ui.label("NIST 800-88 Clear: Single pass overwrite");
                                
                                ui.separator();
                                
                                // NIST 800-88 Purge method
                                let disk_for_purge = disk.clone();
                                if ui.add_sized([200.0, 30.0], egui::Button::new("🔥 PURGE (Multi Pass)")).clicked() {
                                    self.start_sanitization(SanitizationMethod::Purge, disk_for_purge);
                                }
                                ui.label("NIST 800-88 Purge: 3-pass DoD method");
                                
                                ui.separator();
                                
                                // New Task button (original functionality)
                                if ui.add_sized([120.0, 30.0], egui::Button::new("📝 New Task")).clicked() {
                                    println!("New Task clicked for drive {}", disk.drive_letter);
                                    self.show_popup = false;
                                    self.selected_disk = None;
                                }
                            }
                            
                            ui.separator();
                            
                            // Close button
                            if ui.add_sized([120.0, 25.0], egui::Button::new("❌ Close")).clicked() {
                                if !self.sanitization_in_progress {
                                    self.show_popup = false;
                                    self.selected_disk = None;
                                }
                            }
                        }
                    });
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