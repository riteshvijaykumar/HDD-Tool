// Cross-platform system operations module
// Provides unified interface for Windows and Linux drive operations

use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DriveInfo {
    pub path: String,
    pub label: String,
    pub drive_type: String,
    pub total_space: u64,
    pub free_space: u64,
}

#[cfg(windows)]
pub mod windows_impl {
    use super::*;
    use windows::{
        core::PWSTR,
        Win32::Storage::FileSystem::{
            GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
            CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, OPEN_EXISTING,
        },
        Win32::Foundation::{CloseHandle, HANDLE},
    };
    
    // Drive type constants
    const DRIVE_UNKNOWN: u32 = 0;
    const DRIVE_REMOVABLE: u32 = 2;
    const DRIVE_FIXED: u32 = 3;
    const DRIVE_REMOTE: u32 = 4;
    const DRIVE_CDROM: u32 = 5;
    const DRIVE_RAMDISK: u32 = 6;

    pub fn get_drives() -> io::Result<Vec<DriveInfo>> {
        let mut drives = Vec::new();
        
        unsafe {
            let logical_drives = GetLogicalDrives();
            
            for i in 0..26 {
                if (logical_drives & (1 << i)) != 0 {
                    let drive_letter = (b'A' + i as u8) as char;
                    let drive_path = format!("{}:\\", drive_letter);
                    
                    // Convert to wide string for Windows API
                    let drive_path_wide: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();
                    let drive_path_pwstr = PWSTR::from_raw(drive_path_wide.as_ptr() as *mut u16);
                    
                    // Get drive type
                    let drive_type = match GetDriveTypeW(drive_path_pwstr) {
                        DRIVE_FIXED => "Fixed Drive (HDD/SSD)",
                        DRIVE_REMOVABLE => "Removable Drive (USB/Floppy)",
                        DRIVE_CDROM => "CD-ROM Drive",
                        DRIVE_REMOTE => "Network Drive",
                        DRIVE_RAMDISK => "RAM Disk",
                        DRIVE_UNKNOWN => "Unknown",
                        _ => "Other",
                    }.to_string();
                    
                    // Get volume information
                    let mut volume_name_buffer = [0u16; 256];
                    let mut file_system_buffer = [0u16; 256];
                    let mut serial_number = 0u32;
                    let mut max_component_length = 0u32;
                    let mut file_system_flags = 0u32;
                    
                    let volume_info_result = GetVolumeInformationW(
                        drive_path_pwstr,
                        Some(&mut volume_name_buffer),
                        Some(&mut serial_number),
                        Some(&mut max_component_length),
                        Some(&mut file_system_flags),
                        Some(&mut file_system_buffer),
                    );
                    
                    let label = if volume_info_result.is_ok() {
                        String::from_utf16_lossy(&volume_name_buffer)
                            .trim_end_matches('\0')
                            .to_string()
                    } else {
                        format!("Drive {}", drive_letter)
                    };
                    
                    // Get disk space
                    let mut free_bytes = 0u64;
                    let mut total_bytes = 0u64;
                    let mut total_free_bytes = 0u64;
                    
                    let space_result = GetDiskFreeSpaceExW(
                        drive_path_pwstr,
                        Some(&mut free_bytes),
                        Some(&mut total_bytes),
                        Some(&mut total_free_bytes),
                    );
                    
                    let (total_space, free_space) = if space_result.is_ok() {
                        (total_bytes, free_bytes)
                    } else {
                        (0, 0)
                    };
                    
                    drives.push(DriveInfo {
                        path: drive_path,
                        label,
                        drive_type,
                        total_space,
                        free_space,
                    });
                }
            }
        }
        
        Ok(drives)
    }
}

#[cfg(unix)]
pub mod unix_impl {
    use super::*;
    use std::fs;
    use std::ffi::{CString, OsStr};
    use std::os::unix::ffi::OsStrExt;
    use sysinfo::{System, SystemExt, DiskExt};

    pub fn get_drives() -> io::Result<Vec<DriveInfo>> {
        let mut drives = Vec::new();
        let mut system = System::new_all();
        system.refresh_disks();
        
        for disk in system.disks() {
            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let name = disk.name().to_string_lossy().to_string();
            let file_system = String::from_utf8_lossy(disk.file_system()).to_string();
            
            // Determine drive type based on device path and file system
            let device_path = mount_point.clone();
            let drive_type = determine_drive_type(&device_path, &file_system);
            
            let label = if name.is_empty() {
                format!("Drive ({})", mount_point)
            } else {
                name
            };
            
            drives.push(DriveInfo {
                path: mount_point,
                label,
                drive_type,
                total_space: disk.total_space(),
                free_space: disk.available_space(),
            });
        }
        
        // Also check common device paths for unmounted drives
        let device_paths = [
            "/dev/sda", "/dev/sdb", "/dev/sdc", "/dev/sdd",
            "/dev/nvme0n1", "/dev/nvme1n1",
            "/dev/mmcblk0", "/dev/mmcblk1",
        ];
        
        for &dev_path in &device_paths {
            if Path::new(dev_path).exists() {
                // Check if this device is already mounted
                let already_mounted = drives.iter().any(|d| d.path.contains(&dev_path[5..]));
                
                if !already_mounted {
                    drives.push(DriveInfo {
                        path: dev_path.to_string(),
                        label: format!("Unmounted Device ({})", dev_path),
                        drive_type: determine_drive_type_from_path(dev_path),
                        total_space: get_device_size(dev_path).unwrap_or(0),
                        free_space: 0, // Unmounted, so no free space info
                    });
                }
            }
        }
        
        Ok(drives)
    }
    
    fn determine_drive_type(mount_point: &str, file_system: &str) -> String {
        if mount_point.starts_with("/media/") || mount_point.starts_with("/mnt/") {
            "Removable Drive (USB/External)".to_string()
        } else if file_system.contains("ntfs") || file_system.contains("fat") {
            "External Drive".to_string()
        } else if file_system.contains("ext") || file_system.contains("xfs") || file_system.contains("btrfs") {
            "Fixed Drive (HDD/SSD)".to_string()
        } else {
            format!("Drive ({})", file_system)
        }
    }
    
    fn determine_drive_type_from_path(device_path: &str) -> String {
        if device_path.contains("nvme") {
            "NVMe SSD".to_string()
        } else if device_path.contains("mmcblk") {
            "SD Card/eMMC".to_string()
        } else if device_path.starts_with("/dev/sd") {
            "SATA Drive (HDD/SSD)".to_string()
        } else {
            "Unknown Drive".to_string()
        }
    }
    
    fn get_device_size(device_path: &str) -> io::Result<u64> {
        use std::fs::File;
        use std::io::{Seek, SeekFrom};
        
        let mut file = File::open(device_path)?;
        let size = file.seek(SeekFrom::End(0))?;
        Ok(size)
    }
}

// Public interface that delegates to platform-specific implementation
pub fn get_system_drives() -> io::Result<Vec<DriveInfo>> {
    #[cfg(windows)]
    return windows_impl::get_drives();
    
    #[cfg(unix)]
    return unix_impl::get_drives();
    
    #[cfg(not(any(windows, unix)))]
    return Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Platform not supported"
    ));
}

// Cross-platform device access functions
pub fn can_access_device_directly(device_path: &str) -> bool {
    #[cfg(windows)]
    {
        // On Windows, check if we can open the device with appropriate permissions
        use windows::{
            core::PWSTR,
            Win32::{
                Foundation::HANDLE,
                Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, OPEN_EXISTING},
            },
        };
        
        unsafe {
            let device_path_wide: Vec<u16> = device_path.encode_utf16().chain(std::iter::once(0)).collect();
            let device_path_pwstr = PWSTR::from_raw(device_path_wide.as_ptr() as *mut u16);
            
            let handle = CreateFileW(
                device_path_pwstr,
                0x80000000u32, // GENERIC_READ
                FILE_SHARE_READ,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            );
            
            if let Ok(h) = handle {
                windows::Win32::Foundation::CloseHandle(h).ok();
                true
            } else {
                false
            }
        }
    }
    
    #[cfg(unix)]
    {
        // On Linux, check if we can open the device file
        use std::fs::File;
        
        match File::open(device_path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    
    #[cfg(not(any(windows, unix)))]
    false
}

pub fn get_device_path_for_sanitization(drive_info: &DriveInfo) -> String {
    #[cfg(windows)]
    {
        // On Windows, convert drive letter to physical device path
        if drive_info.path.len() >= 2 && drive_info.path.chars().nth(1) == Some(':') {
            let drive_letter = drive_info.path.chars().nth(0).unwrap();
            format!("\\\\.\\{}:", drive_letter)
        } else {
            drive_info.path.clone()
        }
    }
    
    #[cfg(unix)]
    {
        // On Linux, if it's a mount point, try to find the underlying device
        if drive_info.path.starts_with("/") && !drive_info.path.starts_with("/dev/") {
            // This is a mount point, we need to find the device
            // For now, return the path as-is, but in a real implementation
            // we'd parse /proc/mounts to find the device
            drive_info.path.clone()
        } else {
            drive_info.path.clone()
        }
    }
    
    #[cfg(not(any(windows, unix)))]
    drive_info.path.clone()
}