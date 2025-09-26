# HDD Tool Distribution Packages

This directory contains platform-specific distribution packages and installers.

## Package Types

### Windows
- **ZIP Archives**: Portable applications with installers
- **MSI Packages**: Windows Installer packages (future)
- **Chocolatey**: Package manager distribution (future)

### Linux
- **DEB Packages**: Debian/Ubuntu distribution
- **RPM Packages**: Red Hat/CentOS/Fedora distribution
- **AppImage**: Universal Linux application (future)
- **Snap Package**: Universal Linux package (future)

### macOS
- **DMG Images**: macOS disk images with applications
- **PKG Installers**: macOS installer packages
- **Homebrew**: Package manager distribution (future)

## Build Targets

### Supported Architectures
- **x86_64**: 64-bit Intel/AMD processors
- **i686**: 32-bit Intel/AMD processors (Windows only)
- **aarch64**: 64-bit ARM processors (Linux, macOS Apple Silicon)

### Platform Matrix
| Platform | x86_64 | i686 | aarch64 |
|----------|---------|------|---------|
| Windows  | ✅     | ✅   | ❌      |
| Linux    | ✅     | ❌   | ✅      |
| macOS    | ✅     | ❌   | ✅      |

## Installation Methods

### Automated Installers
Each package includes platform-specific installers:
- **Windows**: `install.bat` (requires Administrator)
- **Linux/macOS**: `install.sh` (requires root)

### Manual Installation
For advanced users, binaries can be extracted and placed manually:
1. Extract archive
2. Copy binaries to desired location
3. Set up PATH environment variable
4. Configure systemd service (Linux server)

## Security Considerations

### Code Signing
- **Windows**: Binaries should be code signed for production
- **macOS**: Requires Apple Developer certificate
- **Linux**: Package signing for repositories

### Permissions
- All platforms require elevated privileges for disk operations
- Server components need network binding permissions
- File system access for configuration and logs

## Distribution Channels

### Official Releases
- GitHub Releases (primary)
- Project website downloads
- Direct distribution links

### Package Managers (Future)
- **Windows**: Chocolatey, winget
- **Linux**: APT, YUM, Snap, Flatpak
- **macOS**: Homebrew, MacPorts

## Verification

### Checksums
All release packages include SHA256 checksums for verification:
```bash
# Verify package integrity
sha256sum -c checksums.txt
```

### GPG Signatures
Production releases include GPG signatures:
```bash
# Verify signature
gpg --verify package.tar.gz.sig package.tar.gz
```