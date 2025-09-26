HDD Tool v0.1.0 - Linux Distribution Package
===============================================

This package contains the HDD Tool, a NIST SP 800-88 compliant disk sanitization utility
for Linux systems.

QUICK START (Easiest):
1. Extract this package
2. Fix line endings: chmod +x fix-line-endings.sh && ./fix-line-endings.sh
3. ./hdd-tool

ALTERNATIVE QUICK START:
1. Extract this package
2. dos2unix hdd-tool && chmod +x hdd-tool
3. ./hdd-tool

INSTALLATION OPTIONS:
================
Option 1 - Direct Run (Recommended):
1. Extract package to any directory
2. chmod +x hdd-tool
3. ./hdd-tool

Option 2 - System Installation:
1. Extract package to temporary directory
2. sudo ./install.sh

Option 3 - Full Launcher:
1. chmod +x hdd-tool-linux
2. ./hdd-tool-linux help
3. ./hdd-tool-linux run

MANUAL INSTALLATION:
1. Copy bin/hdd_tool to /usr/local/bin/
2. Copy share/applications/hdd-tool.desktop to /usr/share/applications/
3. Copy share/pixmaps/hdd-tool.png to /usr/share/pixmaps/
4. Install dependencies: hdparm, util-linux

USAGE:
- Quick: ./hdd-tool
- System: sudo hdd_tool (after install.sh)
- GUI: Find in applications menu under "System Tools"

REQUIREMENTS:
- Linux x86_64 system
- Administrative privileges (root/sudo)
- hdparm package for SSD secure erase
- util-linux package for disk utilities

UNINSTALLATION:
Run: sudo ./uninstall.sh

For more information, see usr/share/doc/hdd-tool/README.md

BUILD INFORMATION:
==================
This is a cross-platform Rust application built with:
- Rust 2024 Edition
- eframe/egui for GUI
- Platform-specific system calls
- NIST SP 800-88 compliant sanitization algorithms

FEATURES:
=========
✅ Cross-platform compatibility (Windows/Linux)
✅ Device-specific sanitization algorithms
✅ NIST SP 800-88 compliant methods (Clear/Purge)
✅ Support for HDD, SSD, NVMe, USB, and SD cards
✅ Hardware-based secure erase when available
✅ Progress monitoring and audit reporting
✅ Comprehensive verification and validation

SECURITY NOTICE:
===============
This tool performs irreversible data destruction. Always verify target devices
before proceeding with sanitization operations. Maintain appropriate backups
of any data you wish to preserve.

BUILDING FROM SOURCE:
====================
To build on Linux:
1. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
2. Install dependencies: sudo apt install hdparm util-linux (Ubuntu/Debian)
3. Clone repository: git clone https://github.com/riteshvijaykumar/HDD-Tool
4. Build: cargo build --release
5. Run: ./target/release/hdd_tool

Copyright © 2025 SIH Team
Licensed under MIT License

For issues and support: https://github.com/riteshvijaykumar/HDD-Tool/issues