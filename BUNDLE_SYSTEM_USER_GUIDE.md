# Model Bundle System - User Guide

## What Are Model Bundles?

Model bundles group the 4 required components for image generation:
1. **Transformer** - Main diffusion model (FLUX Schnell, Dev, or Z-Image)
2. **T5 Encoder** - Text understanding model
3. **CLIP Encoder** - Text-image alignment model
4. **VAE Decoder** - Converts latents to images

## Why Use Bundles?

### Before Bundles (Legacy Mode)
- App used hardcoded paths to specific models
- Could only use predetermined combinations
- Switching models was difficult
- No visibility into what was installed

### With Bundles
- ✅ **Auto-Discovery**: Automatically finds all installed models
- ✅ **Flexibility**: Mix and match components from different sources
- ✅ **Memory Optimization**: Choose quantized versions to save VRAM
- ✅ **Easy Switching**: Change bundles with one click
- ✅ **Transparency**: See exactly what components you have

## Getting Started

### Step 1: Scan for Models

1. Open the app
2. Go to **Models** view
3. Click the **Bundles** tab
4. Click **Scan Models** button

**What happens:**
- Scans your HuggingFace cache (`~/.cache/huggingface/hub`)
- Detects all model components (transformers, encoders, VAE, tokenizers)
- Auto-creates bundles from compatible components
- Shows results: "Found X components, created Y bundles"

**Expected bundles** (if you have FLUX installed):
- **FLUX.1 Schnell (Full Precision)** - Complete bundle, ~24GB VRAM
- **FLUX.1 Schnell (Quantized)** - Memory-efficient, ~12GB VRAM
- **FLUX.1 Dev (Full Precision)** - If Dev model installed
- **FLUX.1 Dev (Quantized)** - If quantized Dev installed

### Step 2: Activate a Bundle

1. In the Bundles tab, you'll see a list of detected bundles
2. Click on a bundle to view its details in the right panel
3. Review the components:
   - ✅ Green check = Component available
   - ❌ Red X = Component missing
4. Click **Activate** button

**What happens:**
- Bundle is marked as active (green "Active" tag appears)
- Next image generation will use components from this bundle
- Other bundles are automatically deactivated

**Active Bundle Status:**
- Green banner at top shows: "Active Bundle: [name]"
- Sidebar shows Active tag on the bundle
- Right panel shows Activate button grayed out (already active)

### Step 3: Generate an Image

1. Go to **Generate** view
2. Enter a prompt
3. Click Generate

**What happens:**
- Pipeline loads models from active bundle paths
- Console logs: "Loading models from active bundle"
- Image generated using bundle components

## Advanced Usage

### Creating Custom Bundles

Want to combine specific components? Create a custom bundle:

1. Click **Create Bundle** button
2. Enter bundle name (e.g., "My Optimized FLUX")
3. Add description (optional)
4. Select model family (FLUX or Z-Index)
5. Select components for each role:
   - **Transformer**: Choose full or quantized
   - **T5 Encoder**: Choose full or quantized
   - **CLIP Encoder**: Usually only one option
   - **VAE Decoder**: Usually only one option
6. Click **Create Bundle**

**Example Use Case:**
- Transformer: FLUX Schnell (Quantized) - Save VRAM
- T5: T5-XXL (Full Precision) - Better text understanding
- CLIP: CLIP-L (Standard)
- VAE: FLUX VAE (Standard)

**Result:** Balanced bundle with reduced VRAM but good quality

### Viewing Components

Click the **Components** tab to see all detected components:

**Filter by type:**
- All
- Transformers
- T5 Encoders
- CLIP Encoders
- VAE Decoders

**Component Details:**
- Name and quantization level
- Repository source
- File format (Safetensors, GGUF, JSON)
- File size and estimated VRAM
- Architecture (flux-schnell, t5-xxl, etc.)
- LoRA support (Transformers only)
- Sharding info (if model is split into multiple files)
- Availability status
- Full file path

### Managing Bundles

**Edit Bundle** (User-created only):
- Click pencil icon on bundle card
- Update name or description
- Click Update

**Delete Bundle** (User-created only):
- Click trash icon on bundle card
- Confirm deletion
- Bundle removed (components remain)

**Note:** Auto-discovered bundles cannot be edited or deleted (they regenerate on next scan)

## Understanding Bundle Types

### Auto-Discovered
- Created automatically by scanning
- Regenerated each scan
- Cannot be edited or deleted
- Use optimized component groupings

**Examples:**
- "FLUX.1 Schnell (Full Precision)"
- "FLUX.1 Dev (Q8_0)"

### User-Created
- Created manually by user
- Persist across scans
- Can be edited and deleted
- Allow custom component combinations

**Examples:**
- "My Custom FLUX"
- "Low VRAM Setup"

### System
- Reserved for future system-managed bundles
- Not currently used

## Bundle Status Indicators

### Tags
- **Active** (Green) - Currently in use
- **Incomplete** (Yellow) - Missing required components
- **Custom** (Blue) - User-created bundle

### Component Status
- ✅ **Green check** - Component file exists and is ready
- ❌ **Red X** - Component file missing or unavailable

### VRAM Indicators
- Shows total estimated VRAM usage
- Helps choose appropriate bundle for your GPU
- Example: "12.5 GB VRAM" vs "23.8 GB VRAM"

## Troubleshooting

### "No bundles found" after scan

**Possible causes:**
1. No models downloaded yet
2. HuggingFace cache in non-standard location
3. Models not in expected format

**Solutions:**
1. Download models from Downloads tab first
2. Set `HF_HOME` environment variable
3. Check console for scan errors

### "Bundle has missing components"

**Possible causes:**
1. Component file was deleted
2. HuggingFace cache was cleaned
3. File path changed

**Solutions:**
1. Rescan models to update availability
2. Redownload missing components
3. Choose a different bundle
4. Delete incomplete bundle

### "Failed to activate bundle"

**Possible causes:**
1. Database connection error
2. Bundle has missing components

**Solutions:**
1. Restart app
2. Check bundle completeness
3. Try different bundle
4. Check console for detailed error

### Generation fails after activating bundle

**Possible causes:**
1. Component paths incorrect
2. File permissions issue
3. Bundle validation error

**Solutions:**
1. Check console logs for specific error
2. Deactivate bundle (falls back to legacy mode)
3. Rescan models
4. Verify component files exist at listed paths

## Technical Details

### Bundle-Aware vs Legacy Mode

**Bundle Mode** (When bundle is active):
```
ModelPaths → Loads from database → Uses bundle component paths
```

**Legacy Mode** (No active bundle):
```
ModelPaths → Falls back to hardcoded → Uses traditional HF cache paths
```

**Switching:**
- Activate bundle → Bundle mode
- Deactivate all bundles → Legacy mode
- Automatic, transparent to user

### Where Are Bundles Stored?

**Database**: `~/.rzem-ai-inference/rzem.db`

**Tables:**
- `model_components` - Physical files
- `model_bundles` - Bundle metadata
- `bundle_components` - Relationships

**Note:** Bundles are just metadata. The actual model files remain in your HuggingFace cache.

### Component Detection Logic

**Transformers:**
- flux1-schnell.safetensors
- flux1-dev.safetensors
- *.gguf files
- transformer/ directory (sharded models)

**T5 Encoders:**
- text_encoder_2/ directory (split safetensors)
- t5-*-encoder-*.gguf files

**CLIP Encoders:**
- text_encoder/model.safetensors

**VAE Decoders:**
- ae.safetensors
- vae/ directory

**Tokenizers:**
- tokenizer/tokenizer.json (CLIP)
- tokenizer_2/tokenizer.json (T5)
- *.tokenizer.json files

## Best Practices

### For Most Users
1. **Use Auto-Discovered Bundles**
   - Scan after installing models
   - Activate the recommended bundle
   - Enjoy automatic component management

### For Advanced Users
2. **Create Custom Bundles**
   - Mix quantized and full-precision components
   - Optimize for your GPU's VRAM
   - Experiment with different combinations

### For Low VRAM
3. **Choose Quantized Bundles**
   - Look for bundles with "(Q8_0)" or "(Q5_K_M)"
   - Reduces VRAM from ~24GB to ~12GB
   - Minimal quality loss

### For Best Quality
4. **Use Full Precision Bundles**
   - Choose bundles without quantization tags
   - Requires more VRAM (~24GB)
   - Best quality output

## FAQs

### Q: Do I need to use bundles?
**A:** No. If no bundle is active, the app falls back to legacy mode and works exactly as before.

### Q: Can I have multiple bundles active?
**A:** No. Only one bundle can be active at a time to prevent component conflicts.

### Q: What happens to my downloads when I scan?
**A:** Nothing. Scanning only detects what's already there. It doesn't download or delete anything.

### Q: Can I delete auto-discovered bundles?
**A:** No. Auto-discovered bundles regenerate on next scan. Only user-created bundles can be deleted.

### Q: How do I go back to the old way?
**A:** Deactivate all bundles. The app will automatically use legacy hardcoded paths.

### Q: Can I share bundles with others?
**A:** Not yet, but bundle import/export is planned for a future release.

### Q: Does activating a bundle reload models immediately?
**A:** No. Models load on the next generation. Current generation continues with loaded models.

## Keyboard Shortcuts

- **Tab** - Navigate between form fields
- **Enter** - Submit forms
- **Escape** - Close dialogs
- **Arrow Keys** - Navigate dropdowns

## Summary

The model bundle system provides:
- 🔍 **Auto-Discovery** - Finds all your models
- 🎛️ **Flexibility** - Mix and match components
- 💾 **Memory Management** - Choose quantized for less VRAM
- 🚀 **Easy Switching** - One-click bundle changes
- 📊 **Visibility** - See what you have installed

Enjoy more control over your model setup! 🎉
