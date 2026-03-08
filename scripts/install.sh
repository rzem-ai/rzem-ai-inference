#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OS="$(uname -s)"

if [ "$OS" = "Linux" ]; then
  echo "==> Installing system dependencies for pywebview (GTK/WebKit)..."
  sudo apt-get install -y \
    gir1.2-webkit2-4.1 \
    gir1.2-gtk-3.0 \
    libgirepository-2.0-dev \
    gcc \
    libcairo2-dev \
    pkg-config
fi

echo "==> Creating Python virtual environment..."
cd "$ROOT_DIR"
if [ "$OS" = "Linux" ]; then
  # --system-site-packages required on Linux to access system PyGObject (gi)
  # and GTK/WebKit bindings that pywebview depends on.
  uv venv .venv --system-site-packages
else
  uv venv .venv
fi

echo "==> Installing Python dependencies..."
uv sync
if [ "$OS" = "Linux" ]; then
  uv pip install pyinstaller PyGObject
else
  uv pip install pyinstaller
fi

echo "==> Installing frontend dependencies..."
cd "$ROOT_DIR/frontend"
npm install

echo ""
echo "Done! Run the app with:"
echo "  bash scripts/dev.sh"
