# Cross-Platform Release Management

This directory contains scripts and configurations for building and releasing HDD Tool across multiple platforms.

## Scripts

### `build-release.sh` (Linux/macOS)
Cross-platform build script that creates release packages for:
- Windows (x64, x86)
- Linux (x64, ARM64)
- macOS (Intel, Apple Silicon)

**Usage:**
```bash
chmod +x build-release.sh
./build-release.sh [version]
```

### `build-release.bat` (Windows)
Windows-specific build script for creating Windows releases:
- Windows x64
- Windows x86

**Usage:**
```cmd
build-release.bat [version]
```

## GitHub Actions

### `release.yml`
Automated cross-platform release workflow that:
- Builds for all supported platforms
- Creates installers for each platform
- Generates release archives
- Creates GitHub releases with assets
- Supports both tag-triggered and manual releases

### `ci.yml`
Continuous integration workflow that:
- Runs tests on all platforms
- Performs linting and formatting checks
- Validates builds across Windows, Linux, and macOS

## Release Process

### Automated Release (Recommended)
1. Create and push a git tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```
2. GitHub Actions automatically builds and creates the release

### Manual Release
1. Use GitHub Actions workflow dispatch:
   - Go to Actions tab in GitHub
   - Select "Cross-Platform Release"
   - Click "Run workflow"
   - Enter version (e.g., v1.0.0)

### Local Development Build
1. Run the appropriate build script:
   ```bash
   # Linux/macOS
   ./scripts/build-release.sh v1.0.0-dev
   
   # Windows
   scripts\build-release.bat v1.0.0-dev
   ```

## Platform-Specific Notes

### Windows
- Requires Administrator privileges for installation
- Creates desktop shortcuts and Start Menu entries
- Adds HDD Tool to system PATH
- Supports both x64 and x86 architectures

### Linux
- Requires root privileges for installation
- Creates systemd service for server component
- Installs desktop entry for GUI applications
- Supports x64 and ARM64 architectures

### macOS
- Requires Administrator privileges for installation
- Supports both Intel and Apple Silicon
- Integrates with macOS security framework
- May require code signing for distribution

## Dependencies

### Build Dependencies
- Rust toolchain with cross-compilation targets
- Platform-specific system libraries
- Cross compilation tools (cross crate)

### Runtime Dependencies
- **Windows**: Windows 10/11, Administrator privileges
- **Linux**: hdparm, util-linux, modern Linux distribution
- **macOS**: macOS 10.15+, Administrator privileges

## Archive Formats

- **Windows**: ZIP archives with batch installers
- **Linux/macOS**: tar.gz archives with shell installers

## Security Considerations

- All installers require elevated privileges
- Server components include security configuration
- Binaries should be code signed for production releases
- HTTPS configuration for server deployments