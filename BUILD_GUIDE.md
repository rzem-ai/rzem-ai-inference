# Building Standalone Executables

This guide explains how to build RZEM AI Inference as a standalone executable with auto-update functionality.

## Overview

The application can be built as a single executable that bundles:
- Python interpreter
- All Python dependencies (PyTorch, Diffusers, etc.)
- Frontend (Vue.js build)
- Application code

**NOT bundled** (downloaded on first run):
- AI models (~20GB from HuggingFace Hub)

## Quick Start

### Build Executable

```bash
# Linux/macOS
./scripts/build.sh

# Windows
scripts\build-windows.bat
```

### Create Release

```bash
# Tag version and create GitHub release
./scripts/create-release.sh v0.2.0
```

## Auto-Update System

The app automatically checks for updates and can update itself:

1. **Checks GitHub releases** every hour
2. **Downloads update** if available
3. **Verifies checksum** (SHA256)
4. **Installs and restarts** automatically

Users can also manually check via the UI.

## Build Sizes

| Platform | Compressed | Extracted |
|----------|------------|-----------|
| Linux    | ~350MB     | ~900MB    |
| macOS    | ~400MB     | ~1.1GB    |
| Windows  | ~450MB     | ~1.2GB    |

Models are NOT included (~20GB download on first run).

## See BUILD_GUIDE.md for detailed instructions
