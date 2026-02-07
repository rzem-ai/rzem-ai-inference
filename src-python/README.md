# RZEM AI Inference - Python Backend

This is the Python backend for RZEM AI Inference, ported from the original Rust + Tauri implementation.

## Architecture

- **Desktop UI**: pywebview (replaces Tauri)
- **ML Framework**: PyTorch + Diffusers (replaces Candle)
- **Image Generation**: FLUX.1 Schnell/Dev via Diffusers
- **Database**: SQLite via aiosqlite
- **Async Runtime**: asyncio (replaces Tokio)

## Installation

1. Install Python dependencies:

```bash
cd src-python
pip install -r requirements.txt
```

Or install as a package:

```bash
cd src-python
pip install -e .
```

2. Build the frontend (if not already built):

```bash
cd ..
npm install
npm run build
```

## Running

### Local Mode (Default)

Desktop app with local GPU inference:

```bash
python src-python/main.py
```

Or if installed:

```bash
rzem-ai
```

### Debug Mode

```bash
python src-python/main.py --debug
```

### Server Mode

Run as a server with REST API + WebSocket (not yet implemented):

```bash
python src-python/main.py --server --port 8080
```

### Client Mode

Connect to a remote server (not yet implemented):

```bash
python src-python/main.py --client --server-url http://192.168.1.100:8080
```

## GPU Support

- **CUDA**: Automatically used if available (NVIDIA GPUs)
- **MPS**: Automatically used on Apple Silicon (M1/M2/M3)
- **CPU**: Fallback if no GPU available (slow)

## Project Structure

```
src-python/
├── main.py              # Entry point
├── app_state.py         # Application state management
├── api.py               # pywebview API bridge (replaces Tauri commands)
├── inference/           # Image generation
│   ├── device.py        # GPU/CPU device selection
│   └── flux_pipeline.py # FLUX.1 pipeline
├── queue/               # Job queue management
│   ├── types.py         # Job types
│   ├── manager.py       # Queue manager
│   └── processor.py     # Job processor
├── db/                  # Database (gallery)
│   └── database.py      # SQLite operations
├── shared/              # Shared types
│   └── protocol.py      # Runtime config
└── requirements.txt     # Python dependencies
```

## API Bridge (pywebview)

The API class in `api.py` exposes methods to the JavaScript frontend via pywebview's JS API.

Frontend can call them using:

```javascript
// Example: Queue a generation
const result = await window.pywebview.api.queue_generation({
  prompt: "a beautiful sunset",
  steps: 4,
  width: 1024,
  height: 1024,
  seed: -1,
  model_component_id: "flux-schnell",
  t5_component_id: "t5-xxl",
  clip_component_id: "clip-l",
  vae_component_id: "vae",
});

console.log(result.job_id);
```

## Key Differences from Rust Backend

1. **Desktop Wrapper**: pywebview instead of Tauri
2. **ML Framework**: PyTorch/Diffusers instead of Candle
3. **IPC**: pywebview JS API instead of Tauri commands/events
4. **Async**: asyncio instead of Tokio
5. **Model Loading**: Diffusers auto-downloads from HuggingFace

## TODO

- [ ] Implement event system for real-time progress updates
- [ ] Add server mode with FastAPI/aiohttp
- [ ] Add client mode
- [ ] Port settings management
- [ ] Port model downloader UI
- [ ] Add batch processing support
- [ ] Port LoRA UI controls
- [ ] Add embedding cache
- [ ] Optimize memory usage
- [ ] Add tests

## Development

For development with hot reload:

```bash
# Terminal 1: Frontend dev server
npm run dev

# Terminal 2: Python backend
python src-python/main.py --debug
```

Note: You'll need to update the frontend to point to the dev server URL instead of loading from dist.

## Performance Notes

- First generation will be slow as models download from HuggingFace
- Models are cached in `~/.cache/huggingface/hub/`
- GPU memory usage: ~16-24GB for FLUX.1
- Generation time: ~2-10s on modern GPUs (depending on steps/resolution)
