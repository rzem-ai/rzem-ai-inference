# RZEM AI Inference - API Documentation

This document provides comprehensive API documentation for the Tauri IPC commands used between the Vue frontend and Rust backend.

## Overview

This is a **Tauri application** using IPC (Inter-Process Communication) rather than traditional REST APIs:
- Backend defines commands with `#[tauri::command]`
- Frontend calls them via `invoke('command_name', payload)`
- All communication is type-safe and serialized via Serde

## API Endpoints Summary

| Category | Commands |
|----------|----------|
| App Initialization | `health_check`, `init_database` |
| Generation Queue | `add_to_queue`, `get_queue_jobs`, `get_queue_job`, `cancel_queue_job`, `clear_completed_jobs` |
| Gallery | `get_gallery_images`, `search_gallery_images`, `toggle_favorite`, `add_image_tag`, `remove_image_tag`, `delete_gallery_image` |
| Models | `get_available_models`, `get_model_status`, `check_models_downloaded`, `download_flux_schnell`, `download_flux_dev` |
| Cache | `get_cache_stats`, `get_cache_config`, `set_cache_config`, `set_cache_preset`, `clear_model_cache` |
| API Keys | `get_hf_token`, `set_hf_token`, `get_claude_api_key`, `set_claude_api_key`, `get_fal_key`, `set_fal_key` |
| Image Analysis | `analyze_image_for_prompt`, `read_image_metadata` |

---

## Sequence Diagrams

### 1. Application Initialization Flow

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend

    Note over Vue,Rust: App Startup (useAppInit.ts)

    Vue->>Rust: invoke('health_check')
    Note right of Vue: No payload
    Rust-->>Vue: 200 OK
    Note left of Rust: Response:<br/>"Rust backend is healthy!"

    Vue->>Rust: invoke('init_database', payload)
    Note right of Vue: Request:<br/>{ "dbPath": "/path/to/gallery.db" }

    alt Success
        Rust-->>Vue: Ok(String)
        Note left of Rust: Response:<br/>"Database initialized"
    else Error
        Rust-->>Vue: Err(String)
        Note left of Rust: Error:<br/>"Failed to initialize: <reason>"
    end

    Vue->>Rust: invoke('get_available_models')
    Note right of Vue: No payload
    Rust-->>Vue: Ok(Vec<ModelAvailability>)
    Note left of Rust: Response:<br/>[{<br/>  "id": "flux-schnell",<br/>  "name": "FLUX.1 [schnell]",<br/>  "is_downloaded": true,<br/>  "has_quantized": true<br/>}]
```

---

### 2. Image Generation Queue Flow

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend
    participant Events as Tauri Events

    Note over Vue,Rust: Add Job to Queue (stores/queue.ts)

    Vue->>Rust: invoke('add_to_queue', { params })
    Note right of Vue: Request:<br/>{<br/>  "params": {<br/>    "prompt": "A White West Highland White Terrier in the style of Pixar",<br/>    "negative_prompt": "",<br/>    "model": "flux-schnell",<br/>    "sampler": "euler",<br/>    "scheduler": "simple",<br/>    "steps": 4,<br/>    "cfg_scale": 1.0,<br/>    "width": 1024,<br/>    "height": 1024,<br/>    "seed": 42,<br/>    "number_of_images": "2"  }<br/>}

    Rust-->>Vue: Ok(String)
    Note left of Rust: Response:<br/>"job_abc123" (job ID)

    loop Job Progress Events
        Events-->>Vue: emit('job-update', JobUpdate)
        Note left of Events: Event:<br/>{<br/>  "jobId": "job_abc123",<br/>  "status": "processing",<br/>    "stages": 2,<br/>  "progress": 50,<br/>  "step": 2,<br/>  "totalSteps": 4<br/>}
    end

    Events-->>Vue: emit('job-update', JobComplete)
    Note left of Events: Event:<br/>{<br/>  "jobId": "job_abc123",<br/>  "status": "completed",<br/>  "outputPath": "/path/to/image.png"<br/>}
```

---

### 3. Queue Management Flow

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend

    Note over Vue,Rust: Get All Jobs
    Vue->>Rust: invoke('get_queue_jobs')
    Note right of Vue: No payload
    Rust-->>Vue: Ok(Vec<GenerationJob>)
    Note left of Rust: Response:<br/>[{<br/>  "id": "job_abc123",<br/>  "status": "pending",<br/>  "params": {...},<br/>  "createdAt": 1705900000,<br/>  "progress": 0<br/>}]

    Note over Vue,Rust: Get Single Job
    Vue->>Rust: invoke('get_queue_job', { jobId })
    Note right of Vue: Request:<br/>{ "jobId": "job_abc123" }

    alt Job Found
        Rust-->>Vue: Ok(Some(GenerationJob))
        Note left of Rust: Response:<br/>{<br/>  "id": "job_abc123",<br/>  "status": "processing",<br/>  "progress": 75<br/>}
    else Not Found
        Rust-->>Vue: Ok(None)
        Note left of Rust: Response: null
    end

    Note over Vue,Rust: Cancel Job
    Vue->>Rust: invoke('cancel_queue_job', { jobId })
    Note right of Vue: Request:<br/>{ "jobId": "job_abc123" }

    alt Cancelled
        Rust-->>Vue: Ok(true)
    else Cannot Cancel
        Rust-->>Vue: Ok(false)
    end

    Note over Vue,Rust: Clear Completed
    Vue->>Rust: invoke('clear_completed_jobs')
    Note right of Vue: No payload
    Rust-->>Vue: Ok(())
```

---

### 4. Gallery Management Flow

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend

    Note over Vue,Rust: Load Gallery (stores/gallery.ts)

    Vue->>Rust: invoke('get_gallery_images', { limit })
    Note right of Vue: Request:<br/>{ "limit": 100 }
    Rust-->>Vue: Ok(Vec<GalleryImage>)
    Note left of Rust: Response:<br/>[{<br/>  "id": "img_xyz789",<br/>  "filePath": "/gallery/image.png",<br/>  "thumbnailPath": "/thumbs/image.png",<br/>  "createdAt": 1705900000,<br/>  "width": 1024,<br/>  "height": 1024,<br/>  "fileSize": 2048000,<br/>  "isFavorite": false,<br/>  "prompt": "a cat astronaut",<br/>  "modelName": "flux-schnell",<br/>  "steps": 4,<br/>  "seed": 42,<br/>  "tags": ["ai", "cat"]<br/>}]

    Note over Vue,Rust: Search Images
    Vue->>Rust: invoke('search_gallery_images', { query })
    Note right of Vue: Request:<br/>{ "query": "astronaut" }
    Rust-->>Vue: Ok(Vec<ImageMetadata>)
    Note left of Rust: Response: filtered image list

    Note over Vue,Rust: Toggle Favorite
    Vue->>Rust: invoke('toggle_favorite', { imageId })
    Note right of Vue: Request:<br/>{ "imageId": "img_xyz789" }
    Rust-->>Vue: Ok(String)
    Note left of Rust: Response:<br/>"Favorite toggled"
```

---

### 5. Gallery Tags Flow

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend

    Note over Vue,Rust: Add Tag to Image
    Vue->>Rust: invoke('add_image_tag', { imageId, tag })
    Note right of Vue: Request:<br/>{<br/>  "imageId": "img_xyz789",<br/>  "tag": "favorite-style"<br/>}

    alt Success
        Rust-->>Vue: Ok(String)
        Note left of Rust: Response:<br/>"Tag added"
    else Error
        Rust-->>Vue: Err(String)
        Note left of Rust: Error:<br/>"Image not found"
    end

    Note over Vue,Rust: Remove Tag from Image
    Vue->>Rust: invoke('remove_image_tag', { imageId, tag })
    Note right of Vue: Request:<br/>{<br/>  "imageId": "img_xyz789",<br/>  "tag": "favorite-style"<br/>}
    Rust-->>Vue: Ok(String)
    Note left of Rust: Response:<br/>"Tag removed"

    Note over Vue,Rust: Delete Image
    Vue->>Rust: invoke('delete_gallery_image', { imageId })
    Note right of Vue: Request:<br/>{ "imageId": "img_xyz789" }
    Rust-->>Vue: Ok(String)
    Note left of Rust: Response:<br/>"Image deleted"
```

---

### 6. Model Management Flow

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend
    participant Events as Tauri Events

    Note over Vue,Rust: Check Model Status (ModelsView.vue)

    Vue->>Rust: invoke('get_model_status')
    Note right of Vue: No payload
    Rust-->>Vue: Ok(Vec<ModelFileStatus>)
    Note left of Rust: Response:<br/>[{<br/>  "name": "flux1-schnell-q8_0.gguf",<br/>  "exists": true,<br/>  "path": "~/.cache/rzem/models/..."<br/>}, {<br/>  "name": "t5xxl_fp16.safetensors",<br/>  "exists": false,<br/>  "path": "~/.cache/rzem/models/..."<br/>}]

    Vue->>Rust: invoke('check_models_downloaded')
    Note right of Vue: No payload
    Rust-->>Vue: Ok(bool)
    Note left of Rust: Response: true/false

    Note over Vue,Rust: Download Model
    Vue->>Rust: invoke('download_flux_schnell')
    Note right of Vue: No payload

    loop Download Progress Events
        Events-->>Vue: emit('download-progress', Progress)
        Note left of Events: Event:<br/>{<br/>  "file": "flux1-schnell.gguf",<br/>  "downloaded": 5000000000,<br/>  "total": 12000000000,<br/>  "percent": 41.6<br/>}
    end

    Rust-->>Vue: Ok(String)
    Note left of Rust: Response:<br/>"Model downloaded successfully"
```

---

### 7. Cache Management Flow

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend

    Note over Vue,Rust: Cache Operations (ManageView.vue)

    Vue->>Rust: invoke('get_cache_stats')
    Note right of Vue: No payload
    Rust-->>Vue: Ok(CacheStats)
    Note left of Rust: Response:<br/>{<br/>  "embedding_hits": 150,<br/>  "embedding_misses": 23,<br/>  "cached_embeddings": 45,<br/>  "pipeline_reuses": 89,<br/>  "pipeline_recreations": 3,<br/>  "current_model_type": "schnell",<br/>  "models_loaded": {<br/>    "t5": true,<br/>    "clip": true,<br/>    "vae": true,<br/>    "flux": true<br/>  }<br/>}

    Vue->>Rust: invoke('get_cache_config')
    Note right of Vue: No payload
    Rust-->>Vue: Ok(CacheConfigResponse)
    Note left of Rust: Response:<br/>{<br/>  "keep_vae_loaded": true,<br/>  "keep_flux_loaded": false,<br/>  "keep_t5_loaded": true,<br/>  "keep_clip_loaded": true,<br/>  "embedding_cache_size": 100,<br/>  "idle_timeout_secs": 300<br/>}

    Note over Vue,Rust: Apply Preset
    Vue->>Rust: invoke('set_cache_preset', { preset })
    Note right of Vue: Request:<br/>{ "preset": "keep_all" }
    Note right of Vue: Valid presets:<br/>"default" | "keep_all" | "memory_saver"
    Rust-->>Vue: Ok(String)
    Note left of Rust: Response:<br/>"Preset applied"
```

---

### 8. Cache Configuration Flow

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend

    Note over Vue,Rust: Update Cache Config
    Vue->>Rust: invoke('set_cache_config', config)
    Note right of Vue: Request:<br/>{<br/>  "keep_vae_loaded": true,<br/>  "keep_flux_loaded": true,<br/>  "keep_t5_loaded": true,<br/>  "keep_clip_loaded": true,<br/>  "embedding_cache_size": 200,<br/>  "idle_timeout_secs": 600<br/>}
    Rust-->>Vue: Ok(String)
    Note left of Rust: Response:<br/>"Config updated"

    Note over Vue,Rust: Clear All Cache
    Vue->>Rust: invoke('clear_model_cache')
    Note right of Vue: No payload
    Rust-->>Vue: Ok(String)
    Note left of Rust: Response:<br/>"Cache cleared, all models unloaded"
```

---

### 9. API Key Management Flow

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend

    Note over Vue,Rust: API Keys (ManageView.vue)

    rect rgb(240, 248, 255)
        Note over Vue,Rust: HuggingFace Token
        Vue->>Rust: invoke('get_hf_token')
        Note right of Vue: No payload
        Rust-->>Vue: Ok(Option<String>)
        Note left of Rust: Response:<br/>"hf_xxxx..." or null

        Vue->>Rust: invoke('set_hf_token', { token })
        Note right of Vue: Request:<br/>{ "token": "hf_xxxx..." }<br/>or { "token": null } to clear
        Rust-->>Vue: Ok(String)
        Note left of Rust: Response:<br/>"Token saved"
    end

    rect rgb(255, 248, 240)
        Note over Vue,Rust: Claude API Key
        Vue->>Rust: invoke('get_claude_api_key')
        Rust-->>Vue: Ok(Option<String>)

        Vue->>Rust: invoke('set_claude_api_key', { key })
        Note right of Vue: Request:<br/>{ "key": "sk-ant-..." }
        Rust-->>Vue: Ok(String)
    end

    rect rgb(240, 255, 240)
        Note over Vue,Rust: Fal.ai Key
        Vue->>Rust: invoke('get_fal_key')
        Rust-->>Vue: Ok(Option<String>)

        Vue->>Rust: invoke('set_fal_key', { key })
        Note right of Vue: Request:<br/>{ "key": "fal_..." }
        Rust-->>Vue: Ok(String)
    end
```

---

### 10. Image Analysis Flow

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend
    participant Claude as Claude API

    Note over Vue,Rust: Analyze Image for Prompt (imageAnalysis.ts)

    Vue->>Rust: invoke('analyze_image_for_prompt', payload)
    Note right of Vue: Request:<br/>{<br/>  "imageData": "data:image/png;base64,iVBOR...",<br/>  "mediaType": "image/png"<br/>}

    Rust->>Claude: POST /v1/messages
    Note right of Rust: Sends image to Claude<br/>for analysis
    Claude-->>Rust: Analysis response

    alt Success
        Rust-->>Vue: Ok(String)
        Note left of Rust: Response:<br/>"A detailed photograph of a sunset<br/>over mountains with vibrant orange<br/>and purple hues..."
    else API Error
        Rust-->>Vue: Err(String)
        Note left of Rust: Error:<br/>"Claude API error: Invalid API key"
    end

    Note over Vue,Rust: Read Image Metadata
    Vue->>Rust: invoke('read_image_metadata', { imagePath })
    Note right of Vue: Request:<br/>{ "imagePath": "/gallery/image.png" }

    alt Has Metadata
        Rust-->>Vue: Ok(Some(Value))
        Note left of Rust: Response:<br/>{<br/>  "prompt": "original prompt",<br/>  "seed": 42,<br/>  "steps": 4,<br/>  "model": "flux-schnell"<br/>}
    else No Metadata
        Rust-->>Vue: Ok(None)
        Note left of Rust: Response: null
    end
```

---

### 11. Direct Generation Flow (Legacy)

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend

    Note over Vue,Rust: Direct Image Generation<br/>(Not used by frontend - queue preferred)

    Vue->>Rust: invoke('generate_image', params)
    Note right of Vue: Request:<br/>{<br/>  "prompt": "a beautiful landscape",<br/>  "steps": 4,<br/>  "width": 1024,<br/>  "height": 1024,<br/>  "seed": -1<br/>}

    Note over Rust: Blocking operation<br/>~30-60 seconds

    alt Success
        Rust-->>Vue: Ok(String)
        Note left of Rust: Response:<br/>"/path/to/generated/image.png"
    else Generation Failed
        Rust-->>Vue: Err(String)
        Note left of Rust: Error:<br/>"Generation failed: Out of memory"
    end
```

---

## Error Response Patterns

All commands follow this error pattern:

```mermaid
sequenceDiagram
    participant Vue as Vue Frontend
    participant Rust as Rust Backend

    Vue->>Rust: invoke('any_command', payload)

    alt Success
        Rust-->>Vue: Ok(T)
        Note left of Rust: Type T varies by command
    else Error
        Rust-->>Vue: Err(String)
        Note left of Rust: Error message string<br/>describing what went wrong
    end
```

### Common Error Scenarios

| Error Type | Example Message |
|------------|-----------------|
| Database errors | `"Failed to query database: ..."` |
| File not found | `"Image not found: ..."` |
| Model errors | `"Model not downloaded"` |
| API errors | `"API key not configured"` |
| Generation errors | `"Generation failed: Out of memory"` |

---

## Data Types Reference

### GenerationParams
```typescript
interface GenerationParams {
  prompt: string;
  negative_prompt?: string;
  steps: number;
  cfg_scale: number;
  width: number;
  height: number;
  seed: number;
  model: string;
}
```

### GenerationJob
```typescript
interface GenerationJob {
  id: string;
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled';
  params: GenerationParams;
  createdAt: number;
  progress: number;
  step?: number;
  totalSteps?: number;
  outputPath?: string;
  error?: string;
}
```

### GalleryImage
```typescript
interface GalleryImage {
  id: string;
  filePath: string;
  thumbnailPath?: string;
  createdAt: number;
  width: number;
  height: number;
  fileSize: number;
  isFavorite: boolean;
  prompt: string;
  negativePrompt?: string;
  modelName: string;
  steps?: number;
  cfgScale?: number;
  seed?: number;
  sampler?: string;
  tags: string[];
}
```

### ModelAvailability
```typescript
interface ModelAvailability {
  id: string;
  name: string;
  is_downloaded: boolean;
  has_quantized: boolean;
}
```

### CacheStats
```typescript
interface CacheStats {
  embedding_hits: number;
  embedding_misses: number;
  cached_embeddings: number;
  pipeline_reuses: number;
  pipeline_recreations: number;
  current_model_type: string | null;
  models_loaded: {
    t5: boolean;
    clip: boolean;
    vae: boolean;
    flux: boolean;
  };
}
```

### CacheConfig
```typescript
interface CacheConfig {
  keep_vae_loaded: boolean;
  keep_flux_loaded: boolean;
  keep_t5_loaded: boolean;
  keep_clip_loaded: boolean;
  embedding_cache_size: number;
  idle_timeout_secs: number;
}
```

---

## Source File Locations

| Component | Path |
|-----------|------|
| All Tauri Commands | `src-tauri/src/lib.rs` |
| Queue Store | `src/stores/queue.ts` |
| Gallery Store | `src/stores/gallery.ts` |
| Models Store | `src/stores/models.ts` |
| App Initialization | `src/composables/useAppInit.ts` |
| Image Analysis Service | `src/services/imageAnalysis.ts` |
| Models View | `src/views/ModelsView.vue` |
| Manage View | `src/views/ManageView.vue` |
