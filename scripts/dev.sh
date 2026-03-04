#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
export LOG_LEVEL=DEBUG
export DEV_MODE=1

export DISABLE_SSL_VERIFICATION=1

# VS Code snap overrides GTK/GIO/XDG env vars, causing WebKit sub-processes
# to load snap's libpthread which crashes immediately. Restore originals.
unset GTK_PATH GTK_EXE_PREFIX GTK_IM_MODULE_FILE GIO_MODULE_DIR LOCPATH \
      GSETTINGS_SCHEMA_DIR GDK_BACKEND XDG_DATA_HOME
# Restore XDG_DATA_DIRS to pre-snap value if available
if [[ -n "${XDG_DATA_DIRS_VSCODE_SNAP_ORIG:-}" ]]; then
  export XDG_DATA_DIRS="$XDG_DATA_DIRS_VSCODE_SNAP_ORIG"
fi

cd "$ROOT_DIR"
uv run python main.py
