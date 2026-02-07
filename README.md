# FLUX Image Generator

AI-powered image generation desktop application using FLUX.1-schnell model with Python + pywebview.

## Features

- 🎨 Text-to-image generation with FLUX.1-schnell via Diffusers
- ⚡ Fast inference (4 steps optimized)
- 📊 Job queue with real-time progress tracking
- 🖼️ Gallery with favorites and tagging
- 🔍 Search and filter generated images
- 💾 SQLite-based metadata storage
- 🎯 LoRA support for model customization
- 🚀 GPU auto-detection (CUDA/MPS/CPU)

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + PrimeVue + TailwindCSS
- **Backend**: Python + pywebview + PyTorch + Diffusers
- **ML Pipeline**: FLUX.1-schnell from HuggingFace Hub
- **Database**: SQLite via aiosqlite

## Prerequisites

- **Python** 3.10+ (for backend)
- **Node.js** 18+ and npm (for frontend build)
- **GPU**: CUDA-capable NVIDIA GPU or Apple Silicon (M1/M2/M3) recommended
- **~20GB disk space** for FLUX model auto-download
- **~16-24GB VRAM** for GPU inference (CPU fallback available)

## Quick Start

### 1. Install Python Dependencies

```bash
# Create virtual environment (recommended)
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r src-python/requirements.txt
```

### 2. Build Frontend

```bash
# Install Node dependencies
npm install

# Build the frontend
npm run build
```

### 3. Run the Application

```bash
# Option 1: Using npm script (recommended)
npm start

# Option 2: Using helper script
./run-python.sh

# Option 3: Direct Python
python src-python/main.py

# With debug logging
npm run python:debug
```

## First Run

On first launch, FLUX.1-schnell model will automatically download from HuggingFace Hub (~20GB):
- Models are cached in `~/.cache/huggingface/hub/`
- Download takes 5-20 minutes depending on connection
- Requires ~20GB free disk space
- No HuggingFace token required (public model)

## Usage

### Generating Images

1. **Generate Tab**: Enter your prompt and click "Generate"
   - Prompt: Describe the image you want
   - Steps: 4 (default for Schnell)
   - Seed: -1 for random, or specify for reproducibility

2. **Queue Tab**: Monitor generation progress
   - View pending, running, and completed jobs
   - Cancel jobs if needed
   - Clear completed history

3. **Gallery Tab**: Browse generated images
   - View all generated images
   - Search by prompt
   - Favorite images
   - Add tags for organization
   - Delete unwanted images

## Architecture

### Frontend (Vue 3 + TypeScript)
- **Pinia** for state management
- **PrimeVue** for UI components
- **TailwindCSS** for styling
- **Backend Bridge** for pywebview/Tauri compatibility

### Backend (Python + pywebview)
- **PyTorch + Diffusers** for ML inference
- **HuggingFace Hub** for model auto-download
- **aiosqlite** for gallery metadata
- **asyncio** async runtime
- **pywebview** for desktop wrapper

### Model Pipeline (FLUX.1-schnell)
1. **Text Encoders** (T5-XXL + CLIP-L) for prompt understanding
2. **FLUX Transformer** diffusion model (4 steps, optimized)
3. **VAE Decoder** for latent-to-RGB conversion
4. **LoRA Support** for model customization

## Project Structure

```
rzem-ai-inference/
├── src/                    # Frontend Vue code
│   ├── components/        # UI components
│   ├── stores/           # Pinia state stores
│   ├── views/            # Main view pages
│   └── utils/
│       └── backend-bridge.ts  # Backend abstraction layer
├── src-python/            # Backend Python code
│   ├── main.py           # Entry point
│   ├── api.py            # API bridge (exposed to JS)
│   ├── inference/        # FLUX pipeline
│   ├── queue/            # Job queue system
│   ├── db/               # SQLite database
│   └── requirements.txt  # Python dependencies
├── dist/                  # Built frontend (after npm run build)
├── docs/                  # Documentation
├── MIGRATION_GUIDE.md    # Rust→Python migration guide
├── CLAUDE_PYTHON.md      # Python coding standards
└── run-python.sh         # Helper script to run app
```

## Development

### Running in Development Mode

```bash
# Build frontend + run with debug logging
npm run python:dev

# Or manually:
npm run build
python src-python/main.py --debug
```

### Running Tests

```bash
# Frontend tests
npm test

# Python tests (TODO: add pytest)
cd src-python
pytest
```

### Code Quality

```bash
# Frontend linting
npm run lint

# Python linting
cd src-python
ruff check .
mypy .
```

## Troubleshooting

### Models not downloading
- First run downloads ~20GB from HuggingFace Hub
- Check internet connection and disk space (~30GB free)
- Models cache in `~/.cache/huggingface/hub/`
- No API key required (FLUX.1-schnell is public)

### GPU not detected

**NVIDIA (CUDA):**
```bash
# Check CUDA availability
python -c "import torch; print(torch.cuda.is_available())"

# If False, reinstall PyTorch with CUDA
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu118
```

**Apple Silicon (MPS):**
```bash
# Check MPS availability
python -c "import torch; print(torch.backends.mps.is_available())"
```

Application automatically falls back to CPU if no GPU detected (slower but works).

### Out of memory
- FLUX requires ~16-24GB VRAM for GPU inference
- Reduce image resolution (1024→512) to use less memory
- CPU fallback available but very slow

### "No backend available" error
- Don't open `dist/index.html` directly in browser
- Must run via `python src-python/main.py` or `npm start`
- pywebview creates the necessary backend bridge

### Import errors
```bash
# Reinstall Python dependencies
pip install -r src-python/requirements.txt

# Or with virtual environment
python3 -m venv venv
source venv/bin/activate
pip install -r src-python/requirements.txt
```

## License

[Your License Here]

## Acknowledgments

- Black Forest Labs for FLUX.1-schnell model
- Hugging Face for model hosting
- Candle ML framework by Hugging Face
