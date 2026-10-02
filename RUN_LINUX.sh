#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

echo "=========================================="
echo "      GiftWallpaper - Linux runner"
echo "=========================================="

if ! command -v cargo >/dev/null 2>&1; then
  echo "[ERROR] Rust/Cargo is not installed."
  echo "Install Rust from https://rustup.rs/ and run this script again."
  exit 1
fi

echo "[OK] $(rustc --version)"
echo "[OK] $(cargo --version)"
echo
echo "Building and starting GiftWallpaper..."
cargo run --release
