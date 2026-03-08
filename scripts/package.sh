#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/build/dist"
PACKAGE_DIR="$ROOT_DIR/build/packages"

APP_NAME="Inference"
VERSION="${1:-0.2.0}"
PLATFORM="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Build first
"$ROOT_DIR/scripts/build.sh"

echo "==> Packaging release..."
mkdir -p "$PACKAGE_DIR"

ARCHIVE_NAME="${APP_NAME}-${VERSION}-${PLATFORM}-${ARCH}"

cd "$DIST_DIR"

if [ "$PLATFORM" = "darwin" ]; then
  # macOS: create .dmg with drag-to-Applications layout
  create-dmg \
    --volname "$APP_NAME" \
    --window-size 600 400 \
    --icon-size 100 \
    --icon "$APP_NAME.app" 150 185 \
    --app-drop-link 450 185 \
    "$PACKAGE_DIR/${ARCHIVE_NAME}.dmg" \
    "$DIST_DIR/$APP_NAME.app"
  echo "Package: $PACKAGE_DIR/${ARCHIVE_NAME}.dmg"
else
  # Linux / other: create .tar.gz
  tar -czf "$PACKAGE_DIR/${ARCHIVE_NAME}.tar.gz" "$APP_NAME"
  echo "Package: $PACKAGE_DIR/${ARCHIVE_NAME}.tar.gz"
fi

echo ""
echo "Done! Release packages:"
ls -lh "$PACKAGE_DIR/"
