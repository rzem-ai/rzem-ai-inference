#!/usr/bin/env bash
# Build the patched Electrobun extractor from the local fork and copy it
# into node_modules. This runs automatically via postinstall.
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FORK_DIR="$(cd "$PROJECT_DIR/.." && pwd)/electrobun/package"
EXTRACTOR_SRC="$FORK_DIR/src/extractor"
ZIG="$FORK_DIR/vendors/zig/zig"
TARGET="$PROJECT_DIR/node_modules/electrobun/dist-linux-x64/extractor"

if [ ! -f "$ZIG" ]; then
  echo "Zig not found at $ZIG — downloading..."
  mkdir -p "$FORK_DIR/vendors/zig"
  curl -fSL https://ziglang.org/download/0.13.0/zig-linux-x86_64-0.13.0.tar.xz \
    | tar -xJ --strip-components=1 -C "$FORK_DIR/vendors/zig" \
      zig-linux-x86_64-0.13.0/zig zig-linux-x86_64-0.13.0/lib zig-linux-x86_64-0.13.0/doc
fi

echo "Building extractor from fork..."
cd "$EXTRACTOR_SRC" && "$ZIG" build -Doptimize=ReleaseSmall -Dcpu=baseline

if [ -d "$(dirname "$TARGET")" ]; then
  cp "$EXTRACTOR_SRC/zig-out/bin/extractor" "$TARGET"
  echo "Patched extractor installed to $TARGET"
else
  echo "Warning: node_modules/electrobun/dist-linux-x64/ not found — skipping copy"
fi
