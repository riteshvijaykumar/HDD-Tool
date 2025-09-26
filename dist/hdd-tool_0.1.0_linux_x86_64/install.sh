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