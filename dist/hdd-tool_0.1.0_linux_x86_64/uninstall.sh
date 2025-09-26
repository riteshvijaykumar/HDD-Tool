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