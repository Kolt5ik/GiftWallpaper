#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

if ! command -v cargo >/dev/null 2>&1; then
  echo "[ERROR] Rust/Cargo is not installed. See https://rustup.rs/"
  exit 1
fi

echo "[1/5] Toolchain"
rustc --version
cargo --version

echo "[2/5] Formatting"
cargo fmt --all -- --check

echo "[3/5] Compile check"
cargo check --all-targets

echo "[4/5] Tests"
cargo test

echo "[5/5] Clippy"
cargo clippy --all-targets -- -D warnings

echo "ALL CHECKS PASSED"
