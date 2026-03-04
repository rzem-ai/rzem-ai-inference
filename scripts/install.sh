#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

#echo "==> Installing system dependencies for pywebview (GTK/WebKit)..."
#sudo apt-get install -y \
#  gir1.2-webkit2-4.1 \
#  gir1.2-gtk-3.0 \
#  libgirepository-2.0-dev \
#  gcc \
#  libcairo2-dev \
#  pkg-config

echo "==> Creating Python virtual environment..."
# Use /usr/bin/python3 with --system-site-packages to access system PyGObject (gi)
# and GTK/WebKit bindings, which pywebview requires on Linux.
#uv venv --system-site-packages --python "$ROOT_DIR/.venv"

echo "==> Installing Python dependencies..."
cd "$ROOT_DIR"
uv sync
uv pip install pyinstaller PyGObject

echo "==> Installing frontend dependencies..."
cd "$ROOT_DIR/frontend"
npm install

echo ""
echo "Done! Run the app with:"
echo "  bash scripts/dev.sh"
