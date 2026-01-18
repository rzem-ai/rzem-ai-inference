# CUDA 12.8 / glibc 2.40+ Header Conflict Fix

## Problem

When building Rust projects with CUDA support (e.g., using `candle` with the `cuda` feature) on systems with glibc 2.40 or newer (Ubuntu 24.10+, Fedora 41+, etc.), you may encounter compilation errors like:

```
/usr/include/x86_64-linux-gnu/bits/mathcalls.h(206): error: exception specification is incompatible with that of previous function "rsqrt"
/usr/include/x86_64-linux-gnu/bits/mathcalls.h(83): error: exception specification is incompatible with that of previous function "cospi"
/usr/include/x86_64-linux-gnu/bits/mathcalls.h(85): error: exception specification is incompatible with that of previous function "sinpi"
```

## Root Cause

glibc 2.40+ added new math functions (`rsqrt`, `rsqrtf`, `sinpi`, `sinpif`, `cospi`, `cospif`) with `noexcept(true)` exception specifications. CUDA 12.8's `math_functions.h` declares these same functions without the `noexcept(true)` specifier, causing a C++ exception specification mismatch.

**Affected configurations:**
- CUDA 12.8.x with glibc 2.40, 2.41, or 2.42
- Ubuntu 24.10, 25.04, or newer
- Fedora 41+
- Any distribution with glibc >= 2.40

## Solution

Patch the CUDA header file to add `noexcept(true)` to the conflicting function declarations.

### Automated Fix (Recommended)

Use the provided script in this repository:

```bash
sudo ./fix_cuda_glibc.sh
```

The script will:
1. Backup the original CUDA header
2. Patch all 6 conflicting function declarations
3. Verify the patches were applied
4. Test NVCC compilation
5. Restore the backup if anything fails

### Manual Fix

If you prefer to patch manually:

```bash
# Backup the original header
sudo cp /usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h \
       /usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h.bak

# Apply patches
sudo sed -i 's/rsqrt(double x);$/rsqrt(double x) noexcept(true);/' \
    /usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h

sudo sed -i 's/rsqrtf(float x);$/rsqrtf(float x) noexcept(true);/' \
    /usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h

sudo sed -i 's/sinpi(double x);$/sinpi(double x) noexcept(true);/' \
    /usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h

sudo sed -i 's/sinpif(float x);$/sinpif(float x) noexcept(true);/' \
    /usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h

sudo sed -i 's/cospi(double x);$/cospi(double x) noexcept(true);/' \
    /usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h

sudo sed -i 's/cospif(float x);$/cospif(float x) noexcept(true);/' \
    /usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h
```

### Verify the Fix

Test that NVCC compiles successfully:

```bash
echo '__global__ void test() {} int main() { return 0; }' > /tmp/test.cu
nvcc /tmp/test.cu -o /tmp/test && echo "NVCC works!"
```

## Additional Requirements

### gcc/g++ Version Alignment

CUDA requires both `gcc` and `g++` to be the same version. If you see errors like:

```
gcc: fatal error: cannot execute 'cc1plus': posix_spawnp: No such file or directory
```

Check your compiler versions:

```bash
gcc --version
g++ --version
```

If they differ, align them (CUDA 12.8 supports gcc 11-14):

```bash
sudo apt install gcc-14 g++-14
sudo ln -sf /usr/bin/gcc-14 /usr/bin/gcc
sudo ln -sf /usr/bin/g++-14 /usr/bin/g++
```

## Restoring Original Headers

If you need to restore the original CUDA headers (e.g., before a CUDA update):

```bash
sudo cp /usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h.bak \
       /usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h
```

## Note on CUDA Updates

After updating CUDA, you may need to reapply this patch. The `fix_cuda_glibc.sh` script is idempotent and safe to run multiple times.

## Future Resolution

This issue is expected to be fixed in a future CUDA release (likely CUDA 13.2+). Once NVIDIA releases an update with glibc 2.40+ compatibility, this patch will no longer be necessary.

## References

- [NVIDIA Developer Forums: glibc 2.40 compatibility](https://forums.developer.nvidia.com/t/cuda-12-5-1-math-functions-h-incompatible-exception-specification/297778)
- glibc commit adding `rsqrt`, `sinpi`, `cospi`: glibc 2.40 release notes
- Affected CUDA header: `/usr/local/cuda/targets/x86_64-linux/include/crt/math_functions.h`

## Environment Tested

- Ubuntu 25.04 (glibc 2.42)
- CUDA 12.8.1
- gcc/g++ 14.3.0
- candle 0.8.4 with `cuda` feature
- RTX 5090 (compute capability 12.0)
