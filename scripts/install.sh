#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "==> Creating Python virtual environment..."
# Use /usr/bin/python3 with --system-site-packages to access system PyGObject (gi)
# and GTK/WebKit bindings, which pywebview requires on Linux.
uv venv --system-site-packages --python /usr/bin/python3 "$ROOT_DIR/.venv"

echo "==> Installing Python dependencies..."
cd "$ROOT_DIR"
uv sync
uv pip install pyinstaller

echo "==> Installing frontend dependencies..."
cd "$ROOT_DIR/frontend"
npm install

echo ""
echo "Done! Run the app with:"
echo "  bash scripts/dev.sh"
