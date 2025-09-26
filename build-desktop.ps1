# Build script without server features

# Build only the main desktop application (no server)
cargo build --bin hdd-tool --no-default-features

Write-Host "✅ Desktop application built successfully!"
Write-Host "📍 Location: target/debug/hdd-tool.exe"
Write-Host ""
Write-Host "🚀 To run: ./target/debug/hdd-tool.exe"
Write-Host ""
Write-Host "📝 Note: Server features require PostgreSQL database setup"
Write-Host "   See SERVER_SETUP_GUIDE.md for database configuration"