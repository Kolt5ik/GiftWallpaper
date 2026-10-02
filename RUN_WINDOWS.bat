@echo off
setlocal
cd /d "%~dp0"

echo ==========================================
echo       GiftWallpaper - Windows runner
echo ==========================================
echo.

where cargo >nul 2>nul
if errorlevel 1 (
    echo [ERROR] Rust/Cargo is not installed or is not in PATH.
    echo.
    echo 1. Open https://rustup.rs/
    echo 2. Install Rust using the default stable MSVC toolchain.
    echo 3. If Rust asks for Visual Studio C++ Build Tools, install them.
    echo 4. Close this window, open a new terminal, and run this file again.
    echo.
    start "" "https://rustup.rs/"
    pause
    exit /b 1
)

echo [OK] Rust found:
rustc --version
cargo --version
echo.
echo Building and starting GiftWallpaper...
echo First build can take a while because Cargo downloads and compiles dependencies.
echo.

cargo run --release
if errorlevel 1 (
    echo.
    echo [ERROR] Build or launch failed.
    echo Run CHECK_WINDOWS.bat to get a clearer diagnostic.
    pause
    exit /b 1
)

endlocal
