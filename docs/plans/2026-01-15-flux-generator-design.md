# Flux Generator - Full Design Specification

**Version**: 1.0
**Date**: 2026-01-15
**Status**: Approved for Implementation

## Overview

A desktop application for local AI image generation using Flux models via Candle, with server mode for distributed inference. Built with Tauri 2 + Vue 3 + Rust.

## Tech Stack

- **Frontend**: Vue 3, TypeScript, PrimeVue, TailwindCSS, Pinia, Vue Router
- **Backend**: Rust with Candle for ML inference
- **Threading**: Tokio for async operations, separate thread pool for Candle inference
- **Storage**: SQLite for metadata, filesystem for images
- **API**: Tauri commands + REST/WebSocket for server mode

## Architecture

```
┌─────────────────────────────────────────────────┐
│           Vue 3 Frontend (UI Layer)             │
│  ┌─────────┬──────────┬─────────┬────────────┐ │
│  │Generate │  Refine  │ Compare │   Manage   │ │
│  │Workspace│Workspace │Workspace│ Workspace  │ │
│  └─────────┴──────────┴─────────┴────────────┘ │
└────────────────────┬────────────────────────────┘
                     │ Tauri Commands
┌────────────────────▼────────────────────────────┐
│         Rust Backend (Tauri Main Process)       │
│  ┌──────────────┬─────────────┬──────────────┐ │
│  │ Command      │  Queue      │   Model      │ │
│  │ Handlers     │  Manager    │   Manager    │ │
│  └──────────────┴─────────────┴──────────────┘ │
│  ┌──────────────────────────────────────────┐  │
│  │     Inference Thread Pool (Candle)       │  │
│  │  • Model Loading  • Generation           │  │
│  │  • Progress Callbacks • Memory Mgmt      │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## Core Features

### 1. Image Generation Modes
- **Text-to-Image**: Generate from text prompts
- **Image-to-Image**: Transform existing images
- **Inpainting**: Edit specific regions with advanced mask editor

### 2. Model Support
- **Flux Schnell**: Fast 4-step model (12GB, Apache 2.0)
- **Flux Dev**: High-quality model (24GB, non-commercial)
- **Flux Pro**: Remote API integration for best quality
- **LoRA Support**: Apply multiple LoRAs with individual weights

### 3. Operating Modes
- **Local Mode**: All inference on local GPU/CPU
- **Server Mode**: Share GPU with remote clients via REST/WebSocket API
- **Client Mode**: Connect to remote server, full UI with delegated inference

### 4. Advanced Features
- Live preview during generation (progressive updates)
- Smart queue with batch support for similar prompts
- Multi-LoRA mixing with weight control
- Preset system for generation settings
- Smart gallery with full-text search and tagging
- Real-time GPU/memory monitoring with auto-recovery
- Integrated model hub with download manager

## Workspace Organization

### Generate Workspace (Primary creation mode)

**Left Panel (35%)**:
- Mode selector: txt2img / img2img / inpainting
- Prompt/negative prompt textareas
- Parameter controls: steps, CFG, dimensions, seed, batch
- Model selector and LoRA panel
- Generate button with queue count

**Center Panel (25%)**:
- Generation queue with live progress
- Recent history thumbnails
- Quick actions: favorite, compare, gallery

**Right Canvas (40%)**:
- Large preview/editing area
- Live progressive preview during generation
- For img2img: upload + strength slider
- For inpainting: canvas editor with brush/magic select/gradient tools
- Zoom/pan controls, metadata overlay

### Refine Workspace (Models & LoRAs)

**Model Hub**:
- Tabs: "My Models" / "Browse HuggingFace" / "Browse Civitai"
- Card-based layout with thumbnails, sizes, licenses
- Download manager with pause/resume
- Storage overview and cleanup tools

**LoRA Library**:
- Grid view with preview images
- Metadata: trigger words, weights, tags
- Search and filter functionality
- LoRA combination presets
- Import from filesystem or download

### Compare Workspace (History & Gallery)

**Smart Gallery**:
- Masonry/grid view of generated images
- Full-text search on prompts (SQLite FTS5)
- Filters: date, model, favorites, tags, dimensions
- Bulk operations: tag, favorite, export, delete

**Compare Mode**:
- Side-by-side view (up to 4 images)
- Synchronized zoom/pan
- Metadata diff showing parameter differences
- "Generate variation" from selected settings

### Manage Workspace (Settings & System)

**Performance Tab**:
- Real-time GPU/CPU/RAM monitoring
- Device selection (CUDA/Metal/CPU)
- Memory limits and precision settings (f32/f16/bf16)
- Benchmark tool for different configs

**Connection Tab** (Server Mode):
- Mode selector: Local / Server / Client
- Server settings: port, auth token, active connections
- Client settings: server discovery, manual entry, connection status
- Fallback options

**Storage & Paths**:
- Output directory, model cache, gallery database
- Cleanup tools for unused models and temp files

**Presets**:
- Named preset library (Quick Draft, Final Quality, etc.)
- Import/export as JSON

## Technical Implementation

### Rust Module Structure

```
src-tauri/src/
├── lib.rs                 # Main entry, command registration
├── inference/
│   ├── mod.rs            # Public interface
│   ├── engine.rs         # Candle inference engine
│   ├── pipeline.rs       # Flux pipeline implementations
│   ├── progress.rs       # Progress callback system
│   └── scheduler.rs      # Sampling schedulers
├── models/
│   ├── mod.rs
│   ├── manager.rs        # Model loading/caching/LRU
│   ├── downloader.rs     # HuggingFace Hub downloads
│   └── flux.rs           # Flux model structures
├── queue/
│   ├── mod.rs
│   ├── manager.rs        # Queue management & batching
│   └── job.rs            # Job definitions
├── gallery/
│   ├── mod.rs
│   ├── db.rs             # SQLite operations
│   └── metadata.rs       # Image metadata extraction
├── lora/
│   ├── mod.rs
│   ├── loader.rs         # LoRA weight loading
│   └── mixer.rs          # Multi-LoRA combination
├── server/
│   ├── mod.rs
│   ├── api.rs            # REST endpoints (axum)
│   ├── websocket.rs      # WebSocket handler
│   ├── auth.rs           # Token-based auth
│   └── discovery.rs      # mDNS server advertisement
├── client/
│   ├── mod.rs
│   ├── connector.rs      # Connect to remote server
│   └── fallback.rs       # Local fallback logic
└── utils/
    ├── image.rs          # Image processing utilities
    └── monitoring.rs     # GPU/memory monitoring
```

### Frontend Structure

```
src/
├── main.ts               # App initialization
├── App.vue               # Root component
├── router/
│   └── index.ts         # Route definitions
├── stores/
│   ├── generation.ts    # Generation state & queue
│   ├── gallery.ts       # Gallery & search
│   ├── models.ts        # Model & LoRA management
│   └── settings.ts      # App settings & connection
├── views/
│   ├── GenerateView.vue
│   ├── RefineView.vue
│   ├── CompareView.vue
│   └── ManageView.vue
├── components/
│   ├── generation/
│   ├── gallery/
│   ├── models/
│   └── shared/
└── types/
    └── index.ts         # TypeScript definitions
```

### Key Technologies

**Rust Dependencies**:
```toml
candle-core = { version = "0.8", features = ["cuda"] }
candle-nn = "0.8"
candle-transformers = "0.8"
tokenizers = "0.15"
hf-hub = "0.3"
safetensors = "0.4"
image = "0.25"
axum = "0.7"           # For server mode
tokio-tungstenite = "0.21"  # WebSocket
```

**Vue Dependencies**:
- Already present from template: PrimeVue, TailwindCSS, Pinia, VueUse
- Additional: (TBD if needed)

### Threading & Performance

**Inference Threading**:
- Tauri main thread receives commands, returns immediately
- Queue manager spawns tokio tasks for each job
- Candle inference runs in `spawn_blocking` to avoid blocking async runtime
- Progress callbacks emit Tauri events to frontend (~2 updates/sec)

**Memory Management**:
- Model weights cached with LRU eviction
- Configurable precision (f32/f16/bf16)
- Auto-recovery: OOM → reduce precision → reduce batch → offload to CPU
- Real-time monitoring with warnings

**Smart Queueing**:
- Batch jobs with same model + similar dimensions
- Different seeds batched together efficiently
- Memory-aware batch sizing

### Database Schema

**SQLite Tables**:
- `images`: Main table with generation metadata
- `tags`, `image_tags`: Many-to-many tagging
- `image_loras`: Track which LoRAs were used
- `sessions`: Group related generations
- `presets`: Saved generation configurations
- `images_fts`: Full-text search (FTS5) on prompts

**Metadata in Images**:
- PNG tEXt chunks / JPEG EXIF
- Complete generation parameters embedded
- Allows external tools to read settings

### Server Mode Protocol

**REST API**:
```
POST   /api/generate          # Submit job
GET    /api/queue             # Queue status
DELETE /api/queue/{job_id}    # Cancel job
GET    /api/models            # Available models
GET    /api/performance       # Server metrics
GET    /api/result/{job_id}   # Download image
```

**WebSocket**:
```
ws://server:port/ws
- Auth: Bearer token
- Events: job_queued, progress_update, preview_frame, job_complete
- Bidirectional real-time communication
```

**Discovery**:
- mDNS/Bonjour for automatic server discovery on LAN
- Manual entry for remote servers
- Connection status with latency monitoring

## Storage & Paths

**Default Locations**:
```
~/.flux-generator/
├── models/              # Model weights
│   ├── flux-schnell/
│   ├── flux-dev/
│   └── downloads/       # Partial downloads (.part files)
├── loras/               # LoRA files
│   ├── {lora_name}/
│   │   ├── model.safetensors
│   │   ├── metadata.json
│   │   └── preview.png
│   └── presets/         # LoRA combinations
├── outputs/             # Generated images
│   └── YYYY-MM-DD/
├── thumbnails/          # 256x256 cached thumbnails
├── gallery.db           # SQLite database
└── config.json          # App configuration
```

## Implementation Phases

### Phase 1: Core Infrastructure
1. Set up Tauri + Vue project structure
2. Implement basic Candle integration
3. Create modular Rust backend structure
4. Set up SQLite gallery database
5. Implement basic UI with workspace navigation

### Phase 2: Generation Features
1. Text-to-image with Flux Schnell
2. Progressive preview system
3. Queue management
4. Image-to-image mode
5. Inpainting with mask editor

### Phase 3: Model & LoRA Management
1. Model manager with download system
2. LoRA loader and mixing
3. Model hub UI
4. LoRA library UI
5. Preset system

### Phase 4: Gallery & Compare
1. Smart gallery with search
2. Tagging system
3. Compare workspace
4. Export functionality
5. Metadata embedding

### Phase 5: Advanced Features
1. Flux Dev support
2. Multi-LoRA combinations
3. Performance monitoring
4. Auto-recovery system
5. Batch generation optimization

### Phase 6: Server Mode
1. REST API with axum
2. WebSocket for real-time updates
3. Server/Client mode switching
4. Authentication system
5. mDNS discovery

### Phase 7: Polish & Optimization
1. UI/UX refinements
2. Performance optimization
3. Error handling
4. Documentation
5. Testing

## Success Criteria

- Generate high-quality images locally with Flux models
- Responsive UI with live preview during generation
- Efficient multi-model and multi-LoRA support
- Server mode works reliably over network
- Gallery can handle thousands of images performantly
- Intuitive UX matching or exceeding web-based tools
- Professional-grade error recovery and monitoring

## Future Enhancements (Post-v1)

- ControlNet support
- Upscaling integration
- Animation/video generation
- Custom training UI
- Plugin system for extensions
- Cloud sync (if needed)
- Mobile companion app
