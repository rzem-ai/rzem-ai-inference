#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Activate venv if present
if [ -f "$ROOT_DIR/.venv/bin/activate" ]; then
  source "$ROOT_DIR/.venv/bin/activate"
fi

# Build frontend if dist doesn't exist
if [ ! -f "$ROOT_DIR/frontend/dist/index.html" ]; then
  echo "==> Building frontend..."
  cd "$ROOT_DIR/frontend"
  npm run build
fi

echo "==> Running application..."
cd "$ROOT_DIR"
python main.py
