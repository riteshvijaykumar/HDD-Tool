# HDD Tool - Linux Executable Guide

## Available Linux Executables

This project provides multiple ways to run the HDD Tool on Linux systems:

### 1. `hdd-tool` - Quick Launcher (Recommended)

The simplest way to run the application:

```bash
# Make executable (first time only)
chmod +x hdd-tool

# Run the application
./hdd-tool
```

**Features:**
- ✅ Automatically builds if needed
- ✅ Handles root privileges automatically
- ✅ Minimal setup required
- ✅ Perfect for quick usage

### 2. `run-linux.sh` - Standard Launcher

More detailed launcher with better error handling:

```bash
# Make executable
chmod +x run-linux.sh

# Run the application
./run-linux.sh
```

**Features:**
- ✅ Detailed dependency checking
- ✅ Better error messages
- ✅ System integration checks
- ✅ Suitable for regular use

### 3. `hdd-tool-linux` - Full-Featured Launcher

Complete launcher with system installation capabilities:

```bash
# Make executable
chmod +x hdd-tool-linux

# Show all options
./hdd-tool-linux help

# Run the application
./hdd-tool-linux run

# Install system-wide
sudo ./hdd-tool-linux install

# Check dependencies
./hdd-tool-linux check
```

**Available Commands:**
- `run, start` - Run the application (default)
- `build, compile` - Build from source  
- `install` - Install system-wide (requires sudo)
- `uninstall` - Remove system installation (requires sudo)
- `check, deps` - Check system dependencies
- `clean` - Clean build artifacts
- `help` - Show help message

## Quick Start Guide

### Option A: Direct Execution (Easiest)
```bash
# 1. Clone or download the project
git clone https://github.com/riteshvijaykumar/HDD-Tool
cd HDD-Tool

# 2. Run directly
chmod +x hdd-tool
./hdd-tool
```

### Option B: Build and Run
```bash
# 1. Ensure Rust is installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Install system dependencies
sudo apt install hdparm util-linux  # Ubuntu/Debian
# or
sudo dnf install hdparm util-linux  # Fedora

# 3. Build and run
cargo build --release
sudo ./target/release/hdd_tool
```

### Option C: System Installation
```bash
# 1. Install system-wide
chmod +x hdd-tool-linux
sudo ./hdd-tool-linux install

# 2. Run from anywhere
sudo hdd_tool
# or find in applications menu
```

## System Requirements

### Runtime Requirements
- Linux x86_64 system
- Root/sudo privileges (for disk access)
- hdparm package (for SSD secure erase)
- util-linux package (for basic disk utilities)

### Build Requirements (if building from source)
- Rust 1.70+ with Cargo
- GCC or Clang compiler
- pkg-config
- Development libraries

### Install Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install hdparm util-linux build-essential pkg-config
```

**Fedora:**
```bash
sudo dnf install hdparm util-linux gcc pkg-config
```

**CentOS/RHEL:**
```bash
sudo yum install hdparm util-linux gcc pkg-config
```

**Arch Linux:**
```bash
sudo pacman -S hdparm util-linux gcc pkgconf
```

## Usage Examples

### Basic Usage
```bash
# Quick start - builds and runs automatically
./hdd-tool

# Standard launcher with more options
./run-linux.sh

# Full launcher with command options
./hdd-tool-linux run
```

### Advanced Usage
```bash
# Check system compatibility
./hdd-tool-linux check

# Clean rebuild
./hdd-tool-linux clean
./hdd-tool-linux build

# Install for all users
sudo ./hdd-tool-linux install

# Remove installation
sudo ./hdd-tool-linux uninstall
```

### Development Usage
```bash
# Build manually
cargo build --release

# Run with debug info
cargo run

# Run tests
cargo test

# Check for issues
cargo clippy
```

## Troubleshooting

### Common Issues

**1. Permission Denied**
```bash
# Make script executable
chmod +x hdd-tool

# Run with sudo for disk access
sudo ./hdd-tool
```

**2. Rust Not Found**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**3. Missing Dependencies**
```bash
# Check what's missing
./hdd-tool-linux check

# Install on Ubuntu/Debian
sudo apt install hdparm util-linux build-essential

# Install on Fedora
sudo dnf install hdparm util-linux gcc
```

**4. Build Failures**
```bash
# Clean and rebuild
./hdd-tool-linux clean
./hdd-tool-linux build

# Or manually
cargo clean
cargo build --release
```

**5. GUI Not Starting**
```bash
# Check display environment
echo $DISPLAY

# For SSH users, enable X11 forwarding
ssh -X user@host

# Or use X11 forwarding
export DISPLAY=:0
```

## Security Considerations

### Important Warnings
- ⚠️ **Root Required**: Tool needs sudo/root for disk access
- ⚠️ **Data Destruction**: Operations are irreversible
- ⚠️ **Device Verification**: Always verify target devices
- ⚠️ **Backup Data**: Maintain backups before sanitization

### Safe Usage Practices
1. **Always verify target device** before starting
2. **Disconnect non-target drives** when possible
3. **Use test mode first** if available
4. **Keep backups** of important data
5. **Document operations** for audit trails

## File Structure

```
HDD-Tool/
├── hdd-tool              # Quick launcher (recommended)
├── run-linux.sh          # Standard launcher  
├── hdd-tool-linux        # Full-featured launcher
├── src/                  # Source code
├── target/release/       # Built binaries (after build)
│   └── hdd_tool         # Main executable
├── Cargo.toml           # Rust project configuration
└── dist/                # Distribution packages
```

## Support

### Getting Help
- **Documentation**: See `CROSS_PLATFORM_README.md`
- **Issues**: Report at GitHub repository
- **Build Problems**: Check system dependencies
- **Runtime Issues**: Verify root privileges

### Command Help
```bash
# Show launcher help
./hdd-tool-linux help

# Show application help (after running)
sudo ./hdd-tool --help
```

---

**Quick Start Summary:**
1. `chmod +x hdd-tool`
2. `./hdd-tool`
3. Follow on-screen instructions

That's it! The executable handles the rest automatically. 🚀