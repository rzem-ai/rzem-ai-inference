# Directory Model Scanner Design

## Overview

Add the ability to scan arbitrary directories for model files, identifying component types by inspecting file headers rather than relying on filename patterns or HuggingFace cache structure.

## User Flow

1. User clicks "Scan Models" button in ModelsSidebar
2. Directory picker dialog opens
3. User selects a folder (e.g., `/mnt/models` or `~/Downloads/models`)
4. Backend recursively scans for `.safetensors` and `.gguf` files
5. Each file's header is inspected to determine component type
6. Progress events update the UI during scan
7. Discovered components are added to the database

## Technical Approach

### File Type Detection

**Safetensors Format:**
- First 8 bytes: header length (u64 little-endian)
- Following bytes: JSON header with tensor names and shapes
- Tensor naming patterns reveal component type:
  - `double_blocks.*`, `single_blocks.*` → FLUX Transformer
  - `encoder.block.*`, `shared.weight` → T5 Encoder
  - `text_model.encoder.layers.*` → CLIP Encoder
  - `decoder.up_blocks.*`, `encoder.down_blocks.*` → VAE

**GGUF Format:**
- Magic bytes: `GGUF` at start
- Structured metadata including `general.architecture`
- Can identify model type directly from metadata fields

### New Backend Functions

1. `identify_safetensors_component(path) -> Option<ComponentType>`
   - Read header, parse tensor names, classify component

2. `identify_gguf_component(path) -> Option<ComponentType>`
   - Read GGUF metadata, extract architecture info

3. `scan_directory_for_models(path) -> Vec<DiscoveredComponent>`
   - Recursively find all model files
   - Identify each file's component type
   - Return structured results with file info

4. Tauri command: `scan_directory_for_models(path: String)`
   - Exposed to frontend
   - Emits progress events

### Frontend Changes

1. `handleScanModels()` in ModelsSidebar.vue:
   - Open directory picker dialog
   - Call backend scan command
   - Listen for progress events
   - Refresh model list on completion

## Files to Modify

- `src-tauri/src/models/scanner.rs` - Add file inspection functions
- `src-tauri/src/lib.rs` - Add new Tauri command
- `src/views/models/ModelsSidebar.vue` - Implement handleScanModels
- `src/stores/models.ts` - Add isScanning state if needed

## Progress Events

```typescript
interface ScanProgressEvent {
  stage: 'discovering' | 'identifying' | 'complete';
  message: string;
  progress: number; // 0-100
  filesFound?: number;
  filesProcessed?: number;
}
```
