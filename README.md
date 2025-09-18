# SafeWipe

SafeWipe is a secure, cross-platform data sanitization and verification tool designed 
to comply with NIST SP 800-88 guidelines. It ensures sensitive data is permanently 
erased from storage devices (HDDs, SSDs, removable drives, and mobile devices) 
using industry-standard Clear, Purge, and Destroy methods.

## ✨ Features
- Cross-platform support (Windows, Linux, macOS, Android, iOS)
- Secure erasure methods:
  - Clear (overwrite)
  - Purge (ATA Secure Erase, NVMe Sanitize, Crypto Erase)
  - Destroy (documentation + workflow support)
- Verification of wipe success (block sampling, crypto key validation)
- Compliance reporting with certificates (JSON/PDF)
- Modern, unified UI with JetBrains Compose Multiplatform
- Rust-powered sanitization engine for maximum security and performance

## 🔒 Compliance
SafeWipe follows the sanitization categories and recommendations of 
**NIST Special Publication 800-88 Rev.1**.

## 🛠 Tech Stack
- **Rust** → Core sanitization engine (system commands, crypto, verification)
- **Compose Multiplatform (Kotlin)** → Cross-platform UI
- **Interop** → JNI (Android), Kotlin/Native cinterop (iOS/Desktop), UniFFI optional
- **Build Tools** → Gradle, Cargo, cbindgen, cargo-ndk

## 🚀 Roadmap
- [ ] Core sanitization engine in Rust
- [ ] Compose Multiplatform UI (drive selection, wipe progress, reports)
- [ ] Cross-platform bindings (Android/iOS/Desktop)
- [ ] Compliance testing and verification
- [ ] Packaging (installers, APKs, DMGs, AppImages)

---

SafeWipe makes secure data erasure simple, reliable, and compliant.
