#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/build/dist"
PACKAGE_DIR="$ROOT_DIR/build/packages"

APP_NAME="PyWebViewApp"
VERSION="${1:-0.1.0}"
PLATFORM="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Build first
"$ROOT_DIR/scripts/build.sh"

echo "==> Packaging release..."
mkdir -p "$PACKAGE_DIR"

ARCHIVE_NAME="${APP_NAME}-${VERSION}-${PLATFORM}-${ARCH}"

cd "$DIST_DIR"

if [ "$PLATFORM" = "darwin" ]; then
  # macOS: create .zip
  zip -r "$PACKAGE_DIR/${ARCHIVE_NAME}.zip" "$APP_NAME"
  echo "Package: $PACKAGE_DIR/${ARCHIVE_NAME}.zip"
else
  # Linux / other: create .tar.gz
  tar -czf "$PACKAGE_DIR/${ARCHIVE_NAME}.tar.gz" "$APP_NAME"
  echo "Package: $PACKAGE_DIR/${ARCHIVE_NAME}.tar.gz"
fi

echo ""
echo "Done! Release package:"
ls -lh "$PACKAGE_DIR/${ARCHIVE_NAME}"*
