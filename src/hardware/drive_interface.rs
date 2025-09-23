use std::io;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, GENERIC_READ, GENERIC_WRITE},
        Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
        System::IO::DeviceIoControl,
    },
};
use crate::core::{DriveGeometry, SecurityFeatures, WipeError, WipeErrorCode, WipeResult2};

pub const ATA_IDENTIFY_DEVICE: u8 = 0xEC;
pub const ATA_READ_NATIVE_MAX_ADDRESS: u8 = 0xF8;
pub const ATA_READ_NATIVE_MAX_ADDRESS_EXT: u8 = 0x27;
pub const ATA_SET_MAX_ADDRESS: u8 = 0xF9;
pub const ATA_SET_MAX_ADDRESS_EXT: u8 = 0x37;
pub const ATA_SECURITY_ERASE_PREPARE: u8 = 0xF3;
pub const ATA_SECURITY_ERASE_UNIT: u8 = 0xF4;
pub const ATA_SANITIZE_DEVICE: u8 = 0xB4;

const IOCTL_ATA_PASS_THROUGH: u32 = 0x0004D02C;
const IOCTL_ATA_PASS_THROUGH_DIRECT: u32 = 0x0004D030;

#[repr(C)]
pub struct AtaPassThroughEx {
    pub length: u16,
    pub ata_flags: u16,
    pub path_id: u8,
    pub target_id: u8,
    pub lun: u8,
    pub reserved_as_uchar: u8,
    pub data_transfer_length: u32,
    pub timeout_value: u32,
    pub reserved_as_ulong: u32,
    pub data_buffer_offset: usize,
    pub previous_task_file: [u8; 8],
    pub current_task_file: [u8; 8],
}

#[repr(C)]
pub struct IdentifyDeviceData {
    pub data: [u16; 256],
}

pub struct DriveInterface {
    handle: HANDLE,
    drive_path: String,
}

impl DriveInterface {
    pub fn new(drive_path: &str) -> WipeResult2<Self> {
        let drive_path_wide: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();
        
        unsafe {
            let handle = CreateFileW(
                PCWSTR::from_raw(drive_path_wide.as_ptr()),
                GENERIC_READ.0 | GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            );

            match handle {
                Ok(h) if h != INVALID_HANDLE_VALUE => Ok(Self {
                    handle: h,
                    drive_path: drive_path.to_string(),
                }),
                _ => Err(WipeError {
                    code: WipeErrorCode::AccessDenied,
                    message: format!("Failed to open drive: {}", drive_path),
                    sector: None,
                }),
            }
        }
    }

    pub fn identify_device(&self) -> WipeResult2<IdentifyDeviceData> {
        let mut identify_data = IdentifyDeviceData {
            data: [0; 256],
        };

        let mut pass_through = AtaPassThroughEx {
            length: std::mem::size_of::<AtaPassThroughEx>() as u16,
            ata_flags: 0x02, // ATA_FLAGS_DATA_IN
            path_id: 0,
            target_id: 0,
            lun: 0,
            reserved_as_uchar: 0,
            data_transfer_length: 512,
            timeout_value: 10,
            reserved_as_ulong: 0,
            data_buffer_offset: std::mem::size_of::<AtaPassThroughEx>(),
            previous_task_file: [0; 8],
            current_task_file: [0, 0, 0, 0, 0, 0, ATA_IDENTIFY_DEVICE, 0],
        };

        let mut bytes_returned = 0u32;
        let buffer_size = std::mem::size_of::<AtaPassThroughEx>() + 512;
        let mut buffer = vec![0u8; buffer_size];

        unsafe {
            std::ptr::copy_nonoverlapping(
                &pass_through as *const _ as *const u8,
                buffer.as_mut_ptr(),
                std::mem::size_of::<AtaPassThroughEx>(),
            );

            let result = DeviceIoControl(
                self.handle,
                IOCTL_ATA_PASS_THROUGH,
                Some(buffer.as_ptr() as _),
                buffer_size as u32,
                Some(buffer.as_mut_ptr() as _),
                buffer_size as u32,
                Some(&mut bytes_returned),
                None,
            );

            if result.is_ok() {
                std::ptr::copy_nonoverlapping(
                    buffer.as_ptr().add(std::mem::size_of::<AtaPassThroughEx>()),
                    &mut identify_data as *mut _ as *mut u8,
                    512,
                );
                Ok(identify_data)
            } else {
                Err(WipeError {
                    code: WipeErrorCode::HardwareError,
                    message: "Failed to execute IDENTIFY DEVICE command".to_string(),
                    sector: None,
                })
            }
        }
    }

    pub fn read_native_max_address(&self, use_ext: bool) -> WipeResult2<u64> {
        let command = if use_ext { 
            ATA_READ_NATIVE_MAX_ADDRESS_EXT 
        } else { 
            ATA_READ_NATIVE_MAX_ADDRESS 
        };

        let mut pass_through = AtaPassThroughEx {
            length: std::mem::size_of::<AtaPassThroughEx>() as u16,
            ata_flags: 0x02, // ATA_FLAGS_DATA_IN
            path_id: 0,
            target_id: 0,
            lun: 0,
            reserved_as_uchar: 0,
            data_transfer_length: 0,
            timeout_value: 10,
            reserved_as_ulong: 0,
            data_buffer_offset: std::mem::size_of::<AtaPassThroughEx>(),
            previous_task_file: [0; 8],
            current_task_file: [0, 0, 0, 0, 0, 0, command, 0],
        };

        let mut bytes_returned = 0u32;
        let buffer_size = std::mem::size_of::<AtaPassThroughEx>();
        let mut buffer = vec![0u8; buffer_size];

        unsafe {
            std::ptr::copy_nonoverlapping(
                &pass_through as *const _ as *const u8,
                buffer.as_mut_ptr(),
                buffer_size,
            );

            let result = DeviceIoControl(
                self.handle,
                IOCTL_ATA_PASS_THROUGH,
                Some(buffer.as_ptr() as _),
                buffer_size as u32,
                Some(buffer.as_mut_ptr() as _),
                buffer_size as u32,
                Some(&mut bytes_returned),
                None,
            );

            if result.is_ok() {
                std::ptr::copy_nonoverlapping(
                    buffer.as_ptr(),
                    &mut pass_through as *mut _ as *mut u8,
                    buffer_size,
                );

                let lba = if use_ext {
                    // 48-bit LBA from task file
                    ((pass_through.current_task_file[5] as u64) << 40) |
                    ((pass_through.current_task_file[4] as u64) << 32) |
                    ((pass_through.current_task_file[3] as u64) << 24) |
                    ((pass_through.current_task_file[2] as u64) << 16) |
                    ((pass_through.current_task_file[1] as u64) << 8) |
                    (pass_through.current_task_file[0] as u64)
                } else {
                    // 28-bit LBA from task file
                    ((pass_through.current_task_file[3] as u64 & 0x0F) << 24) |
                    ((pass_through.current_task_file[2] as u64) << 16) |
                    ((pass_through.current_task_file[1] as u64) << 8) |
                    (pass_through.current_task_file[0] as u64)
                };

                Ok(lba)
            } else {
                Err(WipeError {
                    code: WipeErrorCode::HardwareError,
                    message: "Failed to read native max address".to_string(),
                    sector: None,
                })
            }
        }
    }

    pub fn set_max_address(&self, lba: u64, use_ext: bool) -> WipeResult2<()> {
        let command = if use_ext { 
            ATA_SET_MAX_ADDRESS_EXT 
        } else { 
            ATA_SET_MAX_ADDRESS 
        };

        let mut task_file = [0u8; 8];
        if use_ext {
            // 48-bit LBA
            task_file[0] = (lba & 0xFF) as u8;
            task_file[1] = ((lba >> 8) & 0xFF) as u8;
            task_file[2] = ((lba >> 16) & 0xFF) as u8;
            task_file[3] = ((lba >> 24) & 0xFF) as u8;
            task_file[4] = ((lba >> 32) & 0xFF) as u8;
            task_file[5] = ((lba >> 40) & 0xFF) as u8;
        } else {
            // 28-bit LBA
            task_file[0] = (lba & 0xFF) as u8;
            task_file[1] = ((lba >> 8) & 0xFF) as u8;
            task_file[2] = ((lba >> 16) & 0xFF) as u8;
            task_file[3] = (((lba >> 24) & 0x0F) | 0xE0) as u8; // LBA mode + upper 4 bits
        }
        task_file[6] = command;

        let mut pass_through = AtaPassThroughEx {
            length: std::mem::size_of::<AtaPassThroughEx>() as u16,
            ata_flags: 0x00, // No data transfer
            path_id: 0,
            target_id: 0,
            lun: 0,
            reserved_as_uchar: 0,
            data_transfer_length: 0,
            timeout_value: 30,
            reserved_as_ulong: 0,
            data_buffer_offset: std::mem::size_of::<AtaPassThroughEx>(),
            previous_task_file: [0; 8],
            current_task_file: task_file,
        };

        let mut bytes_returned = 0u32;
        let buffer_size = std::mem::size_of::<AtaPassThroughEx>();
        let mut buffer = vec![0u8; buffer_size];

        unsafe {
            std::ptr::copy_nonoverlapping(
                &pass_through as *const _ as *const u8,
                buffer.as_mut_ptr(),
                buffer_size,
            );

            let result = DeviceIoControl(
                self.handle,
                IOCTL_ATA_PASS_THROUGH,
                Some(buffer.as_ptr() as _),
                buffer_size as u32,
                Some(buffer.as_mut_ptr() as _),
                buffer_size as u32,
                Some(&mut bytes_returned),
                None,
            );

            if result.is_ok() {
                Ok(())
            } else {
                Err(WipeError {
                    code: WipeErrorCode::HardwareError,
                    message: "Failed to set max address".to_string(),
                    sector: None,
                })
            }
        }
    }

    pub fn parse_drive_geometry(&self, identify_data: &IdentifyDeviceData) -> DriveGeometry {
        let words = &identify_data.data;
        
        // Extract model, serial, firmware
        let model = Self::extract_ata_string(&words[27..47]);
        let serial = Self::extract_ata_string(&words[10..20]);
        let firmware = Self::extract_ata_string(&words[23..27]);

        // Check for 48-bit addressing support
        let use_ext = words[83] & 0x0400 != 0;
        
        // Get user-addressable capacity
        let user_capacity = if use_ext {
            ((words[103] as u64) << 48) | ((words[102] as u64) << 32) | 
            ((words[101] as u64) << 16) | (words[100] as u64)
        } else {
            ((words[61] as u64) << 16) | (words[60] as u64)
        };

        // Try to get native capacity
        let native_capacity = self.read_native_max_address(use_ext).unwrap_or(user_capacity);
        
        // Check for HPA
        let has_hpa = native_capacity > user_capacity;
        let hpa_size = if has_hpa { 
            (native_capacity - user_capacity) * 512 
        } else { 
            0 
        };

        // Check for DCO support
        let has_dco = words[83] & 0x0800 != 0;
        let dco_size = 0; // DCO size detection requires manufacturer-specific methods

        DriveGeometry {
            model,
            serial,
            firmware,
            total_sectors: native_capacity,
            sector_size: 512,
            user_capacity: user_capacity * 512,
            native_capacity: native_capacity * 512,
            has_hpa,
            has_dco,
            hpa_size,
            dco_size,
        }
    }

    pub fn parse_security_features(&self, identify_data: &IdentifyDeviceData) -> SecurityFeatures {
        let words = &identify_data.data;
        
        // Word 128: Security features
        let security_word = words[128];
        
        SecurityFeatures {
            security_supported: security_word & 0x0001 != 0,
            security_enabled: security_word & 0x0002 != 0,
            security_locked: security_word & 0x0004 != 0,
            security_frozen: security_word & 0x0008 != 0,
            enhanced_erase_supported: security_word & 0x0020 != 0,
            sanitize_supported: words[59] & 0x1000 != 0, // Word 59: Sanitize feature
            crypto_scramble_supported: words[69] & 0x0001 != 0, // Word 69: Additional supported
        }
    }

    fn extract_ata_string(words: &[u16]) -> String {
        let mut bytes = Vec::new();
        for &word in words {
            bytes.push((word >> 8) as u8);   // High byte first
            bytes.push((word & 0xFF) as u8); // Low byte second
        }
        
        // Remove trailing spaces and null bytes
        while let Some(&last) = bytes.last() {
            if last == 0 || last == b' ' {
                bytes.pop();
            } else {
                break;
            }
        }
        
        String::from_utf8_lossy(&bytes).trim().to_string()
    }

    pub fn unlock_hpa(&self) -> WipeResult2<bool> {
        let identify_data = self.identify_device()?;
        let words = &identify_data.data;
        let use_ext = words[83] & 0x0400 != 0;
        
        let current_max = if use_ext {
            ((words[103] as u64) << 48) | ((words[102] as u64) << 32) | 
            ((words[101] as u64) << 16) | (words[100] as u64)
        } else {
            ((words[61] as u64) << 16) | (words[60] as u64)
        };
        
        let native_max = self.read_native_max_address(use_ext)?;
        
        if native_max > current_max {
            self.set_max_address(native_max, use_ext)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Drop for DriveInterface {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}