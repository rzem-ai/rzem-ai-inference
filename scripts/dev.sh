#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Activate venv if present
if [ -f "$ROOT_DIR/.venv/bin/activate" ]; then
  source "$ROOT_DIR/.venv/bin/activate"
fi

cleanup() {
  echo ""
  echo "==> Shutting down..."
  kill "$VITE_PID" 2>/dev/null || true
  wait "$VITE_PID" 2>/dev/null || true
  exit 0
}
trap cleanup SIGINT SIGTERM

echo "==> Starting Vite dev server..."
cd "$ROOT_DIR/frontend"
npm run dev &
VITE_PID=$!

# Wait for Vite to be ready
echo "==> Waiting for Vite on http://localhost:1978..."
for i in $(seq 1 30); do
  if curl -s http://localhost:1978 >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done

echo "==> Starting pywebview (dev mode)..."
cd "$ROOT_DIR"
DEV_MODE=1 python main.py

# pywebview exited (window closed) — clean up Vite
cleanup
