#!/bin/bash

# HDD Tool - Direct Linux Executable
# Simple launcher for the HDD Tool on Linux systems

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY_PATH="$SCRIPT_DIR/target/release/hdd_tool"

echo "========================================"
echo "  HDD Tool v0.1.0"
echo "  NIST SP 800-88 Compliant Disk Sanitization"
echo "========================================"
echo

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    print_warning "HDD Tool requires root privileges to access storage devices"
    print_info "Restarting with sudo..."
    exec sudo "$0" "$@"
fi

# Check if binary exists
if [[ ! -f "$BINARY_PATH" ]]; then
    print_warning "Binary not found. Building project..."
    
    cd "$SCRIPT_DIR"
    
    # Check if Cargo.toml exists
    if [[ ! -f "Cargo.toml" ]]; then
        print_error "This doesn't appear to be the HDD Tool project directory"
        exit 1
    fi
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install Rust:"
        echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Check for system dependencies
    print_info "Checking system dependencies..."
    missing_deps=()
    
    if ! command -v hdparm &> /dev/null; then
        missing_deps+=("hdparm")
    fi
    
    if ! command -v lsblk &> /dev/null; then
        missing_deps+=("util-linux")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        print_warning "Missing system dependencies: ${missing_deps[*]}"
        print_info "Install with your package manager:"
        if command -v apt-get &> /dev/null; then
            echo "  sudo apt-get install -y ${missing_deps[*]}"
        elif command -v dnf &> /dev/null; then
            echo "  sudo dnf install -y ${missing_deps[*]}"
        fi
    fi
    
    # Build the project
    print_info "Building HDD Tool..."
    # Drop privileges for build (run as original user)
    if [[ -n "$SUDO_USER" ]]; then
        sudo -u "$SUDO_USER" cargo build --release
    else
        cargo build --release
    fi
    
    if [[ ! -f "$BINARY_PATH" ]]; then
        print_error "Build failed or binary not created"
        exit 1
    fi
    
    print_success "Build completed successfully"
fi

# Make binary executable
chmod +x "$BINARY_PATH"

# Security warning
print_warning "SECURITY WARNING:"
print_warning "This tool performs irreversible data destruction!"
print_warning "Always verify target devices before proceeding!"
echo

# Run the application
print_info "Starting HDD Tool..."
exec "$BINARY_PATH" "$@"