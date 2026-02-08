#!/bin/bash
# Dev mode: Vite HMR for frontend + watchfiles auto-restart for backend

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

cleanup() {
    echo ""
    echo -e "${YELLOW}Shutting down...${NC}"
    # Kill all child processes
    kill 0 2>/dev/null
    exit 0
}
trap cleanup SIGINT SIGTERM

# Activate venv
if [ ! -d "venv" ]; then
    echo -e "${RED}No venv found. Run './run-python.sh' first to set up.${NC}"
    exit 1
fi
source venv/bin/activate

# Ensure watchfiles is installed
if ! python3 -c "import watchfiles" 2>/dev/null; then
    echo -e "${YELLOW}Installing watchfiles...${NC}"
    pip install watchfiles
fi

echo -e "${GREEN}RZEM AI Inference - Dev Mode${NC}"
echo "======================================"
echo -e "Frontend: ${GREEN}http://localhost:5173${NC} (Vite HMR)"
echo -e "Backend:  ${GREEN}auto-restart on .py changes${NC}"
echo "======================================"
echo ""

# Start Vite dev server in background
npm run dev &
VITE_PID=$!

# Wait for Vite to be ready
echo -e "${YELLOW}Waiting for Vite dev server...${NC}"
for i in $(seq 1 30); do
    if curl -s http://localhost:5173 > /dev/null 2>&1; then
        echo -e "${GREEN}Vite dev server ready${NC}"
        break
    fi
    sleep 0.5
done

# Start Python backend with watchfiles (auto-restart on .py changes)
watchfiles --filter python "python3 src-python/main.py --debug --dev" src-python/ &
PYTHON_PID=$!

echo ""
echo -e "${GREEN}Dev environment running. Press Ctrl+C to stop.${NC}"
echo ""

# Wait for either process to exit
wait
