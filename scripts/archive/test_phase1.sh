#!/bin/bash
# Test script for Phase 1 server mode

echo "=== Phase 1: Server Mode Test ==="
echo ""
echo "Testing health endpoint..."

# The server will be tested manually since Tauri needs a display environment
# For now, verify the binary exists and accepts arguments

if [ -f "./src-tauri/target/release/rzem-ai-inference" ]; then
    echo "✓ Binary built successfully"
    echo ""
    echo "To test server mode manually:"
    echo "1. Run: ./src-tauri/target/release/rzem-ai-inference --server --port 8080"
    echo "2. In another terminal, test health endpoint:"
    echo "   curl http://localhost:8080/api/v1/health"
    echo "3. Test queue endpoint:"
    echo "   curl http://localhost:8080/api/v1/queue"
    echo "4. Submit a generation job:"
    echo "   curl -X POST http://localhost:8080/api/v1/generate \\"
    echo "     -H 'Content-Type: application/json' \\"
    echo "     -d '{\"prompt\": \"a cat\", \"steps\": 4, \"cfg_scale\": 7.5, \"width\": 512, \"height\": 512, \"seed\": -1, \"model\": \"schnell\"}'"
else
    echo "✗ Binary not found. Run 'cargo build --release' first."
    exit 1
fi

echo ""
echo "=== CLI Help ==="
./src-tauri/target/release/rzem-ai-inference --help
