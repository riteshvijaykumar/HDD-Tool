# HDD Tool - Device-Specific Erasure Architecture

## Overview

This project has been enhanced with a comprehensive device-specific erasure architecture that provides optimized sanitization methods for different storage device types. Each device type now has its own dedicated module with specialized erasure techniques.

## Device-Specific Architecture

### Folder Structure

```
src/devices/
├── mod.rs           # Main device factory and common interfaces
├── hdd/             # Hard Disk Drive specific methods
│   └── mod.rs
├── ssd/             # Solid State Drive specific methods
│   └── mod.rs
├── nvme/            # NVMe drive specific methods
│   └── mod.rs
├── usb/             # USB drive specific methods
│   └── mod.rs
└── sdcard/          # SD Card/MMC specific methods
    └── mod.rs
```

## Device Types and Specialized Methods

### 1. HDD (Hard Disk Drives) - `src/devices/hdd/mod.rs`

**Characteristics**: Traditional magnetic storage requiring multiple overwrite passes

**Specialized Methods**:
- **DoD 5220.22-M Erasure**: 3-pass standard (zeros, ones, random)
- **Gutmann 35-pass**: Maximum security with 35 specialized patterns
- **ATA Secure Erase**: Hardware-based erasure if supported
- **Multi-pass Random**: Configurable number of random passes

**Recommended Algorithms**:
1. DoD 5220.22-M (standard 3-pass)
2. Gutmann (maximum security 35-pass)
3. ATA Secure Erase (hardware-based)
4. Seven Pass (enhanced multi-pass)
5. Three Pass (basic multi-pass)

### 2. SSD (Solid State Drives) - `src/devices/ssd/mod.rs`

**Characteristics**: NAND flash memory with wear leveling, supports TRIM

**Specialized Methods**:
- **ATA Secure Erase**: Primary choice for SSDs
- **Cryptographic Erase**: For self-encrypting drives
- **TRIM-based Erasure**: Efficient for supported drives
- **Single-pass Overwrite**: Minimizes wear while maintaining security
- **NIST Clear**: Single pass with verification

**Recommended Algorithms**:
1. ATA Secure Erase (primary choice)
2. ATA Enhanced Secure Erase (enhanced version)
3. NVMe Crypto Erase (for self-encrypting SSDs)
4. NIST Clear (NIST approved method)
5. Random (single-pass fallback)

### 3. NVMe (Non-Volatile Memory Express) - `src/devices/nvme/mod.rs`

**Characteristics**: High-performance drives using NVMe protocol

**Specialized Methods**:
- **NVMe Secure Erase**: User data erase command
- **NVMe Cryptographic Erase**: Key rotation for encrypted drives
- **NVMe Write Zeroes**: Efficient zero-fill command
- **NVMe Deallocate**: TRIM equivalent for NVMe
- **Single-pass Overwrite**: High-performance pattern writing

**Recommended Algorithms**:
1. NVMe Secure Erase (primary choice)
2. NVMe Crypto Erase (for encrypted drives)
3. NIST Clear (NIST approved method)
4. Random (single-pass fallback)
5. Zeros (simple zero fill)

### 4. USB Drives - `src/devices/usb/mod.rs`

**Characteristics**: Portable flash storage with limited write cycles

**Specialized Methods**:
- **Single-pass Random**: Recommended for longevity
- **Single-pass Zeros**: Simple zero fill
- **Quick Format + Overwrite**: Filesystem + data erasure
- **Conservative 3-pass**: Only in non-conservative mode
- **Filesystem Secure Delete**: File-level secure deletion

**Features**:
- Conservative mode (default) to preserve drive lifespan
- Aggressive mode for higher security requirements
- Small buffer sizes to prevent timeouts
- Gentle write patterns with pauses

**Recommended Algorithms**:
1. Random (primary choice - single pass)
2. Zeros (simple zero fill)
3. File System Wipe (file-level erasure)
4. Quick Format (format + overwrite)
5. NIST Clear (NIST approved single pass)

### 5. SD Cards/MMC - `src/devices/sdcard/mod.rs`

**Characteristics**: Removable flash memory with very limited write cycles

**Specialized Methods**:
- **Single-pass Random**: Minimal wear approach
- **Single-pass Zeros**: Simple zero fill
- **SD Card Native Erase**: Hardware erase command
- **Filesystem Secure Delete**: Multi-step file-level erasure
- **Conservative 2-pass**: For critical data (zeros + random)

**Features**:
- Wear-leveling awareness
- Gentle write patterns with extended pauses
- Very conservative verification (10MB sample only)
- Support for different SD card grades (standard, high-endurance, industrial)

**Recommended Algorithms**:
1. Random (minimal wear, single pass)
2. Zeros (simple zero fill)
3. Quick Format (filesystem level)
4. File System Wipe (file-level secure deletion)
5. Two Pass (conservative 2-pass for critical data)

## Common Interface

All device erasers implement the `DeviceEraser` trait:

```rust
pub trait DeviceEraser {
    /// Analyze the device to determine optimal erasure method
    fn analyze_device(&self, device_path: &str) -> io::Result<DeviceInfo>;
    
    /// Perform device-specific erasure
    fn erase_device(
        &self,
        device_info: &DeviceInfo,
        algorithm: WipingAlgorithm,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()>;
    
    /// Verify erasure completion
    fn verify_erasure(&self, device_info: &DeviceInfo) -> io::Result<bool>;
    
    /// Get recommended algorithms for this device type
    fn get_recommended_algorithms(&self) -> Vec<WipingAlgorithm>;
}
```

## Device Factory

The `DeviceFactory` automatically:
1. Analyzes the device to determine its type
2. Creates the appropriate specialized eraser
3. Returns both device information and the optimized eraser

```rust
let (device_info, eraser) = DeviceFactory::analyze_and_create("/path/to/device")?;
let recommended_algorithms = eraser.get_recommended_algorithms();
```

## Integration with Main Application

The main application now uses device-specific erasure by default:

1. **Device Analysis**: Automatically detects device type and capabilities
2. **Algorithm Selection**: Uses recommended algorithms or falls back appropriately
3. **Progress Tracking**: Real-time progress reporting with device-specific metrics
4. **Fallback Support**: Graceful fallback to traditional methods if device-specific erasure fails

## Security Features

### Algorithm Matching
- Each device type has recommended algorithms based on its characteristics
- Automatic fallback to compatible algorithms if the selected one isn't optimal
- Security-appropriate defaults (e.g., single-pass for flash, multi-pass for magnetic)

### Verification
- Device-specific verification strategies
- Conservative sampling for flash devices to minimize wear
- Comprehensive verification for magnetic storage

### Progress Reporting
- Real-time progress updates with speed metrics
- Estimated time remaining calculations
- Pass-by-pass progress for multi-pass algorithms

## Usage Examples

### Basic Usage
```rust
// The application automatically detects device type and uses optimal methods
app.start_device_specific_sanitization(&drive_path, &drive_name, drive_index);
```

### Manual Device-Specific Usage
```rust
// Manual device-specific erasure
let (device_info, eraser) = devices::DeviceFactory::analyze_and_create(&device_path)?;
let recommended_algorithms = eraser.get_recommended_algorithms();
let algorithm = recommended_algorithms[0]; // Use first recommended
eraser.erase_device(&device_info, algorithm, progress_callback)?;
```

## Benefits

1. **Optimized Performance**: Each device type uses methods optimized for its characteristics
2. **Extended Lifespan**: Flash devices use wear-minimizing approaches
3. **Enhanced Security**: Magnetic drives use multi-pass methods when needed
4. **Hardware Acceleration**: Utilizes hardware-based erasure when available
5. **Intelligent Fallbacks**: Graceful degradation when optimal methods aren't available
6. **Real-time Feedback**: Device-specific progress reporting and time estimates

## Future Enhancements

- Support for additional device types (optical drives, tape drives)
- Hardware encryption detection and cryptographic erasure
- Advanced NVMe namespace management
- Integration with SMART data for device health monitoring
- Support for remote/network-attached storage devices

This architecture ensures that each storage device type receives the most appropriate erasure treatment while maintaining the flexibility to handle edge cases and provide comprehensive security coverage.