#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
export LOG_LEVEL=INFO
export DEV_MODE=1

cd "$ROOT_DIR"
exec env | grep DEBUG & uv run python main.py
