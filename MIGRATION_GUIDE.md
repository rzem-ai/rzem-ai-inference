# Migration Guide: Rust/Tauri → Python/pywebview

This document describes the changes made to port the backend from Rust + Tauri to Python + pywebview.

## Summary of Changes

### Backend (Rust → Python)

| Component | Old (Rust/Tauri) | New (Python/pywebview) |
|-----------|------------------|------------------------|
| Desktop Wrapper | Tauri 2 | pywebview 5+ |
| ML Framework | Candle | PyTorch + Diffusers |
| Async Runtime | Tokio | asyncio |
| IPC | Tauri commands | pywebview JS API |
| Events | Tauri event system | Polling-based events |
| GPU Support | Feature flags (cuda/metal) | PyTorch auto-detection |

### Frontend Changes (Minimal)

The frontend changes are minimal thanks to the `backend-bridge.ts` compatibility layer:

1. **New file**: `src/utils/backend-bridge.ts` - Abstracts Tauri vs pywebview
2. **Store updates**: Replace `@tauri-apps/api/core` imports with `backend-bridge`
3. **Event handling**: Works the same, but uses polling behind the scenes for pywebview

## Directory Structure

```
rzem-ai-inference/
├── src-python/              # New Python backend
│   ├── main.py             # Entry point (replaces src-tauri/src/main.rs)
│   ├── api.py              # API bridge (replaces Tauri commands)
│   ├── app_state.py        # Application state
│   ├── events.py           # Event queue for polling
│   ├── inference/          # Image generation
│   │   ├── device.py       # GPU/CPU selection
│   │   └── flux_pipeline.py # FLUX pipeline
│   ├── queue/              # Job queue
│   │   ├── types.py        # Job types
│   │   ├── manager.py      # Queue manager
│   │   └── processor.py    # Job processor
│   ├── db/                 # Database
│   │   └── database.py     # SQLite operations
│   ├── shared/             # Shared types
│   │   └── protocol.py     # Runtime config
│   └── requirements.txt    # Dependencies
├── src-tauri/              # Original Rust backend (can be kept or removed)
└── src/                    # Frontend (minimal changes)
    └── utils/
        └── backend-bridge.ts  # NEW: Backend abstraction
```

## Installation & Setup

### 1. Install Python Dependencies

```bash
cd src-python
pip install -r requirements.txt
```

Or install as a package:

```bash
pip install -e src-python
```

### 2. Build Frontend (if not already built)

```bash
npm install
npm run build
```

### 3. Run the Application

```bash
python src-python/main.py
```

Or if installed:

```bash
rzem-ai
```

## Frontend Migration Steps

### Step 1: Update Tauri Imports

**Before:**
```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
```

**After:**
```typescript
import { invoke, listen } from '@/utils/backend-bridge';
```

### Step 2: Update Store Files

Example for `src/stores/generation.ts`:

```diff
- import { invoke } from '@tauri-apps/api/core';
+ import { invoke } from '@/utils/backend-bridge';
```

The API calls remain the same:

```typescript
// Still works the same
const result = await invoke('queue_generation', { params });
```

### Step 3: Update Event Listeners

No changes needed! The backend-bridge handles the differences:

```typescript
// This works with both Tauri and pywebview
const unlisten = await listen('job-progress', (payload) => {
  console.log('Progress:', payload);
});

// Call unlisten() when done
onUnmounted(() => unlisten());
```

### Step 4: Update Composables

Update any composables that import from Tauri:

**Before:**
```typescript
import { invoke } from '@tauri-apps/api/core';
```

**After:**
```typescript
import { invoke } from '@/utils/backend-bridge';
```

## API Mapping

### Commands (Rust → Python)

| Rust Command | Python Method | Notes |
|--------------|---------------|-------|
| `queue_generation` | `queue_generation()` | Same signature |
| `get_all_jobs` | `get_all_jobs()` | Same signature |
| `get_job` | `get_job()` | Same signature |
| `cancel_job` | `cancel_job()` | Same signature |
| `clear_completed_jobs` | `clear_completed_jobs()` | Same signature |
| `get_all_images` | `get_all_images()` | Same signature |
| `get_image_by_id` | `get_image_by_id()` | Same signature |
| `delete_image` | `delete_image()` | Same signature |
| `toggle_favorite` | `toggle_favorite()` | Same signature |
| `init_database` | `init_database()` | Same signature |
| `health_check` | `health_check()` | Same signature |

### Events

Events work the same way:

- `job-queued` - Job added to queue
- `job-progress` - Generation progress update
- `job-completed` - Job finished
- `job-failed` - Job failed
- `job-cancelled` - Job cancelled

**Implementation difference:**
- Tauri: Native WebSocket-based events
- pywebview: Polling every 100ms via `poll_events()`

The frontend code doesn't need to know the difference!

## Running in Development

### Option 1: Integrated (Python backend with built frontend)

```bash
# Build frontend once
npm run build

# Run Python backend
python src-python/main.py --debug
```

### Option 2: Separate Dev Servers (for hot reload)

This requires modifying `main.py` to load from `http://localhost:5173` instead of `dist/index.html`.

```bash
# Terminal 1: Frontend dev server
npm run dev

# Terminal 2: Python backend (modify main.py first)
python src-python/main.py --debug
```

## Testing Checklist

After migration, test these workflows:

- [ ] Health check on startup
- [ ] Database initialization
- [ ] Queue a generation job
- [ ] Monitor job progress
- [ ] View completed images in gallery
- [ ] Delete an image
- [ ] Toggle favorite
- [ ] Cancel a pending job
- [ ] Clear completed jobs
- [ ] Event polling works
- [ ] GPU detection (check logs)
- [ ] Model download (first run)

## Known Differences

### 1. Model Storage

- **Rust/Candle**: Models in `~/.cache/huggingface/` (safetensors)
- **Python/Diffusers**: Models in `~/.cache/huggingface/hub/` (auto-managed)

Models will be re-downloaded on first run with Python backend.

### 2. Event System

- **Rust/Tauri**: Real-time WebSocket events
- **Python/pywebview**: Polling-based (100ms intervals)

The polling adds slight latency but is acceptable for this use case.

### 3. Performance

- **Rust/Candle**: Lower memory overhead, faster startup
- **Python/PyTorch**: Slightly higher memory, but mature ecosystem

Image generation speed should be similar (both use GPU).

### 4. Server/Client Mode

Not yet implemented in Python backend. Local mode only for now.

## Troubleshooting

### "No backend available" error

Make sure `window.pywebview` is defined. Check browser console.

### Models not loading

Check `~/.cache/huggingface/` has enough disk space. First download can be 20GB+.

### GPU not detected

Check PyTorch installation:

```python
import torch
print(torch.cuda.is_available())  # Should be True for CUDA
print(torch.backends.mps.is_available())  # Should be True for Apple Silicon
```

### Events not updating

Check that `poll_events()` is being called. Look for errors in console.

## Rollback Plan

To revert to the Rust backend:

1. Restore Tauri imports in frontend:
   ```bash
   git checkout src/stores src/composables
   ```

2. Run with Tauri:
   ```bash
   npm run tauri:dev
   ```

The `backend-bridge.ts` is designed to work with both backends, so you can switch back and forth.

## Future Work

- [ ] Implement server mode (FastAPI + WebSocket)
- [ ] Implement client mode
- [ ] Add WebSocket-based events (replace polling)
- [ ] Port all Rust commands to Python API
- [ ] Add model download progress UI
- [ ] Optimize memory usage
- [ ] Add unit tests
- [ ] Add integration tests
- [ ] Performance benchmarking

## References

- InvokeAI: https://github.com/invoke-ai/InvokeAI (Python SD implementation)
- Diffusers FLUX: https://huggingface.co/docs/diffusers/api/pipelines/flux
- pywebview API: https://pywebview.flowrl.com/guide/api.html
