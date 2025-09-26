#!/bin/bash

# Cross-platform build script for HDD Tool
# Usage: ./build-release.sh [version]

set -e

VERSION=${1:-"v0.1.0"}
echo "Building HDD Tool release version: $VERSION"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    if ! command -v cross &> /dev/null; then
        print_warning "Cross is not installed. Installing..."
        cargo install cross
    fi
    
    print_success "Dependencies check completed"
}

# Install cross-compilation targets
install_targets() {
    print_status "Installing cross-compilation targets..."
    
    # Windows targets
    rustup target add x86_64-pc-windows-msvc
    rustup target add i686-pc-windows-msvc
    
    # Linux targets
    rustup target add x86_64-unknown-linux-gnu
    rustup target add aarch64-unknown-linux-gnu
    
    # macOS targets
    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin
    
    print_success "Cross-compilation targets installed"
}

# Build for a specific target
build_target() {
    local target=$1
    local os_name=$2
    local arch=$3
    
    print_status "Building for $target ($os_name-$arch)..."
    
    # Create release directory
    local release_dir="releases/$VERSION/$os_name-$arch"
    mkdir -p "$release_dir"
    
    # Build desktop application
    if [[ "$target" == *"windows"* ]]; then
        cargo build --release --target "$target" --bin hdd-tool
        cargo build --release --target "$target" --bin hdd-tool-server --features server
        
        # Copy binaries
        cp "target/$target/release/hdd-tool.exe" "$release_dir/"
        cp "target/$target/release/hdd-tool-server.exe" "$release_dir/"
        
        # Create Windows installer
        create_windows_installer "$release_dir"
        
        # Create zip archive
        cd "$release_dir"
        zip -r "../hdd-tool-$os_name-$arch.zip" *
        cd - > /dev/null
        
    else
        cargo build --release --target "$target" --bin hdd-tool
        cargo build --release --target "$target" --bin hdd-tool-server --features server
        
        # Copy binaries
        cp "target/$target/release/hdd-tool" "$release_dir/"
        cp "target/$target/release/hdd-tool-server" "$release_dir/"
        
        # Create Unix installer
        create_unix_installer "$release_dir"
        
        # Create tar.gz archive
        cd "$release_dir"
        tar -czf "../hdd-tool-$os_name-$arch.tar.gz" *
        cd - > /dev/null
    fi
    
    # Copy resources
    if [ -d "resources" ]; then
        cp -r resources "$release_dir/"
    fi
    
    if [ -d "reference" ]; then
        cp -r reference "$release_dir/"
    fi
    
    cp README.md "$release_dir/"
    
    print_success "Build completed for $target"
}

create_windows_installer() {
    local release_dir=$1
    
    cat > "$release_dir/install.bat" << 'EOF'
@echo off
echo Installing HDD Tool...

REM Check for administrator privileges
net session >nul 2>&1
if %errorLevel% == 0 (
    echo Administrator privileges detected.
) else (
    echo This installer requires administrator privileges.
    echo Please run as administrator.
    pause
    exit /b 1
)

REM Create program directory
if not exist "%PROGRAMFILES%\HDD Tool\" (
    mkdir "%PROGRAMFILES%\HDD Tool\"
)

REM Copy binaries
copy "hdd-tool.exe" "%PROGRAMFILES%\HDD Tool\"
copy "hdd-tool-server.exe" "%PROGRAMFILES%\HDD Tool\"

REM Copy resources
if exist "resources" (
    xcopy /E /I "resources" "%PROGRAMFILES%\HDD Tool\resources\"
)
if exist "reference" (
    xcopy /E /I "reference" "%PROGRAMFILES%\HDD Tool\reference\"
)

REM Add to PATH
setx /M PATH "%PATH%;%PROGRAMFILES%\HDD Tool"

REM Create desktop shortcut
set "desktop=%USERPROFILE%\Desktop"
set "shortcut=%desktop%\HDD Tool.lnk"
powershell -Command "$WshShell = New-Object -comObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%shortcut%'); $Shortcut.TargetPath = '%PROGRAMFILES%\HDD Tool\hdd-tool.exe'; $Shortcut.Save()"

echo.
echo Installation complete!
echo.
echo You can now:
echo - Run 'hdd-tool' from Command Prompt
echo - Use the desktop shortcut
echo - Find HDD Tool in the Start Menu
echo.
pause
EOF
}

create_unix_installer() {
    local release_dir=$1
    
    cat > "$release_dir/install.sh" << 'EOF'
#!/bin/bash

set -e

echo "Installing HDD Tool..."

# Check for root privileges
if [[ $EUID -ne 0 ]]; then
   echo "This installer requires root privileges. Please run with sudo."
   exit 1
fi

# Create installation directory
mkdir -p /opt/hdd-tool
mkdir -p /usr/local/bin

# Copy binaries
cp hdd-tool /opt/hdd-tool/
cp hdd-tool-server /opt/hdd-tool/
chmod +x /opt/hdd-tool/hdd-tool
chmod +x /opt/hdd-tool/hdd-tool-server

# Copy resources
if [ -d "resources" ]; then
    cp -r resources /opt/hdd-tool/
fi
if [ -d "reference" ]; then
    cp -r reference /opt/hdd-tool/
fi

# Create symbolic links
ln -sf /opt/hdd-tool/hdd-tool /usr/local/bin/hdd-tool
ln -sf /opt/hdd-tool/hdd-tool-server /usr/local/bin/hdd-tool-server

# Create desktop entry for Linux
if command -v desktop-file-install >/dev/null 2>&1; then
    cat > /tmp/hdd-tool.desktop << 'DESKTOP'
[Desktop Entry]
Version=1.0
Type=Application
Name=HDD Tool
Comment=NIST SP 800-88 compliant disk sanitization tool
Exec=/usr/local/bin/hdd-tool
Icon=/opt/hdd-tool/resources/logo.png
Terminal=false
Categories=System;Utility;
DESKTOP
    
    desktop-file-install /tmp/hdd-tool.desktop
    rm /tmp/hdd-tool.desktop
fi

# Create systemd service for server
cat > /etc/systemd/system/hdd-tool-server.service << 'SERVICE'
[Unit]
Description=HDD Tool Server
After=network.target

[Service]
Type=simple
User=hdd-tool
WorkingDirectory=/opt/hdd-tool
ExecStart=/opt/hdd-tool/hdd-tool-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
SERVICE

# Create hdd-tool user
if ! id "hdd-tool" &>/dev/null; then
    useradd -r -s /bin/false -d /opt/hdd-tool hdd-tool
fi

chown -R hdd-tool:hdd-tool /opt/hdd-tool

systemctl daemon-reload

echo ""
echo "Installation complete!"
echo ""
echo "You can now:"
echo "- Run 'hdd-tool' to start the desktop application"
echo "- Run 'sudo systemctl start hdd-tool-server' to start the server"
echo "- Run 'sudo systemctl enable hdd-tool-server' to enable auto-start"
echo ""
EOF
    
    chmod +x "$release_dir/install.sh"
}

# Create release notes
create_release_notes() {
    print_status "Creating release notes..."
    
    cat > "releases/$VERSION/RELEASE_NOTES.md" << EOF
# HDD Tool $VERSION Release Notes

## Overview
HDD Tool is a cross-platform disk sanitization utility that provides NIST SP 800-88 compliant data erasure for various storage devices.

## Features
- **NIST SP 800-88 Compliance**: Implements approved sanitization methods
- **Multi-Device Support**: HDDs, SSDs, NVMe drives, USB storage, SD cards
- **Cross-Platform**: Windows, Linux, macOS support
- **Authentication System**: User management with role-based access
- **Server Backend**: PostgreSQL database with REST API
- **Web Dashboard**: Browser-based management interface
- **Audit Trail**: Comprehensive logging and reporting

## Platform Support

### Windows (x64, x86)
- **Requirements**: Windows 10/11, Administrator privileges
- **Installation**: Extract and run \`install.bat\` as Administrator
- **Features**: Full disk access, SMART monitoring, ATA command support

### Linux (x64, ARM64)
- **Requirements**: Modern Linux distribution, hdparm, util-linux
- **Installation**: Extract and run \`sudo bash install.sh\`
- **Features**: Direct device access, systemd service, desktop integration

### macOS (Intel, Apple Silicon)
- **Requirements**: macOS 10.15+, Administrator privileges
- **Installation**: Extract and run \`sudo bash install.sh\`
- **Features**: Core Storage support, disk utility integration

## Installation Instructions

### Quick Start
1. Download the appropriate package for your platform
2. Extract the archive
3. Run the installer script with administrator/root privileges
4. Launch HDD Tool

### Server Setup
For server functionality:
1. Install PostgreSQL
2. Configure environment variables
3. Run \`hdd-tool-server\`
4. Access web dashboard at http://localhost:3030

## Security Notes
- Always run with appropriate privileges for disk access
- Server communications use HTTPS in production
- User credentials are securely hashed
- Audit logs are tamper-evident

## Support
- Documentation: See included reference materials
- Issues: Report on GitHub repository
- Security: Follow responsible disclosure practices

## Changelog
- Initial cross-platform release
- Complete NIST SP 800-88 implementation
- Web-based management interface
- Multi-architecture support
EOF

    print_success "Release notes created"
}

# Main build function
main() {
    print_status "Starting cross-platform build process"
    
    # Clean previous builds
    rm -rf releases/$VERSION
    mkdir -p releases/$VERSION
    
    check_dependencies
    install_targets
    
    # Build for all platforms
    print_status "Building for all platforms..."
    
    # Windows builds
    build_target "x86_64-pc-windows-msvc" "windows" "x64"
    build_target "i686-pc-windows-msvc" "windows" "x86"
    
    # Linux builds
    build_target "x86_64-unknown-linux-gnu" "linux" "x64"
    build_target "aarch64-unknown-linux-gnu" "linux" "arm64"
    
    # macOS builds (requires macOS host for proper signing)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        build_target "x86_64-apple-darwin" "macos" "x64"
        build_target "aarch64-apple-darwin" "macos" "arm64"
    else
        print_warning "Skipping macOS builds (requires macOS host for proper compilation)"
    fi
    
    create_release_notes
    
    print_success "Cross-platform build completed!"
    print_status "Release artifacts available in: releases/$VERSION/"
    
    echo ""
    echo "Built packages:"
    ls -la releases/$VERSION/*.{zip,tar.gz} 2>/dev/null || true
}

# Run main function
main "$@"