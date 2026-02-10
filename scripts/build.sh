#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/build/dist"

# Activate venv if present
if [ -f "$ROOT_DIR/.venv/bin/activate" ]; then
  source "$ROOT_DIR/.venv/bin/activate"
fi

echo "==> Building frontend..."
cd "$ROOT_DIR/frontend"
npm run build

echo "==> Building application with PyInstaller..."
cd "$ROOT_DIR"

pyinstaller \
  --name "Inference" \
  --distpath "$DIST_DIR" \
  --workpath "$ROOT_DIR/build/work" \
  --specpath "$ROOT_DIR/build" \
  --noconfirm \
  --windowed \
  --add-data "$ROOT_DIR/frontend/dist:frontend/dist" \
  main.py

echo ""
echo "Build complete: $DIST_DIR/Inference/"
