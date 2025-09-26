@echo off
REM Cross-platform build script for Windows
REM Usage: build-release.bat [version]

setlocal enabledelayedexpansion

set VERSION=%1
if "%VERSION%"=="" set VERSION=v0.1.0

echo Building HDD Tool release version: %VERSION%

REM Check if Rust is installed
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust/Cargo is not installed. Please install Rust first.
    exit /b 1
)

echo [INFO] Checking dependencies...

REM Install cross if not available
where cross >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [WARNING] Cross is not installed. Installing...
    cargo install cross
)

echo [SUCCESS] Dependencies check completed

echo [INFO] Installing cross-compilation targets...

REM Install Windows targets
rustup target add x86_64-pc-windows-msvc
rustup target add i686-pc-windows-msvc

REM Install Linux targets (for WSL/Docker cross-compilation)
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu

echo [SUCCESS] Cross-compilation targets installed

REM Create release directory
set RELEASE_DIR=releases\%VERSION%
if exist "%RELEASE_DIR%" rmdir /s /q "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%"

echo [INFO] Building for Windows x64...
mkdir "%RELEASE_DIR%\windows-x64"
cargo build --release --target x86_64-pc-windows-msvc --bin hdd-tool
cargo build --release --target x86_64-pc-windows-msvc --bin hdd-tool-server --features server

copy "target\x86_64-pc-windows-msvc\release\hdd-tool.exe" "%RELEASE_DIR%\windows-x64\"
copy "target\x86_64-pc-windows-msvc\release\hdd-tool-server.exe" "%RELEASE_DIR%\windows-x64\"

REM Copy resources
if exist "resources" xcopy /E /I "resources" "%RELEASE_DIR%\windows-x64\resources\"
if exist "reference" xcopy /E /I "reference" "%RELEASE_DIR%\windows-x64\reference\"
copy "README.md" "%RELEASE_DIR%\windows-x64\"

REM Create Windows installer
call :create_windows_installer "%RELEASE_DIR%\windows-x64"

REM Create zip archive
cd "%RELEASE_DIR%\windows-x64"
powershell -Command "Compress-Archive -Path * -DestinationPath '../hdd-tool-windows-x64.zip'"
cd ..\..

echo [SUCCESS] Build completed for Windows x64

echo [INFO] Building for Windows x86...
mkdir "%RELEASE_DIR%\windows-x86"
cargo build --release --target i686-pc-windows-msvc --bin hdd-tool
cargo build --release --target i686-pc-windows-msvc --bin hdd-tool-server --features server

copy "target\i686-pc-windows-msvc\release\hdd-tool.exe" "%RELEASE_DIR%\windows-x86\"
copy "target\i686-pc-windows-msvc\release\hdd-tool-server.exe" "%RELEASE_DIR%\windows-x86\"

REM Copy resources
if exist "resources" xcopy /E /I "resources" "%RELEASE_DIR%\windows-x86\resources\"
if exist "reference" xcopy /E /I "reference" "%RELEASE_DIR%\windows-x86\reference\"
copy "README.md" "%RELEASE_DIR%\windows-x86\"

REM Create Windows installer
call :create_windows_installer "%RELEASE_DIR%\windows-x86"

REM Create zip archive
cd "%RELEASE_DIR%\windows-x86"
powershell -Command "Compress-Archive -Path * -DestinationPath '../hdd-tool-windows-x86.zip'"
cd ..\..

echo [SUCCESS] Build completed for Windows x86

REM Create release notes
call :create_release_notes

echo [SUCCESS] Cross-platform build completed!
echo [INFO] Release artifacts available in: %RELEASE_DIR%\
echo.
echo Built packages:
dir "%RELEASE_DIR%\*.zip" 2>nul

pause
exit /b 0

:create_windows_installer
set INSTALLER_DIR=%~1
(
echo @echo off
echo echo Installing HDD Tool...
echo.
echo REM Check for administrator privileges
echo net session ^>nul 2^>^&1
echo if %%errorLevel%% == 0 ^(
echo     echo Administrator privileges detected.
echo ^) else ^(
echo     echo This installer requires administrator privileges.
echo     echo Please run as administrator.
echo     pause
echo     exit /b 1
echo ^)
echo.
echo REM Create program directory
echo if not exist "%%PROGRAMFILES%%\HDD Tool\" ^(
echo     mkdir "%%PROGRAMFILES%%\HDD Tool\"
echo ^)
echo.
echo REM Copy binaries
echo copy "hdd-tool.exe" "%%PROGRAMFILES%%\HDD Tool\"
echo copy "hdd-tool-server.exe" "%%PROGRAMFILES%%\HDD Tool\"
echo.
echo REM Copy resources
echo if exist "resources" ^(
echo     xcopy /E /I "resources" "%%PROGRAMFILES%%\HDD Tool\resources\"
echo ^)
echo if exist "reference" ^(
echo     xcopy /E /I "reference" "%%PROGRAMFILES%%\HDD Tool\reference\"
echo ^)
echo.
echo REM Add to PATH
echo setx /M PATH "%%PATH%%;%%PROGRAMFILES%%\HDD Tool"
echo.
echo REM Create desktop shortcut
echo set "desktop=%%USERPROFILE%%\Desktop"
echo set "shortcut=%%desktop%%\HDD Tool.lnk"
echo powershell -Command "$WshShell = New-Object -comObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%%shortcut%%'^); $Shortcut.TargetPath = '%%PROGRAMFILES%%\HDD Tool\hdd-tool.exe'; $Shortcut.Save(^)"
echo.
echo echo.
echo echo Installation complete!
echo echo.
echo echo You can now:
echo echo - Run 'hdd-tool' from Command Prompt
echo echo - Use the desktop shortcut
echo echo - Find HDD Tool in the Start Menu
echo echo.
echo pause
) > "%INSTALLER_DIR%\install.bat"
exit /b 0

:create_release_notes
(
echo # HDD Tool %VERSION% Release Notes
echo.
echo ## Overview
echo HDD Tool is a cross-platform disk sanitization utility that provides NIST SP 800-88 compliant data erasure for various storage devices.
echo.
echo ## Features
echo - **NIST SP 800-88 Compliance**: Implements approved sanitization methods
echo - **Multi-Device Support**: HDDs, SSDs, NVMe drives, USB storage, SD cards
echo - **Cross-Platform**: Windows, Linux, macOS support
echo - **Authentication System**: User management with role-based access
echo - **Server Backend**: PostgreSQL database with REST API
echo - **Web Dashboard**: Browser-based management interface
echo - **Audit Trail**: Comprehensive logging and reporting
echo.
echo ## Windows Support
echo.
echo ### Windows x64
echo - **Requirements**: Windows 10/11 ^(64-bit^), Administrator privileges
echo - **Installation**: Extract hdd-tool-windows-x64.zip and run install.bat as Administrator
echo - **Features**: Full disk access, SMART monitoring, ATA command support
echo.
echo ### Windows x86
echo - **Requirements**: Windows 10/11 ^(32-bit^), Administrator privileges  
echo - **Installation**: Extract hdd-tool-windows-x86.zip and run install.bat as Administrator
echo - **Features**: Full disk access, SMART monitoring, ATA command support
echo.
echo ## Installation Instructions
echo.
echo ### Quick Start
echo 1. Download the appropriate package for your platform
echo 2. Extract the archive
echo 3. Run install.bat as Administrator
echo 4. Launch HDD Tool from desktop shortcut or command prompt
echo.
echo ### Server Setup
echo For server functionality:
echo 1. Install PostgreSQL
echo 2. Configure environment variables
echo 3. Run hdd-tool-server.exe
echo 4. Access web dashboard at http://localhost:3030
echo.
echo ## Security Notes
echo - Always run with administrator privileges for disk access
echo - Server communications use HTTPS in production
echo - User credentials are securely hashed
echo - Audit logs are tamper-evident
echo.
echo ## Changelog
echo - Initial Windows release
echo - Complete NIST SP 800-88 implementation
echo - Web-based management interface
echo - Multi-architecture support ^(x64, x86^)
) > "%RELEASE_DIR%\RELEASE_NOTES.md"
exit /b 0