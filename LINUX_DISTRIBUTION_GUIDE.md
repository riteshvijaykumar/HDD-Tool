# HDD Tool - Linux Distribution Guide

## Overview
This guide explains how to create and distribute the HDD Tool for Linux systems.

## Distribution Package Structure

The Linux distribution package contains:

```
hdd-tool_0.1.0_linux_x86_64/
├── bin/
│   └── hdd_tool                 # Main executable (built on Linux)
├── share/
│   ├── applications/
│   │   └── hdd-tool.desktop     # Desktop entry file
│   └── pixmaps/
│       └── hdd-tool.png         # Application icon
├── usr/
│   └── share/
│       └── doc/
│           └── hdd-tool/
│               ├── LICENSE      # MIT License
│               └── README.md    # Documentation
├── install.sh                   # Installation script
├── uninstall.sh                 # Uninstallation script
└── README.txt                   # Package documentation
```

## Building for Linux Distribution

### Method 1: Build on Linux (Recommended)

1. **On a Linux system**, clone the repository:
   ```bash
   git clone https://github.com/riteshvijaykumar/HDD-Tool
   cd HDD-Tool
   ```

2. **Run the build script**:
   ```bash
   chmod +x build_linux.sh
   ./build_linux.sh
   ```

3. **Create distribution archive**:
   ```bash
   tar -czf hdd-tool_0.1.0_linux_x86_64.tar.gz -C dist hdd-tool_0.1.0_linux_x86_64
   ```

### Method 2: Cross-compilation (Advanced)

If building from Windows or macOS for Linux:

1. **Install cross-compilation target**:
   ```bash
   rustup target add x86_64-unknown-linux-gnu
   ```

2. **Install cross-compiler** (varies by platform):
   - Windows: Install WSL2 or use cross-compilation tools
   - macOS: Install appropriate cross-compiler

3. **Build for Linux**:
   ```bash
   cargo build --release --target x86_64-unknown-linux-gnu
   ```

4. **Copy binary to distribution**:
   ```bash
   cp target/x86_64-unknown-linux-gnu/release/hdd_tool dist/hdd-tool_0.1.0_linux_x86_64/bin/
   ```

## Distribution Formats

### 1. Tarball Distribution (.tar.gz)

**Create:**
```bash
tar -czf hdd-tool_0.1.0_linux_x86_64.tar.gz -C dist hdd-tool_0.1.0_linux_x86_64
```

**Install:**
```bash
tar -xzf hdd-tool_0.1.0_linux_x86_64.tar.gz
cd hdd-tool_0.1.0_linux_x86_64
sudo ./install.sh
```

### 2. Debian Package (.deb)

To create a proper `.deb` package:

1. **Install build tools**:
   ```bash
   sudo apt install dpkg-dev debhelper
   ```

2. **Create debian directory structure**:
   ```bash
   mkdir -p debian/DEBIAN
   mkdir -p debian/usr/local/bin
   mkdir -p debian/usr/share/applications
   mkdir -p debian/usr/share/pixmaps
   mkdir -p debian/usr/share/doc/hdd-tool
   ```

3. **Create control file**:
   ```bash
   cat > debian/DEBIAN/control << EOF
   Package: hdd-tool
   Version: 0.1.0
   Section: utils
   Priority: optional
   Architecture: amd64
   Depends: hdparm, util-linux
   Maintainer: SIH Team <team@example.com>
   Description: NIST SP 800-88 compliant disk sanitization tool
    HDD Tool is a cross-platform disk sanitization utility that provides
    NIST SP 800-88 compliant data erasure for various storage devices.
   EOF
   ```

4. **Copy files and build**:
   ```bash
   cp target/release/hdd_tool debian/usr/local/bin/
   cp hdd-tool.desktop debian/usr/share/applications/
   cp resources/logo.png debian/usr/share/pixmaps/hdd-tool.png
   cp LICENSE CROSS_PLATFORM_README.md debian/usr/share/doc/hdd-tool/
   dpkg-deb --build debian hdd-tool_0.1.0_amd64.deb
   ```

### 3. RPM Package (.rpm)

For Red Hat-based distributions:

1. **Install build tools**:
   ```bash
   sudo dnf install rpm-build rpmlint
   ```

2. **Create RPM spec file** and build using `rpmbuild`

### 4. Snap Package

For universal Linux distribution:

1. **Create snapcraft.yaml**
2. **Build with snapcraft**
3. **Publish to Snap Store**

## Installation Methods

### Automatic Installation (Recommended)
```bash
# Extract and run installer
tar -xzf hdd-tool_0.1.0_linux_x86_64.tar.gz
cd hdd-tool_0.1.0_linux_x86_64
sudo ./install.sh
```

### Manual Installation
```bash
# Copy files manually
sudo cp bin/hdd_tool /usr/local/bin/
sudo cp share/applications/hdd-tool.desktop /usr/share/applications/
sudo cp share/pixmaps/hdd-tool.png /usr/share/pixmaps/
sudo cp -r usr/share/doc/hdd-tool /usr/share/doc/

# Install dependencies
sudo apt install hdparm util-linux  # Ubuntu/Debian
sudo dnf install hdparm util-linux  # Fedora
sudo yum install hdparm util-linux  # RHEL/CentOS
```

## System Requirements

### Runtime Requirements
- Linux x86_64 system
- glibc 2.31+ (Ubuntu 20.04+, similar for other distros)
- hdparm package (for SSD secure erase)
- util-linux package (for disk utilities)
- Administrative privileges (root/sudo)

### Build Requirements
- Rust 1.70+ (latest stable recommended)
- Cargo (included with Rust)
- build-essential or equivalent
- pkg-config
- Linux development headers

## Testing Distribution

### Test Installation
```bash
# Test in clean environment (Docker recommended)
docker run -it --privileged ubuntu:22.04
# Install and test the package
```

### Test Functionality
```bash
# Basic functionality test
sudo hdd_tool --help
# GUI test (requires X11 forwarding or local display)
sudo hdd_tool
```

## Distribution Checklist

- [ ] Binary built on target Linux version
- [ ] All dependencies included in package metadata
- [ ] Desktop integration files included
- [ ] Documentation complete and accurate
- [ ] License file included
- [ ] Installation/uninstallation scripts tested
- [ ] Package tested on target distributions
- [ ] File permissions correctly set
- [ ] Digital signatures (optional but recommended)

## Publishing Options

### GitHub Releases
1. Create release tag: `v0.1.0`
2. Upload tarball as release asset
3. Include release notes

### Package Repositories
1. **Ubuntu PPA**: Create Personal Package Archive
2. **AUR (Arch)**: Submit to Arch User Repository
3. **Fedora COPR**: Community build system
4. **openSUSE Build Service**: Multi-distribution builds

### Direct Distribution
1. Host on project website
2. Provide installation instructions
3. Include checksum verification

## Security Considerations

### Code Signing
```bash
# Sign the binary (optional)
gpg --detach-sign --armor hdd_tool
```

### Checksums
```bash
# Create checksums file
sha256sum hdd-tool_0.1.0_linux_x86_64.tar.gz > hdd-tool_0.1.0_linux_x86_64.tar.gz.sha256
```

### Repository Security
- Use HTTPS for downloads
- Provide GPG signatures
- Include integrity verification instructions

## Support and Maintenance

### Version Updates
1. Update version in Cargo.toml
2. Update package metadata
3. Rebuild and repackage
4. Test on supported distributions
5. Publish new release

### Distribution Support Matrix
- Ubuntu 20.04+ LTS
- Debian 11+
- Fedora 35+
- CentOS Stream 8+
- openSUSE Leap 15.3+
- Arch Linux (rolling)

---

**Note**: This distribution package structure provides a professional, maintainable approach to Linux software distribution while following standard Linux packaging conventions.