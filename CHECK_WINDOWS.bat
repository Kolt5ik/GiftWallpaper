@echo off
setlocal
cd /d "%~dp0"

echo ==========================================
echo       GiftWallpaper - Windows checks
echo ==========================================
echo.

where cargo >nul 2>nul
if errorlevel 1 (
    echo [ERROR] Cargo is not installed. Install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo [1/5] Toolchain
rustc --version || goto :fail
cargo --version || goto :fail

echo.
echo [2/5] Formatting
cargo fmt --all -- --check || goto :fail

echo.
echo [3/5] Compile check
cargo check --all-targets || goto :fail

echo.
echo [4/5] Tests
cargo test || goto :fail

echo.
echo [5/5] Clippy
cargo clippy --all-targets -- -D warnings || goto :fail

echo.
echo ==========================================
echo ALL CHECKS PASSED
echo ==========================================
pause
exit /b 0

:fail
echo.
echo ==========================================
echo CHECK FAILED - copy the error text and send it to me.
echo ==========================================
pause
exit /b 1
