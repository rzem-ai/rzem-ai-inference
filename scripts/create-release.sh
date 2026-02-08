#!/bin/bash
# Create GitHub release with executables and checksums
# Usage: ./scripts/create-release.sh <version>

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v0.2.0"
    exit 1
fi

VERSION=$1
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')

echo "Creating release $VERSION for $PLATFORM"

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) not found"
    echo "Install: https://cli.github.com/"
    exit 1
fi

# Check if built
if [ ! -d "dist-standalone" ]; then
    echo "Error: No build found. Run ./scripts/build.sh first"
    exit 1
fi

# Create release archive
cd dist-standalone

if [ "$PLATFORM" = "darwin" ]; then
    ARCHIVE="RZEM-AI-Inference-macOS-$VERSION.zip"
    echo "Creating macOS archive..."
    zip -r "$ARCHIVE" "RZEM AI Inference.app"
elif [ "$PLATFORM" = "linux" ]; then
    ARCHIVE="RZEM-AI-Inference-Linux-$VERSION.tar.gz"
    echo "Creating Linux archive..."
    tar czf "$ARCHIVE" RZEM-AI-Inference
else
    ARCHIVE="RZEM-AI-Inference-Windows-$VERSION.zip"
    echo "Creating Windows archive..."
    zip "$ARCHIVE" RZEM-AI-Inference.exe
fi

# Generate checksums
echo "Generating checksums..."
shasum -a 256 "$ARCHIVE" > checksums.txt

cd ..

# Create GitHub release
echo "Creating GitHub release..."
gh release create "$VERSION" \
    --title "RZEM AI Inference $VERSION" \
    --notes "Auto-generated release for $VERSION

## What's New
- See CHANGELOG.md for details

## Installation
1. Download the archive for your platform
2. Extract the executable
3. Run the application
4. Models will download automatically on first run (~20GB)

## Checksums
See checksums.txt for SHA256 verification" \
    "dist-standalone/$ARCHIVE" \
    "dist-standalone/checksums.txt"

echo ""
echo "✓ Release $VERSION created successfully!"
echo "Archive: dist-standalone/$ARCHIVE"
echo ""
echo "To delete this release:"
echo "  gh release delete $VERSION"
