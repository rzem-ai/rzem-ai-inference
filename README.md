# pywebview + Vue 3 + Tailwind CSS 4 + PrimeVue 4

Desktop application template combining a Python backend with a modern Vue 3 frontend.

## Architecture

```
Python Backend (pywebview)  ←—js_api bridge—→  Vue 3 Frontend (Vite)
     │                                              │
     ├── main.py (entry point)                      ├── PrimeVue 4 (Aura theme)
     ├── backend/config.py                          ├── Tailwind CSS 4
     ├── backend/api/system.py (bridge)             ├── Pinia (state)
     └── backend/services/app_service.py            └── Vue Router (hash mode)
```

## Prerequisites

- Python 3.13+
- Node.js 20+

## Setup

```bash
# Install Python dependencies
pip install -r requirements.txt

# Install frontend dependencies
cd frontend && npm install
```

## Development

Run two terminals:

```bash
# Terminal 1: Vite dev server (HMR)
cd frontend && npm run dev

# Terminal 2: pywebview window pointing at dev server
DEV_MODE=1 python main.py
```

Or open `http://localhost:1978` in a browser — a mock API simulates the Python backend.

## Production

```bash
cd frontend && npm run build
cd ..
python main.py
```

The pywebview window loads built assets from `frontend/dist/`.

## Project Structure

```
├── main.py                         # Python entry point
├── backend/
│   ├── config.py                   # Dev/prod mode, window settings
│   ├── api/system.py               # js_api bridge (thin facade)
│   └── services/app_service.py     # Business logic (thread-safe)
├── frontend/
│   ├── src/
│   │   ├── main.ts                 # Vue + PrimeVue + Pinia + Router
│   │   ├── bridge.ts               # pywebview readiness + mock fallback
│   │   ├── composables/usePywebview.ts  # Reactive bridge composable
│   │   ├── stores/app.ts           # Pinia store
│   │   ├── router/index.ts         # Hash-mode router
│   │   ├── components/             # Shared components
│   │   └── pages/                  # Route pages
│   └── ...
└── README.md
```

## Key Integration Details

- **Tailwind 4 + PrimeVue**: `@plugin "tailwindcss-primeui"` in CSS + CSS layer ordering in `main.ts`
- **pywebview bridge**: `bridge.ts` waits for `pywebviewready` event before calling Python APIs
- **Hash router**: Required for production — pywebview's HTTP server has no SPA fallback
- **Thread safety**: js_api calls run on separate Python threads; `threading.Lock` guards shared state
