# HDD Tool - Cross-Platform Implementation

## Overview
The HDD Tool has been successfully converted to a cross-platform application that runs on both Windows and Linux operating systems while maintaining full NIST SP 800-88 compliance and device-specific sanitization capabilities.

## Platform Support

### Windows
- Native Windows API integration
- Direct drive access via CreateFile/DeviceIoControl
- Windows-specific drive enumeration
- ATA command pass-through support

### Linux
- Linux system calls via libc and nix crates
- Drive detection through sysinfo crate
- hdparm integration for SSD secure erase
- Unix device access patterns

## Key Features

### Cross-Platform Architecture
- **Platform Abstraction Layer**: `src/platform.rs` provides unified interface for system operations
- **Conditional Compilation**: Uses `cfg(windows)` and `cfg(unix)` for platform-specific code
- **Unified GUI**: eframe/egui provides consistent interface across platforms

### Device-Specific Sanitization
- **HDD Erasure**: Multi-pass overwriting with NIST-compliant patterns
- **SSD Sanitization**: 
  - Windows: Native secure erase commands
  - Linux: hdparm-based secure erase
- **NVMe Support**: NVMe-specific commands and deallocate operations
- **USB Device Handling**: Specialized algorithms for USB storage
- **SD Card Management**: Wear-leveling aware sanitization

### NIST SP 800-88 Compliance
- **Clear**: Single-pass overwrite for basic data removal
- **Purge**: Multi-pass cryptographic overwrite
- **Destroy**: Physical destruction guidance
- **Verification**: Post-sanitization validation

## Technical Implementation

### Dependencies
```toml
# Cross-platform GUI
eframe = "0.29"
egui = "0.29"

# Cross-platform system information
sysinfo = "0.32"

# Platform-specific dependencies
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = ["Win32_Foundation", "Win32_Storage_FileSystem", "Win32_System_IO"] }

[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["ioctl"] }
libc = "0.2"
```

### Cross-Platform Drive Detection
```rust
// Windows implementation
#[cfg(windows)]
pub fn get_system_drives() -> Vec<DriveInfo> {
    // Uses GetLogicalDrives() and Windows APIs
}

// Linux implementation
#[cfg(unix)]
pub fn get_system_drives() -> Vec<DriveInfo> {
    // Uses sysinfo crate for drive enumeration
}
```

### Device-Specific Modules
- `src/devices/hdd/` - Traditional hard disk drives
- `src/devices/ssd/` - Solid-state drives
- `src/devices/nvme/` - NVMe storage devices
- `src/devices/usb/` - USB storage devices
- `src/devices/sdcard/` - SD cards and flash memory

## Building and Running

### Prerequisites
- Rust 2024 Edition
- Platform-specific tools:
  - Windows: No additional tools required
  - Linux: hdparm package for SSD secure erase

### Build Commands
```bash
# Build for current platform
cargo build

# Run the application
cargo run

# Build release version
cargo build --release
```

### Cross-Compilation
```bash
# Build for Linux from Windows (requires cross-compilation setup)
cargo build --target x86_64-unknown-linux-gnu

# Build for Windows from Linux (requires cross-compilation setup)
cargo build --target x86_64-pc-windows-gnu
```

## Security Features

### Sanitization Methods
1. **Single Pass Zero**: Fast zero-fill for basic clearing
2. **Three Pass**: Random, complement, random pattern
3. **Seven Pass**: DoD 5220.22-M standard
4. **NIST Purge**: Cryptographically secure multi-pass
5. **Device-Specific**: Hardware-based secure erase when available

### Validation
- Post-sanitization verification
- Cryptographic validation of data removal
- Compliance reporting and audit trails

## Usage

### GUI Interface
The application provides an intuitive graphical interface with:
- Drive selection and information display
- Sanitization method selection
- Progress monitoring
- Compliance reporting

### Device Detection
Automatically detects and categorizes:
- Internal hard drives (HDD/SSD)
- External USB storage
- SD cards and removable media
- NVMe devices

## Compliance and Standards

### NIST SP 800-88 Rev. 1
- Implements all three sanitization categories
- Provides appropriate methods for different security levels
- Includes verification and reporting capabilities

### Security Considerations
- Requires administrative privileges for direct device access
- Implements secure memory handling
- Provides audit trails for compliance verification

## Development Notes

### Code Structure
- Modular device-specific implementations
- Platform abstraction for cross-compatibility
- Comprehensive error handling and logging
- Extensive test coverage (planned)

### Future Enhancements
- Additional sanitization algorithms
- Network drive support
- Batch operation capabilities
- Enhanced reporting features

## Testing

### Platform Testing
- Windows 10/11 compatibility verified
- Linux distribution testing (Ubuntu, CentOS, etc.)
- Cross-platform GUI consistency validation

### Device Testing
- Multiple storage device types
- Various capacity ranges
- Different interface types (SATA, NVMe, USB)

## Support

For issues or questions:
1. Check the device compatibility list
2. Verify platform-specific requirements
3. Review sanitization method documentation
4. Submit issues with platform and device information

---

**Note**: This tool handles sensitive data sanitization operations. Always verify results and maintain appropriate backups before performing sanitization operations.