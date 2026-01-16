# CUDA Downgrade Instructions: 13.1 → 12.6

## Why Downgrade?

CUDA 13.1 is too new (released late 2025) and has header incompatibilities with glibc. CUDA 12.6 is stable and well-supported by all Rust ML libraries.

## Option 1: Install CUDA 12.6 Alongside (Recommended)

This keeps both versions installed.

```bash
# Download CUDA 12.6
wget https://developer.download.nvidia.com/compute/cuda/12.6.3/local_installers/cuda_12.6.3_560.35.03_linux.run

# Install CUDA 12.6
sudo sh cuda_12.6.3_560.35.03_linux.run --silent --toolkit --override

# Switch default CUDA version
sudo update-alternatives --install /usr/local/cuda cuda /usr/local/cuda-12.6 126
sudo update-alternatives --set cuda /usr/local/cuda-12.6

# Verify
nvcc --version  # Should show CUDA 12.6
```

## Option 2: Remove CUDA 13.1 and Install 12.6

```bash
# Remove CUDA 13.1
sudo apt-get remove --purge '^cuda-13-1'
sudo apt-get remove --purge 'cuda'
sudo apt-get autoremove

# Clean up
sudo rm -rf /usr/local/cuda-13.1
sudo rm -rf /usr/local/cuda

# Download and install CUDA 12.6
wget https://developer.download.nvidia.com/compute/cuda/12.6.3/local_installers/cuda_12.6.3_560.35.03_linux.run
sudo sh cuda_12.6.3_560.35.03_linux.run

# Verify
nvcc --version  # Should show CUDA 12.6
```

## After CUDA Downgrade

1. **Update environment:**
   ```bash
   export PATH=/usr/local/cuda-12.6/bin:$PATH
   export LD_LIBRARY_PATH=/usr/local/cuda-12.6/lib64:$LD_LIBRARY_PATH
   ```

2. **Add to ~/.bashrc** (permanent):
   ```bash
   echo 'export PATH=/usr/local/cuda-12.6/bin:$PATH' >> ~/.bashrc
   echo 'export LD_LIBRARY_PATH=/usr/local/cuda-12.6/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
   source ~/.bashrc
   ```

3. **Enable CUDA in Cargo.toml:**
   ```toml
   candle-core = { version = "0.9.2-alpha.2", features = ["cuda"] }
   candle-nn = { version = "0.9.2-alpha.2", features = ["cuda"] }
   candle-transformers = { version = "0.9.2-alpha.2", features = ["cuda"] }
   ```

4. **Clean and rebuild:**
   ```bash
   cd src-tauri
   cargo clean
   cargo build --release
   ```

## Verification

```bash
# Check CUDA version
nvcc --version

# Check GPU is detected
nvidia-smi

# Test Rust CUDA build
cd src-tauri
cargo build --release 2>&1 | grep -i cuda
```

## Performance Impact

After enabling CUDA:
- **VAE decode**: ~3-5 minutes (CPU) → **< 1 second** (GPU)
- **CLIP encode**: ~5 seconds (CPU) → **< 0.1 seconds** (GPU)
- **Total generation**: ~3-5 minutes → **~2-3 seconds**

Your RTX 5090 has 32GB VRAM which is more than enough for FLUX!
