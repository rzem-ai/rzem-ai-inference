# Moondream v3 Integration Analysis

## Overview

You have **Moondream v3 Preview** (`moondream/moondream3-preview`) and the int4-quantized MLX version (`moondream/md3p-int4`) downloaded. However, integrating Moondream v3 into the current Candle-based pipeline presents significant challenges.

## Current Implementation (Moondream v2)

**Repository**: `santiagomed/candle-moondream`
**Format**: GGUF quantized (q4_0)
**Size**: ~1.5GB
**Framework**: Candle (Rust)
**Status**: ✅ Working

### Architecture
- Simple vision encoder + text decoder
- Single GGUF file for easy loading
- Native Candle support via `candle-transformers`

---

## Moondream v3 Architecture

### Key Changes from v2

**1. Mixture of Experts (MoE)**
```json
"moe": {
  "num_experts": 64,
  "start_layer": 4,
  "experts_per_token": 8,
  "expert_inner_dim": 1024
}
```
- 64 expert networks
- 8 active experts per token
- Starts at layer 4
- Significantly more complex routing

**2. Enhanced Vision System**
```json
"vision": {
  "enc_dim": 1152,
  "enc_patch_size": 14,
  "enc_n_layers": 27,
  "crop_size": 378,
  "max_crops": 12,
  "overlap_margin": 4
}
```
- Multi-crop processing (up to 12 crops per image)
- Larger encoder (27 layers vs ~12 in v2)
- Overlap handling for better high-res understanding

**3. Region Understanding**
```json
"region": {
  "dim": 2048,
  "coord_feat_dim": 256,
  "size_feat_dim": 512
}
```
- Spatial coordinate encoding
- Object detection capabilities
- Size/position awareness

**4. Multiple Skills**
- `query`: Question answering
- `caption`: Image description
- `detect`: Object detection
- `point`: Spatial pointing

### Model Files

**moondream3-preview** (unquantized):
- `model-00001-of-00004.safetensors` (sharded)
- `model-00002-of-00004.safetensors`
- `model-00003-of-00004.safetensors`
- `model-00004-of-00004.safetensors`
- Total size: ~4.2GB
- Python implementation files (`.py`)

**md3p-int4** (MLX quantized):
- `model.safetensors` (int4 quantized)
- Size: ~2.3GB
- **Framework**: MLX (Apple's ML framework)
- **Not compatible with Candle**

---

## Integration Challenges

### 1. ❌ No Candle Support

**candle-transformers 0.8.4** does not include Moondream v3:
- Only has Moondream v2 (`moondream` module)
- No MoE implementations compatible with Moondream v3's routing
- Would require custom implementation

### 2. ❌ MLX Quantized Model Not Compatible

The `md3p-int4` model is quantized for MLX (Apple's framework):
- MLX is Python-only
- Not interoperable with Candle
- Would need conversion (non-trivial)

### 3. ❌ Complex Architecture

Implementing Moondream v3 in Candle would require:
- **MoE routing logic** (64 experts, top-8 selection)
- **Multi-crop vision processing** (12 crops with overlap)
- **Region encoding/decoding** (spatial understanding)
- **Sharded weight loading** (4 safetensors files)
- **BFloat16 precision** (Candle support exists but needs testing)

Estimated effort: **2-4 weeks** of full-time development.

---

## Recommended Options

### Option 1: Keep Moondream v2 ✅ (Recommended for Now)

**Pros:**
- Already working
- No changes needed
- Proven stable
- Fast inference

**Cons:**
- Lower quality than v3
- No spatial understanding
- No object detection

**When to choose:**
- You need tagging **now**
- Moondream v2 quality is acceptable
- Development time is limited

---

### Option 2: PyTorch Subprocess Integration 🔶

Run Moondream v3 via Python subprocess, similar to how we handle other Python-based tools.

#### Architecture

```
Rust (Tauri) → Python subprocess → Moondream v3 (PyTorch) → Results → Rust
```

#### Implementation Plan

**1. Create Python wrapper script** (`src-tauri/scripts/moondream3_tagger.py`):

```python
#!/usr/bin/env python3
import sys
import json
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer
from PIL import Image

def load_model():
    model = AutoModelForCausalLM.from_pretrained(
        "moondream/moondream3-preview",
        trust_remote_code=True,
        torch_dtype=torch.bfloat16,
        device_map="auto"
    )
    tokenizer = AutoTokenizer.from_pretrained("moondream/moondream3-preview")
    return model, tokenizer

def extract_tags(model, tokenizer, image_path):
    image = Image.open(image_path)

    # Use caption skill for tag extraction
    prompt = "List keywords describing this image, separated by commas."

    inputs = model.prepare_inputs(
        images=[image],
        text=[prompt],
        tokenizer=tokenizer
    )

    with torch.no_grad():
        outputs = model.generate(
            **inputs,
            max_new_tokens=200,
            do_sample=True,
            temperature=0.5
        )

    response = tokenizer.decode(outputs[0], skip_special_tokens=True)

    # Parse comma-separated tags
    tags = [tag.strip() for tag in response.split(',')]

    return {
        "tags": tags,
        "confidence": 0.85,  # Default confidence
        "backend": "moondream3"
    }

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(json.dumps({"error": "Usage: moondream3_tagger.py <image_path>"}))
        sys.exit(1)

    try:
        model, tokenizer = load_model()
        result = extract_tags(model, tokenizer, sys.argv[1])
        print(json.dumps(result))
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)
```

**2. Add Rust subprocess wrapper** (`src-tauri/src/vision/moondream3_subprocess.rs`):

```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct Moondream3Response {
    tags: Option<Vec<String>>,
    confidence: Option<f32>,
    backend: Option<String>,
    error: Option<String>,
}

pub fn extract_tags_subprocess(image_path: &Path) -> Result<Vec<super::TagWithConfidence>> {
    // Find Python script (bundled with app)
    let script_path = std::env::current_exe()?
        .parent()
        .context("No parent directory")?
        .join("scripts/moondream3_tagger.py");

    // Run Python subprocess
    let output = Command::new("python3")
        .arg(&script_path)
        .arg(image_path)
        .output()
        .context("Failed to spawn Python subprocess")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Python script failed: {}", stderr);
    }

    // Parse JSON response
    let response: Moondream3Response = serde_json::from_slice(&output.stdout)
        .context("Failed to parse Python script output")?;

    if let Some(error) = response.error {
        anyhow::bail!("Moondream3 error: {}", error);
    }

    let tags = response.tags.context("No tags in response")?;
    let confidence = response.confidence.unwrap_or(0.85);

    // Convert to TagWithConfidence
    Ok(tags
        .into_iter()
        .map(|tag| super::TagWithConfidence {
            tag,
            category: super::TagCategory::Subject,
            confidence,
        })
        .collect())
}
```

**3. Add enum variant for Moondream3** (`src-tauri/src/vision/models.rs`):

```rust
pub enum MoondreamVersion {
    V2Quantized, // Current implementation
    V3Subprocess, // New PyTorch-based
}

pub struct MoondreamTagger {
    version: MoondreamVersion,
    // ... existing fields
}

impl MoondreamTagger {
    pub fn extract_tags(&mut self, image_path: &Path) -> Result<Vec<TagWithConfidence>> {
        match self.version {
            MoondreamVersion::V2Quantized => {
                // Existing v2 implementation
            }
            MoondreamVersion::V3Subprocess => {
                moondream3_subprocess::extract_tags_subprocess(image_path)
            }
        }
    }
}
```

**Pros:**
- Can use official Moondream v3
- Highest quality results
- Access to all v3 features (detect, point, etc.)

**Cons:**
- Requires Python runtime + transformers library
- Slower (subprocess overhead + model loading)
- Higher memory usage (PyTorch + Rust)
- Need to bundle Python dependencies

**When to choose:**
- Quality is paramount
- Users have Python + PyTorch installed
- Willing to accept slower inference

---

### Option 3: Wait for Candle Support ⏳

Monitor candle-transformers for Moondream v3 support.

**Timeline:**
- Uncertain (could be weeks to months)
- Depends on Candle MoE maturity

**When to choose:**
- Not urgent
- Want native Rust performance
- Current v2 is acceptable

---

### Option 4: Contribute Candle Implementation 🚀

Implement Moondream v3 support in Candle yourself.

**Required Knowledge:**
- Deep understanding of transformers
- MoE architecture
- Candle framework internals
- PyTorch → Candle weight conversion

**Effort:** 2-4 weeks full-time

**Benefits:**
- Native Rust performance
- Contribute to open source
- Full control

---

## Recommendation

**Short term (now):**
Keep using Moondream v2. It works, it's stable, and it's fast.

**Medium term (1-2 months):**
If you need better quality:
1. Implement **Option 2** (PyTorch subprocess)
2. Add a settings toggle: "Moondream Version: v2 (fast) | v3 (quality)"
3. Let users choose based on their needs

**Long term (3-6 months):**
Monitor Candle for Moondream v3 support, then migrate to native implementation.

---

## Next Steps

If you want to proceed with **Option 2** (PyTorch subprocess):

1. **Install Python dependencies:**
   ```bash
   pip install transformers torch pillow
   ```

2. **Test Python script manually:**
   ```bash
   python3 scripts/moondream3_tagger.py test_image.png
   ```

3. **Implement Rust subprocess wrapper**
   - Add `moondream3_subprocess.rs`
   - Update `MoondreamTagger` enum
   - Add version selection to settings

4. **Bundle Python script with Tauri**
   - Update `tauri.conf.json` to include scripts/
   - Test in production build

Would you like me to implement Option 2 for you?

---

## Technical Comparison

| Feature | Moondream v2 (Current) | Moondream v3 (Native Candle) | Moondream v3 (Subprocess) |
|---------|------------------------|------------------------------|---------------------------|
| **Performance** | ⭐⭐⭐⭐⭐ Fast | ⭐⭐⭐⭐⭐ Fast | ⭐⭐ Slow |
| **Quality** | ⭐⭐⭐ Good | ⭐⭐⭐⭐⭐ Excellent | ⭐⭐⭐⭐⭐ Excellent |
| **Memory** | ⭐⭐⭐⭐ 1.5GB | ⭐⭐⭐ 4.2GB | ⭐⭐ 4.2GB + overhead |
| **Implementation** | ✅ Done | ❌ 2-4 weeks | 🔶 2-3 days |
| **Dependencies** | Candle only | Candle only | Python + PyTorch |
| **Startup Time** | ⭐⭐⭐⭐⭐ Instant | ⭐⭐⭐⭐⭐ Instant | ⭐⭐ 3-5 seconds |
| **Features** | Caption only | All (detect, point) | All (detect, point) |

---

## Files to Create (Option 2)

```
src-tauri/
├── scripts/
│   ├── moondream3_tagger.py          # Python wrapper
│   └── requirements.txt               # transformers, torch, Pillow
├── src/vision/
│   ├── moondream3_subprocess.rs      # Rust subprocess handler
│   └── models.rs                      # Update MoondreamTagger enum
└── tauri.conf.json                    # Bundle scripts/ directory
```

Let me know if you'd like me to implement this!
