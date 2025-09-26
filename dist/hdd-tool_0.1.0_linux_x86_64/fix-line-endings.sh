#!/bin/bash

# HDD Tool - Line Ending Fix Script
# Run this script to fix Windows line endings in the executables

echo "🔧 Fixing line endings for HDD Tool executables..."

# Check if dos2unix is available
if command -v dos2unix &> /dev/null; then
    echo "Using dos2unix to fix line endings..."
    dos2unix hdd-tool 2>/dev/null || true
    dos2unix run-linux.sh 2>/dev/null || true
    dos2unix hdd-tool-linux 2>/dev/null || true
    dos2unix install.sh 2>/dev/null || true
    dos2unix uninstall.sh 2>/dev/null || true
else
    echo "Using sed to fix line endings..."
    sed -i 's/\r$//' hdd-tool 2>/dev/null || true
    sed -i 's/\r$//' run-linux.sh 2>/dev/null || true
    sed -i 's/\r$//' hdd-tool-linux 2>/dev/null || true
    sed -i 's/\r$//' install.sh 2>/dev/null || true
    sed -i 's/\r$//' uninstall.sh 2>/dev/null || true
fi

# Make all scripts executable
echo "Setting executable permissions..."
chmod +x hdd-tool 2>/dev/null || true
chmod +x run-linux.sh 2>/dev/null || true
chmod +x hdd-tool-linux 2>/dev/null || true
chmod +x install.sh 2>/dev/null || true
chmod +x uninstall.sh 2>/dev/null || true

echo "✅ Line endings fixed and permissions set!"
echo ""
echo "You can now run:"
echo "  ./hdd-tool              # Quick launcher"
echo "  ./run-linux.sh          # Standard launcher"
echo "  ./hdd-tool-linux run    # Advanced launcher"
echo "  sudo ./install.sh       # System installation"
echo ""
echo "🚀 HDD Tool is ready to use!"