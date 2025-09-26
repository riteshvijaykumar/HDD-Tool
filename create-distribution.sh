#!/bin/bash

# HDD Tool - Create Linux Distribution Archive
# This script creates a complete distribution tarball for Linux

set -e

# Configuration
VERSION="0.1.0"
PACKAGE_NAME="hdd-tool_${VERSION}_linux_x86_64"
DIST_DIR="dist"
ARCHIVE_NAME="${PACKAGE_NAME}.tar.gz"

echo "========================================"
echo "  HDD Tool Linux Distribution Creator"
echo "  Creating: $ARCHIVE_NAME"
echo "========================================"
echo

# Check if distribution directory exists
if [[ ! -d "$DIST_DIR/$PACKAGE_NAME" ]]; then
    echo "❌ Distribution directory not found: $DIST_DIR/$PACKAGE_NAME"
    echo "Make sure you've run the package creation scripts first."
    exit 1
fi

# Make all scripts executable
echo "🔧 Setting executable permissions..."
chmod +x "$DIST_DIR/$PACKAGE_NAME/hdd-tool"
chmod +x "$DIST_DIR/$PACKAGE_NAME/run-linux.sh" 
chmod +x "$DIST_DIR/$PACKAGE_NAME/hdd-tool-linux"
chmod +x "$DIST_DIR/$PACKAGE_NAME/install.sh"
chmod +x "$DIST_DIR/$PACKAGE_NAME/uninstall.sh"

# Create archive
echo "📦 Creating distribution archive..."
cd "$DIST_DIR"
tar -czf "../$ARCHIVE_NAME" "$PACKAGE_NAME"
cd ..

# Generate checksums
echo "🔐 Generating checksums..."
if command -v sha256sum &> /dev/null; then
    sha256sum "$ARCHIVE_NAME" > "${ARCHIVE_NAME}.sha256"
elif command -v shasum &> /dev/null; then
    shasum -a 256 "$ARCHIVE_NAME" > "${ARCHIVE_NAME}.sha256"
else
    echo "⚠️  Warning: No checksum utility found"
fi

# Show results
echo
echo "✅ Distribution package created successfully!"
echo
echo "📁 Files created:"
echo "   - $ARCHIVE_NAME ($(du -h "$ARCHIVE_NAME" | cut -f1))"
if [[ -f "${ARCHIVE_NAME}.sha256" ]]; then
    echo "   - ${ARCHIVE_NAME}.sha256"
fi
echo

echo "📋 Distribution Contents:"
echo "   • hdd-tool           - Quick launcher (recommended)"
echo "   • run-linux.sh       - Standard launcher"
echo "   • hdd-tool-linux     - Full-featured launcher"
echo "   • install.sh         - System installer"
echo "   • uninstall.sh       - System uninstaller" 
echo "   • README.txt         - Usage instructions"
echo "   • Desktop integration files"
echo "   • Documentation and license"
echo

echo "🚀 Usage Instructions for End Users:"
echo "   1. Download: $ARCHIVE_NAME"
echo "   2. Extract: tar -xzf $ARCHIVE_NAME"
echo "   3. Run: cd $PACKAGE_NAME && ./hdd-tool"
echo

echo "📤 Ready for distribution!"

# Optional: Show upload commands
cat << EOF

📡 Suggested Upload Commands:
   # GitHub Release
   gh release create v$VERSION $ARCHIVE_NAME ${ARCHIVE_NAME}.sha256

   # SCP to server
   scp $ARCHIVE_NAME ${ARCHIVE_NAME}.sha256 user@server:/path/to/downloads/

   # Verify download
   wget https://your-site.com/downloads/$ARCHIVE_NAME
   sha256sum -c ${ARCHIVE_NAME}.sha256

EOF