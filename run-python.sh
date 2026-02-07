#!/bin/bash
# Run the Python backend with pywebview

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}RZEM AI Inference - Python Backend${NC}"
echo "======================================"
echo ""

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}Error: Python 3 is not installed${NC}"
    echo "Please install Python 3.10 or later"
    exit 1
fi

# Check Python version
PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
PYTHON_MAJOR=$(echo $PYTHON_VERSION | cut -d'.' -f1)
PYTHON_MINOR=$(echo $PYTHON_VERSION | cut -d'.' -f2)

if [ "$PYTHON_MAJOR" -lt 3 ] || [ "$PYTHON_MAJOR" -eq 3 -a "$PYTHON_MINOR" -lt 10 ]; then
    echo -e "${RED}Error: Python 3.10 or later is required${NC}"
    echo "Current version: $PYTHON_VERSION"
    exit 1
fi

echo -e "${GREEN}✓ Python $PYTHON_VERSION${NC}"

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo -e "${YELLOW}Creating virtual environment...${NC}"
    python3 -m venv venv
    echo -e "${GREEN}✓ Virtual environment created${NC}"
fi

# Activate virtual environment
source venv/bin/activate

# Check if dependencies are installed
if ! python3 -c "import torch" 2>/dev/null; then
    echo -e "${YELLOW}Installing dependencies...${NC}"
    echo "This may take several minutes..."
    pip install -r src-python/requirements.txt
    echo -e "${GREEN}✓ Dependencies installed${NC}"
else
    echo -e "${GREEN}✓ Dependencies already installed${NC}"
fi

# Check if frontend is built
if [ ! -d "dist" ]; then
    echo -e "${YELLOW}Frontend not built. Building now...${NC}"
    npm run build
    echo -e "${GREEN}✓ Frontend built${NC}"
else
    echo -e "${GREEN}✓ Frontend already built${NC}"
fi

echo ""
echo "======================================"
echo -e "${GREEN}Starting application...${NC}"
echo "======================================"
echo ""

# Run the Python backend
python3 src-python/main.py "$@"
