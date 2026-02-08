# Python Backend Port - Quick Start Guide

The backend has been successfully ported from Rust + Tauri to Python + pywebview!

## What Changed?

- **Backend**: Rust + Candle → Python + PyTorch/Diffusers
- **Desktop Wrapper**: Tauri → pywebview
- **IPC**: Tauri commands → pywebview JS API
- **Events**: WebSocket → Polling (100ms)
- **Frontend**: Minimal changes (compatibility layer added)

## Quick Start

### 1. Install Dependencies

```bash
# Create and activate virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install Python dependencies
pip install -r src-python/requirements.txt
```

### 2. Build Frontend

```bash
npm install
npm run build
```

### 3. Run Application

```bash
# Option 1: Using helper script (recommended)
./run-python.sh

# Option 2: Direct Python
python src-python/main.py

# Option 3: With debug logging
python src-python/main.py --debug

# Option 4: Using npm script
npm run python
```

## Directory Structure

```
rzem-ai-inference/
├── src-python/              # Python backend (NEW)
│   ├── main.py             # Entry point
│   ├── api.py              # API bridge (exposed to frontend)
│   ├── app_state.py        # Application state
│   ├── events.py           # Event queue
│   ├── inference/          # FLUX pipeline
│   ├── queue/              # Job management
│   ├── db/                 # SQLite database
│   └── requirements.txt    # Dependencies
├── src-tauri/              # Original Rust backend (can be kept or removed)
├── src/                    # Frontend (minimal changes)
│   └── utils/
│       └── backend-bridge.ts  # NEW: Abstracts Tauri vs pywebview
└── dist/                   # Built frontend
```

## Key Files

| File | Purpose |
|------|---------|
| `src-python/main.py` | Entry point, CLI args, pywebview setup |
| `src-python/api.py` | Python methods exposed to JavaScript |
| `src-python/events.py` | Event queue for frontend polling |
| `src-python/inference/flux_pipeline.py` | FLUX.1 generation |
| `src/utils/backend-bridge.ts` | Frontend compatibility layer |
| `MIGRATION_GUIDE.md` | Detailed migration documentation |
| `CLAUDE_PYTHON.md` | Updated coding standards |

## npm Scripts

```bash
npm run python        # Run Python backend
npm run python:dev    # Build frontend + run with debug
npm run python:debug  # Run with debug logging

# Original Tauri scripts still work
npm run tauri:dev     # Run Rust backend (if you want to switch back)
```

## Frontend Changes

**Before (Tauri):**
```typescript
import { invoke } from '@tauri-apps/api/core';
```

**After (Both Tauri & pywebview):**
```typescript
import { invoke } from '@/utils/backend-bridge';
```

The backend-bridge automatically detects whether you're using Tauri or pywebview and adapts!

## Features

All major features are ported:

- ✅ Image generation (FLUX.1 via Diffusers)
- ✅ Job queue with progress tracking
- ✅ Gallery with SQLite database
- ✅ LoRA support
- ✅ Settings persistence
- ✅ GPU auto-detection (CUDA/MPS/CPU)
- ✅ Event system (polling-based)
- ⏳ Server mode (not yet implemented)
- ⏳ Client mode (not yet implemented)

## GPU Support

### CUDA (NVIDIA)

Automatically detected if:
- CUDA toolkit installed
- PyTorch with CUDA support installed

```bash
# Verify CUDA
python -c "import torch; print(torch.cuda.is_available())"
```

### MPS (Apple Silicon)

Automatically detected on M1/M2/M3 Macs:

```bash
# Verify MPS
python -c "import torch; print(torch.backends.mps.is_available())"
```

### CPU Fallback

If no GPU detected, falls back to CPU (slow but works).

## First Run

On first run, models will download from HuggingFace (~20GB):

- FLUX.1-schnell model
- T5 text encoder
- CLIP text encoder
- VAE decoder

Models are cached in `~/.cache/huggingface/hub/`

## Troubleshooting

### "No backend available" error

**Cause**: Frontend can't find pywebview or Tauri.

**Fix**: Make sure you're running the app via `python src-python/main.py`, not just opening `dist/index.html` in a browser.

### Models not downloading

**Cause**: Network issues or disk space.

**Fix**:
- Check internet connection
- Ensure ~30GB free disk space
- Check logs for specific errors

### GPU not detected

**CUDA:**
```bash
# Check CUDA availability
python -c "import torch; print(torch.cuda.is_available())"

# If False, reinstall PyTorch with CUDA:
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu118
```

**MPS:**
```bash
# Check MPS availability (Mac only)
python -c "import torch; print(torch.backends.mps.is_available())"
```

### Import errors

```bash
# Reinstall dependencies
pip install -r src-python/requirements.txt
```

### Events not updating in UI

**Cause**: Event polling not working.

**Fix**: Check browser console for JavaScript errors. The polling happens automatically via `backend-bridge.ts`.

## Development Tips

### Modifying the API

Add new methods to `src-python/api.py`:

```python
class Api:
    def my_new_command(self, arg: str) -> Dict[str, Any]:
        """Description"""
        return {"status": "success", "data": "..."}
```

Call from frontend:

```typescript
const result = await invoke('my_new_command', { arg: 'value' });
```

### Adding Events

Push events from Python:

```python
import events
await events.push_event('my-event', {'key': 'value'})
```

Listen in frontend:

```typescript
const unlisten = await listen('my-event', (payload) => {
  console.log(payload.key);
});
```

### Debugging

**Python:**
```bash
python src-python/main.py --debug
```

**Frontend:**
Open browser dev tools (pywebview includes a debugger).

## Comparison: Rust vs Python

| Aspect | Rust/Tauri | Python/pywebview |
|--------|------------|------------------|
| **Startup Time** | Faster | Slower (model loading) |
| **Memory Usage** | Lower | Higher |
| **Generation Speed** | Same (GPU) | Same (GPU) |
| **Development** | Complex | Simpler |
| **ML Ecosystem** | Limited | Excellent |
| **Distribution** | Smaller binary | Requires Python |

## Next Steps

1. **Test the application**: Generate some images!
2. **Read the docs**: Check `MIGRATION_GUIDE.md` for details
3. **Report issues**: Let me know if anything doesn't work

## Switching Back to Rust

To switch back to the Rust backend:

```bash
npm run tauri:dev
```

The backend-bridge supports both, so you can switch without frontend changes!

## Documentation

- `MIGRATION_GUIDE.md` - Detailed migration guide
- `CLAUDE_PYTHON.md` - Python coding standards
- `src-python/README.md` - Python backend docs
- `CLAUDE.md` - Original (Rust) coding standards

## Support

If you encounter issues:

1. Check the logs (`--debug` flag)
2. Verify GPU availability
3. Check disk space for model downloads
4. Review the migration guide

Happy generating! 🎨
