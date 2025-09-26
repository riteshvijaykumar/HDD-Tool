#!/bin/bash

# HDD Tool Linux Distribution Builder
# This script creates a Linux distribution package

# Set variables
APP_NAME="hdd-tool"
VERSION="0.1.0"
PACKAGE_NAME="${APP_NAME}_${VERSION}_linux_x86_64"
DIST_DIR="dist"
PACKAGE_DIR="${DIST_DIR}/${PACKAGE_NAME}"

# Create directory structure
echo "Creating distribution package for ${APP_NAME} v${VERSION}..."
mkdir -p "${PACKAGE_DIR}/bin"
mkdir -p "${PACKAGE_DIR}/share/applications"
mkdir -p "${PACKAGE_DIR}/share/pixmaps"
mkdir -p "${PACKAGE_DIR}/usr/share/doc/${APP_NAME}"

# Copy binary (will be the Windows exe for now, but script shows structure)
# On actual Linux build: cp target/release/hdd_tool "${PACKAGE_DIR}/bin/"
echo "Note: On Linux, run 'cargo build --release' and copy target/release/hdd_tool to ${PACKAGE_DIR}/bin/"

# Copy desktop entry
cp hdd-tool.desktop "${PACKAGE_DIR}/share/applications/"

# Copy icon
cp resources/logo.png "${PACKAGE_DIR}/share/pixmaps/hdd-tool.png"

# Copy documentation
cp LICENSE "${PACKAGE_DIR}/usr/share/doc/${APP_NAME}/"
cp CROSS_PLATFORM_README.md "${PACKAGE_DIR}/usr/share/doc/${APP_NAME}/README.md"

# Create install script
cat > "${PACKAGE_DIR}/install.sh" << 'EOF'
#!/bin/bash

# HDD Tool Installation Script
set -e

APP_NAME="hdd-tool"
INSTALL_PREFIX="/usr/local"

echo "Installing HDD Tool..."

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root (use sudo)" 
   exit 1
fi

# Install dependencies
echo "Installing dependencies..."
if command -v apt-get &> /dev/null; then
    # Debian/Ubuntu
    apt-get update
    apt-get install -y hdparm util-linux
elif command -v yum &> /dev/null; then
    # RHEL/CentOS
    yum install -y hdparm util-linux
elif command -v dnf &> /dev/null; then
    # Fedora
    dnf install -y hdparm util-linux
elif command -v zypper &> /dev/null; then
    # openSUSE
    zypper install -y hdparm util-linux
else
    echo "Warning: Could not detect package manager. Please install hdparm and util-linux manually."
fi

# Copy files
echo "Installing files..."
cp bin/hdd_tool ${INSTALL_PREFIX}/bin/
chmod +x ${INSTALL_PREFIX}/bin/hdd_tool

cp share/applications/hdd-tool.desktop /usr/share/applications/
cp share/pixmaps/hdd-tool.png /usr/share/pixmaps/
cp -r usr/share/doc/${APP_NAME} /usr/share/doc/

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications
fi

echo "HDD Tool installed successfully!"
echo "You can now run 'hdd_tool' from the command line or find it in your applications menu."
echo ""
echo "IMPORTANT: This tool requires administrative privileges to access storage devices."
echo "Run with: sudo hdd_tool"
EOF

chmod +x "${PACKAGE_DIR}/install.sh"

# Create uninstall script
cat > "${PACKAGE_DIR}/uninstall.sh" << 'EOF'
#!/bin/bash

# HDD Tool Uninstallation Script
set -e

APP_NAME="hdd-tool"
INSTALL_PREFIX="/usr/local"

echo "Uninstalling HDD Tool..."

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root (use sudo)" 
   exit 1
fi

# Remove files
rm -f ${INSTALL_PREFIX}/bin/hdd_tool
rm -f /usr/share/applications/hdd-tool.desktop
rm -f /usr/share/pixmaps/hdd-tool.png
rm -rf /usr/share/doc/${APP_NAME}

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications
fi

echo "HDD Tool uninstalled successfully!"
EOF

chmod +x "${PACKAGE_DIR}/uninstall.sh"

# Create README for the package
cat > "${PACKAGE_DIR}/README.txt" << EOF
HDD Tool v${VERSION} - Linux Distribution Package
===============================================

This package contains the HDD Tool, a NIST SP 800-88 compliant disk sanitization utility
for Linux systems.

INSTALLATION:
1. Extract this package to a temporary directory
2. Run: sudo ./install.sh

MANUAL INSTALLATION:
1. Copy bin/hdd_tool to /usr/local/bin/
2. Copy share/applications/hdd-tool.desktop to /usr/share/applications/
3. Copy share/pixmaps/hdd-tool.png to /usr/share/pixmaps/
4. Install dependencies: hdparm, util-linux

USAGE:
- Run from command line: sudo hdd_tool
- Or find in applications menu under "System Tools"

REQUIREMENTS:
- Linux x86_64 system
- Administrative privileges (root/sudo)
- hdparm package for SSD secure erase
- util-linux package for disk utilities

UNINSTALLATION:
Run: sudo ./uninstall.sh

For more information, see usr/share/doc/${APP_NAME}/README.md

Copyright © 2025 SIH Team
Licensed under MIT License
EOF

echo "Distribution package created in ${PACKAGE_DIR}/"
echo ""
echo "To complete the Linux package:"
echo "1. Build on Linux: cargo build --release"
echo "2. Copy the Linux binary: cp target/release/hdd_tool ${PACKAGE_DIR}/bin/"
echo "3. Create archive: tar -czf ${PACKAGE_NAME}.tar.gz -C ${DIST_DIR} ${PACKAGE_NAME}"
echo ""
echo "Users can then extract and run ./install.sh to install the application."