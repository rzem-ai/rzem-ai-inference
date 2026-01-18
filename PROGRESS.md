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

### Pending/Future Work
- Remove debug `console.log` statements from GenerationInput.vue
- Consider adding progress indicator for large image uploads
- Test with images dragged from web browsers
- Commit the current changes

### Git Status
Last commit: `14e1f64 fix: restore scroll behavior and enable DOM drag-drop events`

Uncommitted changes:
- `src-tauri/tauri.conf.json` - fs:scope expansion
- `src/components/generation/GenerationInput.vue` - Linux drag-drop support, HTML anchor parsing
