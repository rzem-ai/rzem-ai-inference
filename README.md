# Rzem AI Inference

Desktop AI image generation application with a Python inference backend and modern Vue 3 frontend.

## What is Rzem AI Inference?

A native desktop application for AI-powered image generation. The app runs local inference using the [rzem-ai-inference-engine](../rzem-ai-inference-engine) and provides a polished UI for creating, managing, and refining AI-generated images.

**Key Features:**
- Local AI image generation (no cloud dependencies)
- Real-time progress tracking with event-driven architecture
- Rich prompt editing with Tiptap
- Job queue management
- Custom Glass theme UI with PrimeVue components

## Architecture

```
Python Backend (pywebview)  ←—js_api bridge—→  Vue 3 Frontend (Vite)
     │                                              │
     ├── main.py (entry point)                      ├── PrimeVue 4 (Glass theme)
     ├── backend/config.py                          ├── Tailwind CSS 4
     ├── backend/api/                               ├── Tiptap (rich text)
     │   ├── system.py                              ├── Pinia (state)
     │   └── inference.py (generation API)          └── Vue Router (hash mode)
     └── backend/services/
         ├── app_service.py
         └── inference_service.py (engine wrapper)

Dependencies:
     └── ../rzem-ai-inference-engine (editable install)
             └── InferenceEngine, JobParams, event types
```

## Prerequisites

- Python 3.13+
- Node.js 20+
- [rzem-ai-inference-engine](../rzem-ai-inference-engine) repository cloned as a sibling directory

## Setup

```bash
# One-time setup (creates .venv, installs Python + Node deps, installs engine)
bash scripts/install.sh
```

This script:
1. Creates a virtual environment with `--system-site-packages` (required for Linux pywebview/GTK bindings)
2. Installs the sibling `rzem-ai-inference-engine` as an editable package
3. Installs all Python dependencies
4. Installs all frontend Node dependencies

## Development

**Two-terminal workflow (full backend + HMR):**

```bash
# Terminal 1: Vite dev server with Hot Module Replacement
cd frontend && npm run dev

# Terminal 2: pywebview window → localhost:1978 (real inference engine)
DEV_MODE=1 python main.py
```

**Browser-only development (frontend work, no GPU needed):**

```bash
cd frontend && npm run dev
# Open http://localhost:1978 in browser — mock API simulates inference
```

The mock API provides fake generation responses, allowing UI development without running the inference engine.

## Production

**Run from source:**

```bash
cd frontend && npm run build
cd ..
python main.py
```

The pywebview window loads built assets from `frontend/dist/`.

**Build distributable:**

```bash
bash scripts/build.sh
```

Creates a PyInstaller executable in `build/dist/Inference/` with all dependencies bundled.

## Project Structure

```
├── main.py                                  # Python entry point
├── backend/
│   ├── config.py                            # Dev/prod mode, window settings
│   ├── api/
│   │   ├── combined.py                      # CombinedAPI (js_api object)
│   │   ├── system.py                        # SystemAPI (app lifecycle)
│   │   └── inference.py                     # InferenceAPI (generation, jobs)
│   └── services/
│       ├── app_service.py                   # App lifecycle (thread-safe)
│       └── inference_service.py             # Inference engine wrapper + event queue
├── frontend/
│   ├── src/
│   │   ├── main.ts                          # Vue + PrimeVue + Pinia + Router
│   │   ├── bridge.ts                        # pywebview readiness + mock API
│   │   ├── composables/usePywebview.ts      # Reactive bridge composable
│   │   ├── stores/
│   │   │   ├── app.ts                       # App state
│   │   │   └── inference.ts                 # Inference state, job lifecycle, event polling
│   │   ├── types/pywebview.d.ts             # TypeScript API definitions
│   │   ├── theme/                           # Custom Glass theme preset
│   │   ├── router/index.ts                  # Hash-mode router
│   │   ├── components/                      # Shared components
│   │   └── pages/
│   │       └── create/                      # Image generation UI
│   │           ├── Main.vue                 # Results gallery
│   │           └── Menu.vue                 # Generation parameters sidebar
│   └── ...
├── scripts/
│   ├── install.sh                           # Setup script
│   └── build.sh                             # PyInstaller build
└── README.md
```

## Key Integration Details

### Frontend ↔ Backend Communication

- **pywebview bridge**: `bridge.ts` waits for `pywebviewready` event before calling Python APIs
- **API convention**: All Python methods return `{"status": "success", ...}` or `{"status": "error", "message": "..."}`
- **Argument marshalling**: `ApiMeta` metaclass auto-converts camelCase → snake_case and unpacks JS objects as `**kwargs`
- **Thread safety**: js_api calls run on separate Python threads; `threading.Lock` guards shared state

### Inference Engine Integration

- **Event-driven**: Inference engine fires events (`progress_update`, `generation_complete`) from background threads
- **Event polling**: `InferenceService` queues events (max 500); frontend polls `poll_events()` every 200ms
- **Image handling**: PIL images → saved as PNG files → frontend receives paths → calls `get_image_base64()` to load
- **Job lifecycle**: Queue → Running → Complete/Failed (tracked in `stores/inference.ts`)

### UI Stack

- **Tailwind 4 + PrimeVue 4**: `@plugin "tailwindcss-primeui"` + CSS layer ordering in `main.ts`
- **Custom Glass theme**: PrimeVue preset in `frontend/src/theme/`
- **Tiptap editor**: Rich text prompt input with formatting support
- **Hash router**: Required for production — pywebview's HTTP server has no SPA fallback

### Platform-Specific

- **Linux GTK bindings**: `.venv` created with `--system-site-packages` for `gi` module
- **CUDA teardown**: App uses `os._exit(0)` to avoid C++ errors from non-main thread GPU cleanup
- **Port 1978**: Hardcoded in `vite.config.ts` (strictPort) and `backend/config.py`

## Common Commands

```bash
# Frontend type checking
cd frontend && npm run type-check

# Frontend production build
cd frontend && npm run build

# Run production build locally
python main.py

# Build distributable executable
bash scripts/build.sh

# Activate virtual environment manually
source .venv/bin/activate  # Linux/macOS
.venv\Scripts\activate     # Windows
```

## Development Tips

### Working on the Inference Engine

The inference engine is installed as an editable package from `../rzem-ai-inference-engine`. Changes to the engine code are immediately reflected without reinstalling.

### Frontend Development Without GPU

Use the mock API mode to develop UI features without needing a GPU or running the Python backend:

```bash
cd frontend && npm run dev
# Open browser to http://localhost:1978
```

The mock API in `bridge.ts` simulates:
- Image generation requests (returns fake job IDs)
- Progress events (synthetic progress updates)
- Job status queries (mock job data)

### Event Debugging

Enable event logging in the inference store to see real-time event flow:

```typescript
// frontend/src/stores/inference.ts
console.log('[Event]', event.type, event.data);
```

### Database Schema Changes

This app has not been released yet. For database schema changes, simply delete the database file and restart — no migration code needed.

## Troubleshooting

### `ModuleNotFoundError: No module named 'gi'` (Linux)

The virtual environment needs `--system-site-packages` for GTK bindings. Recreate it:

```bash
rm -rf .venv
python -m venv .venv --system-site-packages
source .venv/bin/activate
pip install -r requirements.txt
```

### Frontend can't connect to backend

Check that both services are running on the correct ports:
- Vite dev server: `http://localhost:1978`
- `DEV_MODE=1` must be set for Python to point to dev server

### Images not loading in UI

The frontend polls `poll_events()` for image paths, then calls `get_image_base64()`. Check:
1. Event polling is active (200ms interval in `stores/inference.ts`)
2. Image paths are valid (check `InferenceService` save location)
3. Browser console for base64 decode errors

### Build fails on Windows

PyInstaller may need additional hooks for CUDA libraries. Check `scripts/build.sh` for platform-specific bundling configuration.

## License

[Your license here]
