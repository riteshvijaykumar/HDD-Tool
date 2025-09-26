#!/bin/bash
# Quick Fix for Rustup Default Toolchain Issue

echo "🔧 Fixing Rust toolchain configuration..."

# Set default stable toolchain
echo "📦 Setting default stable Rust toolchain..."
rustup default stable

# Update toolchain to latest
echo "⬆️ Updating to latest stable..."
rustup update stable

# Verify installation
echo "✅ Verification:"
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Default toolchain: $(rustup default)"

echo ""
echo "🚀 Rust is now ready! You can run: ./hdd-tool"