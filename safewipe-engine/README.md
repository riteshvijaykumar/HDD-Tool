# SafeWipe Engine

## Overview
SafeWipe is a NIST SP 800-88 compliant data sanitization engine for Linux environments. It supports secure erasure and cryptographic operations on HDDs, SSDs, and removable media. The engine uses real system commands for destructive operations, making it suitable for production and virtualized testing (e.g., VirtualBox).

## Features
- **Device Scanning:** Detects all storage devices and their capabilities.
- **Sanitization Methods:**
  - Clear (overwrite with patterns)
  - Purge (secure erase, crypto erase, vendor-specific purge)
  - Destroy (physical destruction instructions)
- **Real Device Operations:**
  - ATA Secure Erase / Enhanced Secure Erase (via `hdparm`)
  - NVMe Sanitize (via `nvme-cli`)
  - Vendor-specific purge (via `blkdiscard`)
  - Enable device encryption (via `hdparm`)
  - Generate new encryption key (via `hdparm`)
  - Secure factory reset (via `blkdiscard`)
- **Verification:** Post-wipe verification for clear operations.
- **Safety:** Refuses to wipe system drives. Requires explicit confirmation for destructive actions.
- **CLI and Extensible API:** Command-line interface and modular Rust API.

## Usage

### Prerequisites
- Linux environment (tested on Ubuntu, Fedora, etc.)
- Root privileges for destructive operations
- Utilities: `hdparm`, `blkdiscard`, `nvme-cli` (install via your package manager)

### Build
```
cargo build --release
```

### Run (CLI)
```
sudo ./target/release/safewipe-engine sanitize --method purge --devices sda,sdb --real-devices
```

### Example Commands
- **Scan devices:**
  ```
  ./safewipe-engine scan
  ```
- **Sanitize all non-system devices (purge):**
  ```
  sudo ./safewipe-engine sanitize --method purge --all --real-devices
  ```
- **Legacy wipe command:**
  ```
  sudo ./safewipe-engine wipe --device sda --method clear
  ```

## Supported Operations
| Method                        | Command Used                | Device Type      | Notes                        |
|-------------------------------|-----------------------------|------------------|------------------------------|
| ATA Secure Erase              | hdparm --security-erase     | HDD/SSD (ATA)    | Sets password, erases device |
| ATA Enhanced Secure Erase     | hdparm --security-erase-enhanced | HDD/SSD (ATA) | More thorough erase          |
| NVMe Sanitize                 | nvme sanitize               | SSD (NVMe)       | Requires nvme-cli            |
| Vendor-Specific Purge         | blkdiscard                  | SSD/Removable    | Discards all blocks          |
| Enable Device Encryption      | hdparm --security-set-pass  | HDD/SSD (ATA)    | Enables hardware encryption  |
| Generate New Encryption Key   | hdparm --security-set-pass  | HDD/SSD (ATA)    | Rotates encryption key       |
| Secure Factory Reset          | blkdiscard                  | SSD/Removable    | Secure reset                 |

## Safety Notes
- **Destructive Operations:** These methods will irreversibly erase all data on the target device. Use with caution.
- **Testing:** For safe testing, use virtual disks in VirtualBox or similar environments.
- **System Drives:** The engine will refuse to sanitize system drives for safety.

## Extending
- Add support for more vendor-specific tools by extending `execute_vendor_specific_purge` in `wipe.rs`.
- For Windows/macOS support, adapt system commands and error handling.

## References
- [NIST SP 800-88 Guidelines](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-88r1.pdf)
- [hdparm Documentation](https://man7.org/linux/man-pages/man8/hdparm.8.html)
- [blkdiscard Documentation](https://man7.org/linux/man-pages/man8/blkdiscard.8.html)
- [nvme-cli Documentation](https://github.com/linux-nvme/nvme-cli)

## License
MIT

