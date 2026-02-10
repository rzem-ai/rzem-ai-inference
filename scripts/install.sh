#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "==> Creating Python virtual environment..."
# Use /usr/bin/python3 with --system-site-packages to access system PyGObject (gi)
# and GTK/WebKit bindings, which pywebview requires on Linux.
# Falls back to whatever python3 is in PATH if /usr/bin/python3 doesn't exist.
PYTHON="${PYTHON:-$(command -v /usr/bin/python3 || command -v python3)}"
echo "    Using: $PYTHON ($($PYTHON --version 2>&1))"
"$PYTHON" -m venv --system-site-packages "$ROOT_DIR/.venv"
source "$ROOT_DIR/.venv/bin/activate"

echo "==> Installing Python dependencies..."
pip install --upgrade pip
pip install -r "$ROOT_DIR/requirements.txt"
pip install pyinstaller

echo "==> Installing frontend dependencies..."
cd "$ROOT_DIR/frontend"
npm install

echo ""
echo "Done! Activate the venv with:"
echo "  source .venv/bin/activate"
