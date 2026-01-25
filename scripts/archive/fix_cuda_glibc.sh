#!/bin/bash
# Fix CUDA 12.8 / glibc 2.40+ header conflict
# Patches math_functions.h to add noexcept(true) to conflicting declarations

set -e

CUDA_HEADER="/usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h"
BACKUP_HEADER="${CUDA_HEADER}.bak"

echo "==================================="
echo "CUDA/glibc Header Conflict Fix"
echo "==================================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "This script requires root privileges."
    echo "Please run: sudo $0"
    exit 1
fi

# Check if header exists
if [ ! -f "$CUDA_HEADER" ]; then
    echo "ERROR: CUDA header not found at:"
    echo "  $CUDA_HEADER"
    exit 1
fi

# Create backup if it doesn't exist
if [ ! -f "$BACKUP_HEADER" ]; then
    echo "Creating backup: ${BACKUP_HEADER}"
    cp "$CUDA_HEADER" "$BACKUP_HEADER"
else
    echo "Backup already exists: ${BACKUP_HEADER}"
fi

echo ""
echo "Patching CUDA headers..."

# Patch rsqrt
sed -i 's/rsqrt(double x);$/rsqrt(double x) noexcept(true);/' "$CUDA_HEADER"
echo "  - Patched rsqrt(double)"

sed -i 's/rsqrtf(float x);$/rsqrtf(float x) noexcept(true);/' "$CUDA_HEADER"
echo "  - Patched rsqrtf(float)"

# Patch sinpi
sed -i 's/sinpi(double x);$/sinpi(double x) noexcept(true);/' "$CUDA_HEADER"
echo "  - Patched sinpi(double)"

sed -i 's/sinpif(float x);$/sinpif(float x) noexcept(true);/' "$CUDA_HEADER"
echo "  - Patched sinpif(float)"

# Patch cospi
sed -i 's/cospi(double x);$/cospi(double x) noexcept(true);/' "$CUDA_HEADER"
echo "  - Patched cospi(double)"

sed -i 's/cospif(float x);$/cospif(float x) noexcept(true);/' "$CUDA_HEADER"
echo "  - Patched cospif(float)"

echo ""
echo "Verifying patches..."

# Verify patches were applied
PATCHED=0
grep -q "rsqrt(double x) noexcept(true);" "$CUDA_HEADER" && ((PATCHED++)) || true
grep -q "rsqrtf(float x) noexcept(true);" "$CUDA_HEADER" && ((PATCHED++)) || true
grep -q "sinpi(double x) noexcept(true);" "$CUDA_HEADER" && ((PATCHED++)) || true
grep -q "sinpif(float x) noexcept(true);" "$CUDA_HEADER" && ((PATCHED++)) || true
grep -q "cospi(double x) noexcept(true);" "$CUDA_HEADER" && ((PATCHED++)) || true
grep -q "cospif(float x) noexcept(true);" "$CUDA_HEADER" && ((PATCHED++)) || true

if [ "$PATCHED" -eq 6 ]; then
    echo "  All 6 patches verified ✓"
else
    echo "  WARNING: Only $PATCHED/6 patches found"
    echo "  Header may have already been patched or has different format"
fi

echo ""
echo "Testing NVCC compilation..."

# Create test file
cat > /tmp/test_cuda.cu << 'EOF'
#include <cmath>
__global__ void test_kernel() {}
int main() {
    test_kernel<<<1,1>>>();
    return 0;
}
EOF

# Test compilation
if /usr/local/cuda/bin/nvcc /tmp/test_cuda.cu -o /tmp/test_cuda 2>&1; then
    echo "  NVCC compilation successful ✓"
    rm -f /tmp/test_cuda /tmp/test_cuda.cu
    echo ""
    echo "==================================="
    echo "SUCCESS! CUDA headers patched."
    echo "==================================="
    echo ""
    echo "You can now rebuild the project:"
    echo "  cd /home/alex/Dev/Work/rzem-ai-inference/src-tauri"
    echo "  cargo clean"
    echo "  cargo build --release"
    echo ""
    echo "To restore original headers:"
    echo "  sudo cp $BACKUP_HEADER $CUDA_HEADER"
else
    echo "  NVCC compilation FAILED"
    echo ""
    echo "Something went wrong. Restoring backup..."
    cp "$BACKUP_HEADER" "$CUDA_HEADER"
    echo "Original headers restored."
    exit 1
fi
