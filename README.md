# 🔒 ShredX 1 - NIST SP 800-88 Compliant Data Sanitization Tool

## 🎯 Overview

ShredX 1 is an enterprise-grade data sanitization tool that fully complies with **NIST SP 800-88 Rev. 1 Guidelines for Media Sanitization**. It provides comprehensive data destruction capabilities for all types of storage devices including HDDs, SSDs, NVMe drives, USB drives, and SD cards.

## 🏗️ Quick Workflow Summary

```
🚀 Start → 🔍 Detection → 💻 UI → 👤 Mode Selection → 🛡️ NIST Methods → ✅ Validation → 
🔄 Sanitization → 📈 Monitoring → 🔍 Verification → 📄 Reports → 🏆 Certificates → ✅ Complete
```

## 🛡️ NIST 800-88 Compliance Levels

| Level | Security | Method | Use Case |
|-------|----------|---------|----------|
| **CLEAR** | Confidential | Single Pass Crypto Random | Software recovery protection |
| **PURGE** | Secret/Top Secret | 7-Pass Multi-Pattern | Laboratory recovery protection |
| **DESTROY** | Highest Security | Physical destruction guidance | Complete assurance |

## 📋 Key Features

- ✅ **NIST SP 800-88 Rev. 1 Compliant**
- ✅ **Hardware-based sanitization** (ATA/NVMe Secure Erase)
- ✅ **Multi-pass software sanitization** (DoD, Gutmann, Custom)
- ✅ **Real-time progress monitoring**
- ✅ **Comprehensive verification**
- ✅ **Digital certificate generation**
- ✅ **Audit trail logging**
- ✅ **Professional reporting**

## 🚀 Quick Start

1. **Run as Administrator**: Required for low-level drive access
2. **Select Device**: Choose target storage device
3. **Choose Method**: 
   - **Standard Mode**: NIST Clear/Purge buttons
   - **Advanced Mode**: Full algorithm selection
4. **Execute**: Monitor progress and review results
5. **Verify**: Automatic verification and reporting

## 📊 Supported Algorithms

### Hardware-Based (Recommended)
- ATA Secure Erase (Standard)
- ATA Enhanced Secure Erase  
- NVMe Secure Erase
- NVMe Cryptographic Erase

### NIST 800-88 Methods
- **NIST Clear**: Single pass cryptographic random
- **NIST Purge**: 7-pass enhanced destruction

### Additional Standards
- DoD 5220.22-M (3-pass)
- DoD 5220.22-M ECE (7-pass)
- Gutmann Method (35-pass)
- Custom patterns

## 📁 Documentation

- [📋 Complete Workflow Chart](NIST_800-88_WORKFLOW.md)
- [🔧 Advanced Features](ADVANCED_FEATURES.md)
- [🌐 Interactive Workflow](workflow_chart.html)

## 🔧 Technical Architecture

- **Core Engine**: `src/core/engine.rs`
- **Sanitization**: `src/sanitization.rs`
- **Hardware Interface**: `src/ata_commands.rs`
- **Security**: `src/security/`
- **Reporting**: `src/reporting/`

## ⚡ Performance Optimizations

- 16MB optimized buffer sizes
- Multi-threaded processing (4 threads)
- Sector-aligned operations
- Hardware acceleration when available

---

**⚠️ Security Notice**: This tool permanently destroys data. Ensure proper backups before use. Always verify compliance requirements for your specific use case.