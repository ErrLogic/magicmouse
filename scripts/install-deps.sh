#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f /etc/arch-release ]]; then
    echo "This dependency installer currently targets Arch Linux."
    exit 1
fi

sudo pacman -S --needed --noconfirm base-devel pkgconf systemd

if command -v rustup >/dev/null 2>&1; then
    rustup toolchain install stable
    rustup default stable
    rustup component add rustfmt clippy
else
    sudo pacman -S --needed --noconfirm rust
fi

echo "Dependencies ready."
