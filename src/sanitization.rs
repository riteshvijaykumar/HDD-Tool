use std::fs::{File, OpenOptions, read_dir, remove_file, create_dir_all};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use rand::Rng;

#[derive(Debug, Clone)]
pub enum SanitizationMethod {
    Clear,      // NIST 800-88 Clear: Single pass overwrite
    Purge,      // NIST 800-88 Purge: Multiple pass overwrite
}

#[derive(Debug, Clone)]
pub enum SanitizationPattern {
    Zeros,      // 0x00
    Ones,       // 0xFF
    Random,     // Random data
    DoD5220,    // DoD 5220.22-M pattern
    Custom(u8), // Custom byte pattern
}

#[derive(Debug)]
pub struct SanitizationProgress {
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub current_pass: u32,
    pub total_passes: u32,
    pub percentage: f64,
}

pub struct DataSanitizer {
    buffer_size: usize,
}

impl DataSanitizer {
    pub fn new() -> Self {
        Self {
            buffer_size: 1024 * 1024, // 1MB buffer
        }
    }

    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self { buffer_size }
    }

    /// NIST 800-88 Clear method - Single pass overwrite
    pub fn clear<P: AsRef<Path>>(
        &self,
        device_path: P,
        pattern: SanitizationPattern,
        progress_callback: Option<Box<dyn Fn(SanitizationProgress)>>,
    ) -> io::Result<()> {
        self.sanitize_device(device_path, vec![pattern], progress_callback)
    }

    /// NIST 800-88 Purge method - Multiple pass overwrite
    pub fn purge<P: AsRef<Path>>(
        &self,
        device_path: P,
        progress_callback: Option<Box<dyn Fn(SanitizationProgress)>>,
    ) -> io::Result<()> {
        // DoD 5220.22-M three-pass method
        let patterns = vec![
            SanitizationPattern::Random,
            SanitizationPattern::Custom(0x55), // 01010101
            SanitizationPattern::Custom(0xAA), // 10101010
        ];
        
        self.sanitize_device(device_path, patterns, progress_callback)
    }

    /// Enhanced Purge method with more passes for highly sensitive data
    pub fn enhanced_purge<P: AsRef<Path>>(
        &self,
        device_path: P,
        progress_callback: Option<Box<dyn Fn(SanitizationProgress)>>,
    ) -> io::Result<()> {
        // Gutmann method (simplified) - 7 passes
        let patterns = vec![
            SanitizationPattern::Random,
            SanitizationPattern::Custom(0x55),
            SanitizationPattern::Custom(0xAA),
            SanitizationPattern::Custom(0x92),
            SanitizationPattern::Custom(0x49),
            SanitizationPattern::Custom(0x24),
            SanitizationPattern::Random,
        ];
        
        self.sanitize_device(device_path, patterns, progress_callback)
    }

    /// File-level sanitization for when direct device access fails
    /// This method overwrites all files on the drive and fills free space
    pub fn sanitize_files_and_free_space<P: AsRef<Path>>(
        &self,
        drive_root: P,
        passes: u32,
        _progress_callback: Option<Box<dyn Fn(SanitizationProgress)>>,
    ) -> io::Result<()> {
        let drive_path = drive_root.as_ref();
        
        println!("🔧 Starting file-level sanitization on {}", drive_path.display());
        
        // Check if the drive path exists and is accessible
        if !drive_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, 
                format!("Drive path {} does not exist", drive_path.display())));
        }
        
        if !drive_path.is_dir() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, 
                format!("Path {} is not a directory", drive_path.display())));
        }
        
        // Step 1: Overwrite all existing files
        println!("🗂️  Phase 1: Overwriting all existing files...");
        match self.overwrite_all_files(drive_path, passes) {
            Ok(_) => println!("✅ File overwriting completed"),
            Err(e) => {
                println!("❌ File overwriting failed: {}", e);
                return Err(e);
            }
        }
        
        // Step 2: Fill free space with random data
        println!("💾 Phase 2: Filling free space with random data...");
        match self.fill_free_space(drive_path, passes) {
            Ok(_) => println!("✅ Free space filling completed"),
            Err(e) => {
                println!("❌ Free space filling failed: {}", e);
                return Err(e);
            }
        }
        
        println!("✅ File-level sanitization completed");
        Ok(())
    }

    /// Recursively overwrite all files in a directory
    fn overwrite_all_files(&self, dir: &Path, passes: u32) -> io::Result<()> {
        if !dir.is_dir() {
            println!("❌ Path is not a directory: {}", dir.display());
            return Ok(());
        }

        println!("🔍 Scanning directory: {}", dir.display());
        
        let entries = match read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                println!("❌ Failed to read directory {}: {}", dir.display(), e);
                return Err(e);
            }
        };

        let mut file_count = 0;
        let mut dir_count = 0;

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    println!("❌ Failed to read directory entry: {}", e);
                    continue;
                }
            };
            
            let path = entry.path();

            if path.is_dir() {
                dir_count += 1;
                println!("📁 Processing subdirectory: {}", path.display());
                // Recursively process subdirectories
                if let Err(e) = self.overwrite_all_files(&path, passes) {
                    println!("❌ Failed to process subdirectory {}: {}", path.display(), e);
                }
            } else if path.is_file() {
                file_count += 1;
                println!("📄 Found file: {}", path.display());
                
                // Overwrite the file multiple times
                for pass in 1..=passes {
                    println!("  🔄 Pass {}/{}: Overwriting {}", pass, passes, path.display());
                    if let Err(e) = self.overwrite_single_file(&path) {
                        println!("  ❌ Failed to overwrite {}: {}", path.display(), e);
                        continue;
                    }
                }
                
                // Delete the file after overwriting
                match remove_file(&path) {
                    Ok(_) => println!("  ✅ Deleted: {}", path.display()),
                    Err(e) => println!("  ❌ Failed to delete {}: {}", path.display(), e),
                }
            }
        }
        
        println!("📊 Directory scan complete: {} files, {} subdirectories processed", file_count, dir_count);
        Ok(())
    }

    /// Overwrite a single file with random data
    fn overwrite_single_file(&self, file_path: &Path) -> io::Result<()> {
        if let Ok(metadata) = file_path.metadata() {
            if metadata.len() == 0 {
                return Ok(()); // Skip empty files
            }

            let mut file = OpenOptions::new()
                .write(true)
                .truncate(false)
                .open(file_path)?;

            let file_size = metadata.len();
            let mut bytes_written = 0u64;

            while bytes_written < file_size {
                let remaining = file_size - bytes_written;
                let write_size = std::cmp::min(self.buffer_size as u64, remaining) as usize;
                
                let mut buffer = vec![0u8; write_size];
                self.fill_random(&mut buffer);
                
                file.write_all(&buffer)?;
                bytes_written += write_size as u64;
            }
            
            file.flush()?;
        }
        Ok(())
    }

    /// Fill free space with random data
    fn fill_free_space(&self, drive_path: &Path, passes: u32) -> io::Result<()> {
        for pass in 1..=passes {
            println!("  Pass {}/{}: Filling free space on {}", pass, passes, drive_path.display());
            
            // Create a temporary directory for our fill files
            let temp_dir = drive_path.join("__sanitize_temp__");
            let _ = create_dir_all(&temp_dir);

            let mut file_counter = 0;
            let chunk_size = 50 * 1024 * 1024; // 50MB chunks

            loop {
                let temp_file = temp_dir.join(format!("fill_{}.tmp", file_counter));
                
                match File::create(&temp_file) {
                    Ok(mut file) => {
                        let mut buffer = vec![0u8; chunk_size];
                        self.fill_random(&mut buffer);
                        
                        match file.write_all(&buffer) {
                            Ok(_) => {
                                file_counter += 1;
                                if file_counter % 10 == 0 {
                                    println!("    Created {} fill files...", file_counter);
                                }
                            },
                            Err(_) => {
                                // Disk is probably full, stop creating files
                                let _ = remove_file(&temp_file);
                                break;
                            }
                        }
                    },
                    Err(_) => {
                        // Can't create more files, disk is probably full
                        break;
                    }
                }
            }

            // Clean up temporary files
            if temp_dir.exists() {
                let _ = std::fs::remove_dir_all(&temp_dir);
            }
        }
        Ok(())
    }

    /// Core sanitization implementation
    fn sanitize_device<P: AsRef<Path>>(
        &self,
        device_path: P,
        patterns: Vec<SanitizationPattern>,
        progress_callback: Option<Box<dyn Fn(SanitizationProgress)>>,
    ) -> io::Result<()> {
        let path = device_path.as_ref();
        
        // Get device size
        let device_size = self.get_device_size(path)?;
        let total_passes = patterns.len() as u32;
        
        // Open device for writing
        let mut device = OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)?;

        for (pass_num, pattern) in patterns.iter().enumerate() {
            let current_pass = (pass_num + 1) as u32;
            
            // Seek to beginning of device
            device.seek(SeekFrom::Start(0))?;
            
            // Generate pattern buffer
            let mut buffer = self.generate_pattern_buffer(pattern, self.buffer_size);
            let mut bytes_written = 0u64;
            
            while bytes_written < device_size {
                let remaining = device_size - bytes_written;
                let write_size = std::cmp::min(self.buffer_size as u64, remaining) as usize;
                
                // Adjust buffer size for last chunk if necessary
                if write_size < self.buffer_size {
                    buffer.truncate(write_size);
                    if matches!(pattern, SanitizationPattern::Random) {
                        self.fill_random(&mut buffer);
                    }
                }
                
                // Write pattern to device
                device.write_all(&buffer[..write_size])?;
                device.flush()?;
                
                bytes_written += write_size as u64;
                
                // Report progress
                if let Some(ref callback) = progress_callback {
                    let progress = SanitizationProgress {
                        bytes_processed: bytes_written,
                        total_bytes: device_size,
                        current_pass,
                        total_passes,
                        percentage: (bytes_written as f64 / device_size as f64) * 100.0,
                    };
                    callback(progress);
                }
            }
            
            // Force sync to disk
            device.sync_all()?;
        }
        
        Ok(())
    }

    /// Get the size of a device/file
    fn get_device_size<P: AsRef<Path>>(&self, path: P) -> io::Result<u64> {
        let mut file = File::open(path)?;
        file.seek(SeekFrom::End(0))
    }

    /// Generate a buffer filled with the specified pattern
    fn generate_pattern_buffer(&self, pattern: &SanitizationPattern, size: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; size];
        
        match pattern {
            SanitizationPattern::Zeros => {
                // Buffer is already filled with zeros
            }
            SanitizationPattern::Ones => {
                buffer.fill(0xFF);
            }
            SanitizationPattern::Random => {
                self.fill_random(&mut buffer);
            }
            SanitizationPattern::Custom(byte) => {
                buffer.fill(*byte);
            }
            SanitizationPattern::DoD5220 => {
                // DoD 5220.22-M uses alternating patterns
                for (i, byte) in buffer.iter_mut().enumerate() {
                    *byte = if i % 2 == 0 { 0x55 } else { 0xAA };
                }
            }
        }
        
        buffer
    }

    /// Fill buffer with cryptographically secure random data
    fn fill_random(&self, buffer: &mut [u8]) {
        let mut rng = rand::thread_rng();
        rng.fill(buffer);
    }

    /// Verify sanitization by reading and checking patterns
    pub fn verify_sanitization<P: AsRef<Path>>(
        &self,
        device_path: P,
        expected_pattern: SanitizationPattern,
        sample_size: Option<u64>,
    ) -> io::Result<bool> {
        let path = device_path.as_ref();
        let mut device = File::open(path)?;
        let device_size = self.get_device_size(path)?;
        
        let check_size = sample_size.unwrap_or(std::cmp::min(device_size, 1024 * 1024)); // Default 1MB sample
        let mut buffer = vec![0u8; check_size as usize];
        
        device.read_exact(&mut buffer)?;
        
        // For random patterns, we can't verify the exact content
        // Instead, we check that it's not all zeros or all ones
        match expected_pattern {
            SanitizationPattern::Random => {
                let all_zeros = buffer.iter().all(|&b| b == 0);
                let all_ones = buffer.iter().all(|&b| b == 0xFF);
                Ok(!all_zeros && !all_ones)
            }
            SanitizationPattern::Zeros => {
                Ok(buffer.iter().all(|&b| b == 0))
            }
            SanitizationPattern::Ones => {
                Ok(buffer.iter().all(|&b| b == 0xFF))
            }
            SanitizationPattern::Custom(expected) => {
                Ok(buffer.iter().all(|&b| b == expected))
            }
            SanitizationPattern::DoD5220 => {
                Ok(buffer.iter().enumerate().all(|(i, &b)| {
                    if i % 2 == 0 { b == 0x55 } else { b == 0xAA }
                }))
            }
        }
    }
}

/// SSD-specific sanitization using ATA Secure Erase
#[cfg(windows)]
pub mod ssd_sanitization {
    use windows::{
        core::PWSTR,
        Win32::{
            Foundation::{CloseHandle, HANDLE},
            Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_NONE, OPEN_EXISTING},
        },
    };

    pub fn secure_erase_ssd(drive_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let drive_path_wide: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();
            let drive_path_pwstr = PWSTR::from_raw(drive_path_wide.as_ptr() as *mut u16);

            let handle = CreateFileW(
                drive_path_pwstr,
                0x40000000u32, // GENERIC_WRITE
                FILE_SHARE_NONE,             // No sharing
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            )?;

            // This is a simplified example - real implementation would need:
            // 1. Check if drive supports secure erase
            // 2. Issue SECURITY SET PASSWORD command
            // 3. Issue SECURITY ERASE UNIT command
            // 4. Verify completion

            CloseHandle(handle)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clear_method() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"sensitive data").unwrap();
        
        let sanitizer = DataSanitizer::new();
        sanitizer.clear(temp_file.path(), SanitizationPattern::Zeros, None).unwrap();
        
        let verification = sanitizer.verify_sanitization(
            temp_file.path(), 
            SanitizationPattern::Zeros, 
            None
        ).unwrap();
        
        assert!(verification);
    }

    #[test]
    fn test_pattern_generation() {
        let sanitizer = DataSanitizer::new();
        
        let zeros = sanitizer.generate_pattern_buffer(&SanitizationPattern::Zeros, 100);
        assert!(zeros.iter().all(|&b| b == 0));
        
        let ones = sanitizer.generate_pattern_buffer(&SanitizationPattern::Ones, 100);
        assert!(ones.iter().all(|&b| b == 0xFF));
        
        let custom = sanitizer.generate_pattern_buffer(&SanitizationPattern::Custom(0x42), 100);
        assert!(custom.iter().all(|&b| b == 0x42));
    }
}