#!/usr/bin/env bash
set -euo pipefail

if command -v apt-get >/dev/null 2>&1; then
  sudo apt-get update
  sudo apt-get install -y \
    build-essential \
    pkg-config \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libssl-dev
  echo "Linux build dependencies installed."
else
  echo "This helper currently supports Debian/Ubuntu via apt-get."
  echo "For another distro, install the eframe/winit X11/Wayland development dependencies."
  exit 1
fi
