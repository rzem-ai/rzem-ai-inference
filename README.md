# Rzem AI Inference

Desktop AI image generation application with a Python inference backend and modern Vue 3 frontend.

## What is Rzem AI Inference?

A native desktop application for AI-powered image generation. The app runs local inference using the [rzem-ai-inference-engine](../rzem-ai-inference-engine) and also supports cloud generation via FAL.ai. It provides a polished UI for creating, managing, and refining AI-generated images.

**Key Features:**
- Local AI image generation with automatic VRAM management
- FAL.ai cloud generation (FLUX.1 Dev/Pro, FLUX.2 Dev/Flex/Pro)
- Image gallery with folders and tagging
- Style presets with LoRA support
- AI chat assistant (Claude integration)
- CSV batch processing for bulk generation
- Remote inference server discovery (mDNS/DNS-SD)
- Rich prompt editing with Tiptap
- Real-time progress tracking with event-driven architecture
- Custom Glass theme UI with PrimeVue components

## Architecture

```
Python Backend (pywebview)  ←—js_api bridge—→  Vue 3 Frontend (Vite)
     │                                              │
     ├── main.py (entry point)                      ├── PrimeVue 4 (Glass theme)
     ├── backend/config.py                          ├── Tailwind CSS 4
     ├── backend/db/ (SQLite)                       ├── Tiptap (rich text)
     ├── backend/api/                               ├── Pinia (state)
     │   ├── system.py                              └── Vue Router (hash mode)
     │   ├── inference.py
     │   ├── bundles.py
     │   ├── gallery.py
     │   ├── styles.py
     │   ├── settings.py
     │   ├── chat.py
     │   ├── batch.py
     │   └── discovery.py
     ├── backend/services/
     │   ├── app_service.py
     │   ├── inference_service.py
     │   ├── inference_manager.py
     │   ├── remote_inference_service.py
     │   ├── chat_service.py
     │   └── discovery_service.py
     └── backend/bundles.py (model bundle definitions)

Dependencies:
     └── ../rzem-ai-inference-engine (editable install)
             └── InferenceEngine, JobParams, event types
```

## Prerequisites

- Python 3.13+
- Node.js 20+
- [uv](https://docs.astral.sh/uv/) (Python package manager)
- [rzem-ai-inference-engine](../rzem-ai-inference-engine) repository cloned as a sibling directory

## Setup

```bash
# One-time setup (creates .venv, installs Python + Node deps, installs engine)
bash scripts/install.sh
```

This script:
1. Creates a virtual environment with `uv venv --system-site-packages` (required for Linux pywebview/GTK bindings)
2. Runs `uv sync` to install all Python dependencies from `pyproject.toml` (including `rzem-ai-inference-engine` as an editable package)
3. Installs all frontend Node dependencies

## Development

**Two-terminal workflow (full backend + HMR):**

```bash
# Terminal 1: Vite dev server with Hot Module Replacement
cd frontend && npm run dev

# Terminal 2: pywebview window → localhost:1978 (real inference engine)
bash scripts/dev.sh
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
bash scripts/run.sh
```

This builds the frontend (if needed) and runs the app. The pywebview window loads built assets from `frontend/dist/`.

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
│   ├── bundles.py                           # Model bundle definitions (local + cloud)
│   ├── db/
│   │   ├── database.py                      # SQLite database (images, folders, tags, styles, settings, conversations)
│   │   └── schema.sql                       # Database schema
│   ├── api/
│   │   ├── combined.py                      # CombinedAPI (js_api object)
│   │   ├── system.py                        # SystemAPI (app lifecycle)
│   │   ├── inference.py                     # InferenceAPI (generation, jobs)
│   │   ├── bundles.py                       # BundlesAPI (model bundle management)
│   │   ├── gallery.py                       # GalleryAPI (images, folders, tags)
│   │   ├── styles.py                        # StylesAPI (style presets, LoRAs)
│   │   ├── settings.py                      # SettingsAPI (GPU info, cache, config)
│   │   ├── chat.py                          # ChatAPI (Claude AI assistant)
│   │   ├── batch.py                         # BatchAPI (CSV bulk generation)
│   │   └── discovery.py                     # DiscoveryAPI (remote server discovery)
│   └── services/
│       ├── app_service.py                   # App lifecycle (thread-safe)
│       ├── inference_service.py             # Local inference engine wrapper + event queue
│       ├── inference_manager.py             # Switches between local/remote inference
│       ├── remote_inference_service.py      # Remote inference via REST API
│       ├── inference_protocol.py            # Shared protocol definitions
│       ├── chat_service.py                  # Claude AI integration
│       └── discovery_service.py             # LAN server discovery (mDNS)
├── frontend/
│   ├── src/
│   │   ├── main.ts                          # Vue + PrimeVue + Pinia + Router
│   │   ├── bridge.ts                        # pywebview readiness + mock API
│   │   ├── composables/usePywebview.ts      # Reactive bridge composable
│   │   ├── stores/
│   │   │   ├── app.ts                       # App state
│   │   │   ├── inference.ts                 # Inference state, job lifecycle, event polling
│   │   │   ├── gallery.ts                   # Image gallery, folders, tags
│   │   │   ├── styles.ts                    # Style presets, LoRAs
│   │   │   ├── settings.ts                  # Settings, GPU info
│   │   │   ├── chat.ts                      # Conversations, messages
│   │   │   └── discovery.ts                 # Remote server discovery
│   │   ├── types/pywebview.d.ts             # TypeScript API definitions
│   │   ├── theme/                           # Custom Glass theme preset
│   │   ├── router/index.ts                  # Hash-mode router
│   │   ├── components/                      # Shared components
│   │   └── pages/
│   │       ├── create/                      # Image generation UI
│   │       ├── gallery/                     # Image gallery with folders/tags
│   │       ├── styles/                      # Style preset management
│   │       └── settings/                    # App settings (API keys, GPU, cache, remote servers)
│   └── ...
├── scripts/
│   ├── install.sh                           # Setup: venv + deps + frontend
│   ├── dev.sh                               # Run in dev mode (DEV_MODE=1)
│   ├── run.sh                               # Run production build
│   ├── build.sh                             # PyInstaller build
│   └── package.sh                           # Build + create release archive
└── README.md
```

## Key Integration Details

### Frontend ↔ Backend Communication

- **pywebview bridge**: `bridge.ts` waits for `pywebviewready` event before calling Python APIs
- **API convention**: All Python methods return `{"status": "success", ...}` or `{"status": "error", "message": "..."}`
- **Argument marshalling**: `ApiMeta` metaclass auto-converts camelCase → snake_case and unpacks JS objects as `**kwargs`
- **Thread safety**: js_api calls run on separate Python threads; `threading.Lock` guards shared state

### Inference Engine Integration

- **Local + remote**: `InferenceManager` switches between `LocalInferenceService` and `RemoteInferenceService`
- **FAL.ai cloud**: Cloud bundles inject the API key from settings into `JobParams` before submission
- **Event-driven**: Inference engine fires events from background threads
- **Event polling**: `InferenceService` queues events (max 500); frontend polls `poll_events()` every 200ms
- **Image handling**: PIL images → saved as PNG files → frontend receives paths → calls `get_image_base64()` to load
- **Job lifecycle**: Queue → Running → Complete/Failed (tracked in `stores/inference.ts`)

### UI Stack

- **Tailwind 4 + PrimeVue 4**: `@plugin "tailwindcss-primeui"` + CSS layer ordering in `main.ts`
- **Custom Glass theme**: PrimeVue preset in `frontend/src/theme/`
- **Tiptap editor**: Rich text prompt input with formatting support
- **Hash router**: Required for production — pywebview's HTTP server has no SPA fallback

### Platform-Specific

- **Linux GTK bindings**: `.venv` created with `uv venv --system-site-packages` for `gi` module
- **CUDA teardown**: App uses `os._exit(0)` to avoid C++ errors from non-main thread GPU cleanup
- **Port 1978**: Hardcoded in `vite.config.ts` (strictPort) and `backend/config.py`
- **Database migrations**: Use proper migration strategies for schema changes

## Common Commands

```bash
# Frontend type checking
cd frontend && npm run type-check

# Frontend production build
cd frontend && npm run build

# Run production build locally
bash scripts/run.sh

# Build distributable executable
bash scripts/build.sh

# Run any Python command in the project venv
uv run python main.py
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

## Troubleshooting

### `ModuleNotFoundError: No module named 'gi'` (Linux)

The virtual environment needs `--system-site-packages` for GTK bindings. Recreate it:

```bash
rm -rf .venv
uv venv --system-site-packages --python /usr/bin/python3 .venv
source .venv/bin/activate
uv sync
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
