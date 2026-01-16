# FLUX Image Generator

AI-powered image generation desktop application using FLUX.1-schnell model.

## Features

- 🎨 Text-to-image generation with FLUX.1-schnell
- ⚡ Fast inference (4 steps optimized)
- 📊 Job queue with real-time progress tracking
- 🖼️ Gallery with favorites and tagging
- 🔍 Search and filter generated images
- 💾 SQLite-based metadata storage

## Prerequisites

- **Node.js** 18+ and npm
- **Rust** 1.70+ (for Tauri backend)
- **CUDA-capable GPU** (recommended) or CPU fallback
- **~24GB disk space** for FLUX model download
- **HuggingFace account** with access to FLUX.1-schnell model

## Setup

### 1. Clone and Install Dependencies

```bash
git clone <your-repo-url>
cd rzem-ai-inference
npm install
```

### 2. Configure Environment Variables

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit `.env` and add your HuggingFace token:

```env
HF_API_KEY=hf_your_token_here
```

**Get your HuggingFace token:**
1. Go to https://huggingface.co/settings/tokens
2. Create a new token with "Read" permissions
3. Accept the FLUX.1-schnell model license at https://huggingface.co/black-forest-labs/FLUX.1-schnell

### 3. Build and Run

**Development mode:**
```bash
npm run tauri dev
```

**Production build:**
```bash
npm run tauri build
```

## First Run

1. Launch the application
2. Navigate to **Models** tab
3. Click **Download FLUX Schnell** button
4. Wait for ~24GB download to complete (10-30 minutes depending on connection)
5. Go to **Generate** tab and create your first image!

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
- **Tauri API** for backend communication

### Backend (Rust + Tauri)
- **Candle** ML framework for inference
- **hf-hub** for model downloading
- **SQLite** for gallery metadata
- **tokio** async runtime

### Model Pipeline
1. **CLIP** text encoder (768-dim embeddings)
2. **FLUX Transformer** diffusion model (4 steps)
3. **VAE Decoder** latent-to-RGB conversion

## Project Structure

```
rzem-ai-inference/
├── src/                    # Frontend Vue code
│   ├── components/        # UI components
│   ├── stores/           # Pinia state stores
│   └── views/            # Main view pages
├── src-tauri/             # Backend Rust code
│   ├── src/
│   │   ├── inference/    # ML inference pipeline
│   │   ├── models/       # Model management
│   │   ├── queue/        # Job queue system
│   │   └── gallery/      # Image gallery DB
│   └── Cargo.toml        # Rust dependencies
├── docs/                  # Documentation
│   └── plans/            # Implementation plans
└── .env                   # API keys (gitignored)
```

## Development

### Running Tests

```bash
# Frontend tests
npm test

# Backend tests
cd src-tauri
cargo test
```

### Code Quality

```bash
# Frontend linting
npm run lint

# Backend linting
cd src-tauri
cargo clippy
```

## Troubleshooting

### "Download failed" error
- Ensure your HF_API_KEY is set correctly in `.env`
- Check you've accepted the FLUX model license
- Verify internet connection and disk space

### "Pinia not initialized" error
- Fixed in latest version - update to latest commit

### GPU not detected
- Install CUDA toolkit if using NVIDIA GPU
- Application will fallback to CPU (slower)

### Out of memory
- FLUX requires ~12GB VRAM for inference
- Use smaller batch sizes or CPU fallback

## License

[Your License Here]

## Acknowledgments

- Black Forest Labs for FLUX.1-schnell model
- Hugging Face for model hosting
- Candle ML framework by Hugging Face
