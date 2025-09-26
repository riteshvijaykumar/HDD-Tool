#!/bin/bash

# HDD Tool Linux Build Script
# This script builds the application on Linux and completes the distribution package

set -e

echo "HDD Tool Linux Build Script"
echo "============================"

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "Warning: This script is designed for Linux systems."
    echo "Current OS: $OSTYPE"
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "Error: Cargo.toml not found. Please run this script from the project root directory."
    exit 1
fi

# Install Linux dependencies (optional)
echo "Checking for Linux dependencies..."
if command -v apt-get &> /dev/null; then
    echo "Installing build dependencies (Ubuntu/Debian)..."
    sudo apt-get update
    sudo apt-get install -y hdparm util-linux build-essential pkg-config
elif command -v dnf &> /dev/null; then
    echo "Installing build dependencies (Fedora)..."
    sudo dnf install -y hdparm util-linux gcc pkg-config
elif command -v yum &> /dev/null; then
    echo "Installing build dependencies (RHEL/CentOS)..."
    sudo yum install -y hdparm util-linux gcc pkg-config
else
    echo "Warning: Could not detect package manager. Make sure you have:"
    echo "- hdparm, util-linux, build-essential, pkg-config"
fi

# Build the application
echo ""
echo "Building HDD Tool for Linux..."
cargo build --release

# Check if build was successful
if [[ ! -f "target/release/hdd_tool" ]]; then
    echo "Error: Build failed. Binary not found at target/release/hdd_tool"
    exit 1
fi

# Copy binary to distribution directory
DIST_DIR="dist/hdd-tool_0.1.0_linux_x86_64"
if [[ -d "$DIST_DIR" ]]; then
    echo "Copying binary to distribution directory..."
    cp target/release/hdd_tool "$DIST_DIR/bin/"
    chmod +x "$DIST_DIR/bin/hdd_tool"
    
    # Make scripts executable
    chmod +x "$DIST_DIR/install.sh"
    chmod +x "$DIST_DIR/uninstall.sh"
    
    echo ""
    echo "✅ Build completed successfully!"
    echo ""
    echo "Distribution package ready at: $DIST_DIR/"
    echo ""
    echo "To create a tarball for distribution:"
    echo "tar -czf hdd-tool_0.1.0_linux_x86_64.tar.gz -C dist hdd-tool_0.1.0_linux_x86_64"
    echo ""
    echo "To install locally:"
    echo "cd $DIST_DIR && sudo ./install.sh"
    
else
    echo "Warning: Distribution directory not found. Creating minimal package..."
    mkdir -p "linux_release"
    cp target/release/hdd_tool linux_release/
    cp LICENSE linux_release/
    cp CROSS_PLATFORM_README.md linux_release/README.md
    echo "Binary and documentation copied to linux_release/"
fi

echo ""
echo "🎉 HDD Tool Linux build complete!"