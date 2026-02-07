#!/bin/bash
# Build standalone executable for RZEM AI Inference
# Supports: Linux, macOS, Windows (via WSL or MSYS2)

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}RZEM AI Inference - Standalone Build${NC}"
echo "======================================"
echo ""

# Detect platform
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
echo -e "${GREEN}Platform: ${PLATFORM}${NC}"

# Check Python version
PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
PYTHON_MAJOR=$(echo $PYTHON_VERSION | cut -d'.' -f1)
PYTHON_MINOR=$(echo $PYTHON_VERSION | cut -d'.' -f2)

if [ "$PYTHON_MAJOR" -lt 3 ] || [ "$PYTHON_MAJOR" -eq 3 -a "$PYTHON_MINOR" -lt 10 ]; then
    echo -e "${RED}Error: Python 3.10+ required (found $PYTHON_VERSION)${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Python $PYTHON_VERSION${NC}"

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo -e "${YELLOW}Creating virtual environment...${NC}"
    python3 -m venv venv
fi

# Activate virtual environment
source venv/bin/activate

# Install/upgrade build dependencies
echo -e "${YELLOW}Installing build dependencies...${NC}"
pip install --upgrade pip
pip install pyinstaller
pip install -r src-python/requirements.txt

echo -e "${GREEN}✓ Dependencies installed${NC}"

# Build frontend if not already built
if [ ! -d "dist" ]; then
    echo -e "${YELLOW}Building frontend...${NC}"
    npm install
    npm run build
    echo -e "${GREEN}✓ Frontend built${NC}"
else
    echo -e "${GREEN}✓ Frontend already built${NC}"
fi

# Clean previous build
echo -e "${YELLOW}Cleaning previous build...${NC}"
rm -rf build/ dist-standalone/
mkdir -p dist-standalone

# Run PyInstaller
echo -e "${YELLOW}Building standalone executable...${NC}"
pyinstaller build.spec --clean --noconfirm --distpath dist-standalone

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Build successful!${NC}"
    echo ""
    echo "Executable location:"

    if [ "$PLATFORM" = "darwin" ]; then
        echo "  dist-standalone/RZEM AI Inference.app"
        echo ""
        echo "To run:"
        echo "  open \"dist-standalone/RZEM AI Inference.app\""
    elif [ "$PLATFORM" = "linux" ]; then
        echo "  dist-standalone/RZEM-AI-Inference"
        echo ""
        echo "To run:"
        echo "  ./dist-standalone/RZEM-AI-Inference"
    else
        echo "  dist-standalone/RZEM-AI-Inference.exe"
        echo ""
        echo "To run:"
        echo "  ./dist-standalone/RZEM-AI-Inference.exe"
    fi

    # Show file size
    if [ "$PLATFORM" = "darwin" ]; then
        SIZE=$(du -sh "dist-standalone/RZEM AI Inference.app" | cut -f1)
    else
        SIZE=$(du -sh dist-standalone/RZEM-AI-Inference* | cut -f1)
    fi
    echo ""
    echo "Build size: $SIZE"

    # Package instructions
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Test the executable"
    echo "2. Create release archive:"
    if [ "$PLATFORM" = "darwin" ]; then
        echo "   cd dist-standalone && zip -r RZEM-AI-Inference-macOS.zip \"RZEM AI Inference.app\""
    elif [ "$PLATFORM" = "linux" ]; then
        echo "   cd dist-standalone && tar czf RZEM-AI-Inference-Linux.tar.gz RZEM-AI-Inference"
    else
        echo "   cd dist-standalone && zip RZEM-AI-Inference-Windows.zip RZEM-AI-Inference.exe"
    fi
    echo "3. Create GitHub release and upload archive"
    echo "4. Generate checksums:"
    echo "   shasum -a 256 dist-standalone/* > checksums.txt"

else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
