# Rzem AI Inference - Progress Notes

## Session: 2026-01-19

### Features Implemented

#### 1. TipTap Rich Text Editor for Prompts
- Replaced PrimeVue Textarea with TipTap editor component
- **Files created:**
  - `src/components/generation/PromptEditor.vue` - Reusable TipTap editor component
  - `src/services/imageAnalysis.ts` - Image analysis service
- **Files modified:**
  - `src/components/generation/PromptInput.vue` - Updated to use PromptEditor
  - `package.json` - Added TipTap dependencies (@tiptap/core, @tiptap/vue-3, @tiptap/starter-kit, @tiptap/extension-placeholder, @tiptap/pm)

#### 2. Image Drag-and-Drop Analysis (Claude Vision)
- Drag an image onto the Generate card to analyze it and generate a prompt
- Uses Claude API (Sonnet 4) for vision-based image analysis
- **Backend (Rust):**
  - `src-tauri/src/claude/mod.rs` - Claude API client for image analysis
  - `src-tauri/src/lib.rs` - Added `analyze_image_for_prompt` Tauri command
- **Frontend (Vue):**
  - `src/components/generation/GenerationInput.vue` - Drag-and-drop handlers with visual overlay
  - `src/services/imageAnalysis.ts` - Frontend service to call backend

#### 3. Drag-and-Drop Platform Compatibility
- **Windows/macOS:** Uses standard `dataTransfer.files`
- **Linux (Nautilus/Files):** Handles `file://` URIs in `text/uri-list`
- **Browser images:** Fetches from URLs, extracts from HTML `<img>` and `<a>` tags

### Configuration Changes

#### Tauri Config (`src-tauri/tauri.conf.json`)
```json
{
  "windows": [{
    "dragDropEnabled": false  // Required for DOM drag events to work
  }],
  "security": {
    "capabilities": [{
      "permissions": [{
        "identifier": "fs:scope",
        "allow": [
          { "path": "$HOME/.rzem-ai-inference/**" },
          { "path": "$HOME/**" },
          { "path": "/tmp/**" }
        ]
      }]
    }]
  }
}
```

**Key settings:**
- `dragDropEnabled: false` - Prevents Tauri from intercepting drag events, allowing standard DOM events
- `fs:scope` expanded to `$HOME/**` - Required for reading images dropped from file managers

### Technical Notes

#### Linux File Manager Drag-and-Drop
Ubuntu's Files (Nautilus) sends dropped files as:
- `text/uri-list`: Contains `file:///path/to/file.png`
- `text/html`: Contains `<a>file:///path/to/file.png</a>`

The code handles both formats and uses Tauri's `readFile()` to read local files.

#### Claude API Integration
- Uses Claude Sonnet 4 (`claude-sonnet-4-20250514`)
- Sends base64-encoded images with a prompt to reverse-engineer the image
- Returns a detailed prompt suitable for FLUX image generation

#### 4. Seed Randomization Bug Fix

**Problem:** Generated images were identical regardless of seed setting. The seed parameter was being passed through the frontend but never actually used in the noise generation.

**Root Cause Analysis:**
- The seed was passed from Vue frontend → Rust backend via `GenerationParams.seed`
- In `processor.rs`, the seed was only used for the **output filename**: `flux_{}_{}.png`
- The actual noise generation in `flux::sampling::get_noise()` used `Tensor::randn()` without seeding
- Result: Every generation used a different random noise, making the seed parameter meaningless

**Solution:** Thread the seed through the entire generation pipeline:

1. **`src-tauri/src/models/flux.rs`**
   - `create_noise(height, width)` → `create_noise(height, width, seed: u64)`
   - Added `self.device.set_seed(seed)` call before `flux::sampling::get_noise()`
   - `denoise()` now accepts and passes seed to `create_noise()`

2. **`src-tauri/src/inference/pipeline.rs`**
   - `generate()` and `generate_with_progress()` now accept `seed: u64` parameter
   - Pass seed to `flux.denoise()`

3. **`src-tauri/src/queue/processor.rs`**
   - Convert `params.seed` (i64) to u64
   - Pass to `pipeline.generate_with_progress()`

4. **`src-tauri/src/lib.rs`**
   - `generate_image` command now passes seed to pipeline

5. **`src-tauri/Cargo.toml`**
   - Added `rand = "0.8"` dependency for fallback random seeds

**Technical Detail:**
- Candle's `Device::set_seed()` works on CUDA and Metal backends
- CPU backend doesn't support seeding (logs a note, but GPU is typical use case)
- Setting seed before `Tensor::randn()` makes noise generation deterministic

**Frontend Changes (from previous session):**
- `src/stores/generation.ts` - Added `randomizeSeedOnGenerate` state
- `src/components/generation/LeftSidebar.vue` - Seed lock/unlock toggle UI
- `src/views/GenerateView.vue` - Randomize seed on generate when unlocked, display actual seed used

#### 5. Dual Progress Bars in Queue UI

**Feature:** Split the generation progress display into two progress bars:
1. **Pipeline bar (teal):** Overall progress 0-100% showing current stage
2. **Steps bar (blue):** Denoising step count (0/N → N/N)

**Backend Changes:**
- `src-tauri/src/inference/progress.rs` - Added `current_step` and `total_steps` fields to `GenerationProgress`
- `src-tauri/src/queue/processor.rs` - Emit step info in `job-progress` event

**Frontend Changes:**
- `src/stores/queue.ts` - Added `currentStep` and `totalSteps` to `GenerationJob` interface
- `src/components/generation/BottomPanel.vue` - Dual progress bars with stage/step display
- `src/main.ts` - Registered PrimeVue Tooltip directive globally

**Cleanup - Removed Redundant Components:**
- Deleted `src/components/generation/QueueList.vue` (unused)
- Deleted `src/components/queue/QueuePanel.vue` (unused)
- Deleted `src/components/queue/` directory
- Updated docs with superseded notes

**Progress Display Logic:**
- Before denoising: Steps bar shows 0/N
- During denoising: Steps bar shows current/N (from `currentStep`)
- After denoising: Steps bar shows N/N
- Stage names mapped to human-readable text (e.g., "Loading models...", "Denoising...")

### Pending/Future Work
- Remove debug `console.log` statements from GenerationInput.vue
- Consider adding progress indicator for large image uploads
- Test with images dragged from web browsers
- Test dual progress bars with actual generation
- Commit the current changes

### Git Status
Last commit: `14e1f64 fix: restore scroll behavior and enable DOM drag-drop events`

Uncommitted changes (this session):
- Seed fix: `src-tauri/src/models/flux.rs`, `src-tauri/src/inference/pipeline.rs`, `src-tauri/src/queue/processor.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`
- Progress bars: `src-tauri/src/inference/progress.rs`, `src/stores/queue.ts`, `src/components/generation/BottomPanel.vue`
- Cleanup: Removed `QueueList.vue`, `QueuePanel.vue`
- Config: `src/main.ts` (Tooltip directive)
