"""Model bundle data model and default definitions."""

from __future__ import annotations

import logging
from dataclasses import asdict, dataclass

from backend.tracing import trace_method

logger = logging.getLogger(__name__)


@dataclass
class ModelBundle:
    id: str
    label: str
    description: str
    transformer_type: str  # "flux1_dev" | "flux2_dev" | "z_image" | "qwen_image" | "fal_cloud"
    tier: str  # "performance" | "balanced" | "quality"
    transformer_model: str
    vae_model: str
    clip_tokenizer: str | None = None
    clip_encoder: str | None = None
    t5_tokenizer: str | None = None
    t5_encoder: str | None = None
    t5_encoder_config: dict | str | None = None
    qwen3_tokenizer: str | None = None
    qwen3_encoder: str | None = None
    steps: int = 20
    cfg_scale: float = 1.0
    sampler: str = "euler"
    scheduler: str = "normal"
    vram_estimate_gb: float = 0.0
    is_default: bool = True
    source: str = "local"  # "local" | "cloud"
    fal_endpoint: str | None = None
    fal_aspectratio: list[str] | None = None


# ── Shared encoder repos ──

_CLIP = "openai/clip-vit-large-patch14"
_T5 = "google/t5-v1_1-xxl"
_T5_NF4 = {"load_in_4bit": True, "bnb_4bit_quant_type": "nf4"}
_QWEN3 = "Qwen/Qwen3-0.6B"

DEFAULT_BUNDLES: list[ModelBundle] = [
    # ── FLUX.1 Dev ──
    ModelBundle(
        id="flux1_dev_performance",
        label="FLUX.1 Dev - Fast",
        description="FLUX.1-dev Q4 quantized - lower VRAM, faster inference",
        transformer_type="flux1_dev",
        tier="performance",
        transformer_model="city96/FLUX.1-dev-gguf/flux1-dev-Q4_K_S.gguf",
        vae_model="black-forest-labs/FLUX.1-dev",
        clip_tokenizer=_CLIP,
        clip_encoder=_CLIP,
        t5_tokenizer=_T5,
        t5_encoder=_T5,
        t5_encoder_config=_T5_NF4,
        steps=28,
        cfg_scale=3.5,
        vram_estimate_gb=11.0,
    ),
    ModelBundle(
        id="flux1_dev_balanced",
        label="FLUX.1 Dev - Balanced",
        description="FLUX.1-dev Q8 quantized - good quality/VRAM trade-off",
        transformer_type="flux1_dev",
        tier="balanced",
        transformer_model="city96/FLUX.1-dev-gguf/flux1-dev-Q8_0.gguf",
        vae_model="black-forest-labs/FLUX.1-dev",
        clip_tokenizer=_CLIP,
        clip_encoder=_CLIP,
        t5_tokenizer=_T5,
        t5_encoder=_T5,
        t5_encoder_config=_T5_NF4,
        steps=30,
        cfg_scale=3.5,
        vram_estimate_gb=16.0,
    ),
    # Quality last - safest default (no GGUF dependency)
    ModelBundle(
        id="flux1_dev_quality",
        label="FLUX.1 Dev - Quality",
        description="FLUX.1-dev BF16 full precision - best quality",
        transformer_type="flux1_dev",
        tier="quality",
        transformer_model="black-forest-labs/FLUX.1-dev",
        vae_model="black-forest-labs/FLUX.1-dev",
        clip_tokenizer=_CLIP,
        clip_encoder=_CLIP,
        t5_tokenizer=_T5,
        t5_encoder=_T5,
        steps=40,
        cfg_scale=4.0,
        vram_estimate_gb=33.7,
    ),
    # ── FLUX.1 Kontext ──
    ModelBundle(
        id="flux1_kontext_performance",
        label="FLUX.1 Kontext - Fast",
        description="FLUX.1-Kontext Q4 quantized - lower VRAM, faster inference",
        transformer_type="flux1_kontext",
        tier="performance",
        transformer_model="bullerwins/FLUX.1-Kontext-dev-GGUF/flux1-kontext-dev-Q4_K_S.gguf",
        vae_model="black-forest-labs/FLUX.1-Kontext-dev",
        clip_tokenizer=_CLIP,
        clip_encoder=_CLIP,
        t5_tokenizer=_T5,
        t5_encoder=_T5,
        t5_encoder_config=_T5_NF4,
        steps=28,
        cfg_scale=2.5,
        vram_estimate_gb=11.0,
    ),
    ModelBundle(
        id="flux1_kontext_quality",
        label="FLUX.1 Kontext - Quality",
        description="FLUX.1-Kontext BF16 full precision - best quality",
        transformer_type="flux1_kontext",
        tier="quality",
        transformer_model="black-forest-labs/FLUX.1-Kontext-dev",
        vae_model="black-forest-labs/FLUX.1-Kontext-dev",
        clip_tokenizer=_CLIP,
        clip_encoder=_CLIP,
        t5_tokenizer=_T5,
        t5_encoder=_T5,
        steps=40,
        cfg_scale=2.5,
        vram_estimate_gb=33.7,
    ),
    # ── FLUX.2 Dev ──
    ModelBundle(
        id="flux2_dev_quality",
        label="FLUX.2 Dev",
        description="FLUX.2-dev BF16 with Qwen3 text encoder",
        transformer_type="flux2_dev",
        tier="quality",
        transformer_model="black-forest-labs/FLUX.2-dev",
        vae_model="black-forest-labs/FLUX.2-dev",
        qwen3_tokenizer=_QWEN3,
        qwen3_encoder=_QWEN3,
        steps=28,
        cfg_scale=1.0,
        vram_estimate_gb=23.2,
    ),
    # ── Z-Image ──
    # Z-Image uses Qwen3-4B (hidden_size=2560), bundled inside each repo
    ModelBundle(
        id="z_image_turbo",
        label="Z-Image Turbo",
        description="Z-Image Turbo 9-step with Qwen3-4B text encoder",
        transformer_type="z_image",
        tier="performance",
        transformer_model="Tongyi-MAI/Z-Image-Turbo",
        vae_model="Tongyi-MAI/Z-Image-Turbo",
        qwen3_tokenizer="Tongyi-MAI/Z-Image-Turbo",
        qwen3_encoder="Tongyi-MAI/Z-Image-Turbo",
        steps=9,
        cfg_scale=1.0,
        vram_estimate_gb=20.2,
    ),
    ModelBundle(
        id="z_image_quality",
        label="Z-Image",
        description="Z-Image BF16 with Qwen3-4B text encoder",
        transformer_type="z_image",
        tier="quality",
        transformer_model="Tongyi-MAI/Z-Image",
        vae_model="Tongyi-MAI/Z-Image",
        qwen3_tokenizer="Tongyi-MAI/Z-Image",
        qwen3_encoder="Tongyi-MAI/Z-Image",
        steps=28,
        cfg_scale=1.0,
        vram_estimate_gb=20.2,
    ),
    # ── Qwen-Image ──
    ModelBundle(
        id="qwen_image_quality",
        label="Qwen-Image",
        description="Qwen-Image BF16 with Qwen3 text encoder",
        transformer_type="qwen_image",
        tier="quality",
        transformer_model="unsloth/Qwen-Image-2512-GGUF/qwen-image-2512-Q4_K_S.gguf",
        vae_model="Qwen/Qwen-Image-2512",
        qwen3_tokenizer="Qwen/Qwen-Image-2512",
        qwen3_encoder="Qwen/Qwen-Image-2512",
        steps=28,
        cfg_scale=1.0,
        vram_estimate_gb=48.2,
    ),
    # ── FAL.ai Cloud ──
    ModelBundle(
        id="fal_flux_dev",
        label="FLUX.1 Dev",
        description="FAL.ai cloud - FLUX.1 Dev, balanced quality and speed",
        transformer_type="fal_cloud",
        tier="balanced",
        transformer_model="fal-ai/flux/dev",
        vae_model="cloud",
        steps=28,
        cfg_scale=3.5,
        source="cloud",
        fal_endpoint="fal-ai/flux/dev",
        fal_aspectratio=["square_hd", "square", "portrait_4_3", "portrait_16_9", "landscape_4_3", "landscape_16_9"]
    ),
    ModelBundle(
        id="fal_flux_pro",
        label="FLUX.1 Pro",
        description="FAL.ai cloud - FLUX.1 Pro, fastest cloud generation",
        transformer_type="fal_cloud",
        tier="performance",
        transformer_model="fal-ai/flux-pro/v1.1",
        vae_model="cloud",
        steps=35,
        cfg_scale=3.5,
        source="cloud",
        fal_endpoint="fal-ai/flux-pro/v1.1",
        fal_aspectratio=["square_hd", "square", "portrait_4_3", "portrait_16_9", "landscape_4_3", "landscape_16_9"]
    ),
    ModelBundle(
        id="fal_flux2_dev",
        label="FLUX.2 Dev",
        description="FAL.ai cloud - FLUX.2 Dev, highest quality cloud generation",
        transformer_type="fal_cloud",
        tier="quality",
        transformer_model="fal-ai/flux-2",
        vae_model="cloud",
        steps=35,
        cfg_scale=3.5,
        source="cloud",
        fal_endpoint="fal-ai/flux-2",
        fal_aspectratio=["square_hd", "square", "portrait_4_3", "portrait_16_9", "landscape_4_3", "landscape_16_9"]
    ),
    ModelBundle(
        id="fal_flux2_flex",
        label="FLUX.2 Flex",
        description="FAL.ai cloud - FLUX.2 Flex, highest quality cloud generation",
        transformer_type="fal_cloud",
        tier="quality",
        transformer_model="fal-ai/flux-2-flex",
        vae_model="cloud",
        steps=35,
        cfg_scale=3.5,
        source="cloud",
        fal_endpoint="fal-ai/flux-2-flex",
        fal_aspectratio=["square_hd", "square", "portrait_4_3", "portrait_16_9", "landscape_4_3", "landscape_16_9"]
    ),
    ModelBundle(
        id="fal_flux2_pro",
        label="FLUX.2 Pro",
        description="FAL.ai cloud - FLUX.2 Pro, highest quality cloud generation",
        transformer_type="fal_cloud",
        tier="quality",
        transformer_model="fal-ai/flux-2-pro",
        vae_model="cloud",
        steps=0,
        cfg_scale=0,
        source="cloud",
        fal_endpoint="fal-ai/flux-2-pro",
        fal_aspectratio=["square_hd", "square", "portrait_4_3", "portrait_16_9", "landscape_4_3", "landscape_16_9"]
    ),
    ModelBundle(
        id="fal_nano_banana",
        label="Nano Banana",
        description="FAL.ai cloud - Nano Banana",
        transformer_type="fal_cloud",
        tier="quality",
        transformer_model="fal-ai/flux-2-pro",
        vae_model="cloud",
        steps=0,
        cfg_scale=0,
        source="cloud",
        fal_endpoint="fal-ai/nano-banana",
        fal_aspectratio=["21:9", "16:9", "3:2", "4:3", "5:4", "1:1", "4:5", "3:4", "2:3", "9:16"]
    ),
]


DEFAULT_BUNDLE_TYPES: list[dict] = [
    {
        "id": "flux1_dev",
        "label": "FLUX.1 Dev",
        "icon": "gpu",
        "sort_order": 0,
        "guide": """\
## Prompting

FLUX.1 Dev responds best to **descriptive, natural language prompts**. Write as if you're describing a photograph or painting to someone who can't see it.

- **Be specific and descriptive** — "a woman with red hair standing in a sunlit wheat field at golden hour" works better than "woman in field"
- **Negative prompts have minimal effect** — focus on describing what you *want* rather than what to avoid
- **Quality modifiers help** — phrases like "highly detailed", "professional photograph", "8k" can nudge quality up

## Parameters

- **CFG Scale**: 3.0–4.5 gives the best coherence vs. creativity balance. Higher values increase prompt adherence but can reduce naturalness.
- **Steps**: 28–40 is the sweet spot. Below 20 you'll see artifacts; above 50 gives diminishing returns.
- **Sampler**: `euler` is the default and works well. `dpmpp_2m` with `karras` scheduler can produce slightly sharper results.

## Quantization

Quantized variants (GGUF) trade ~5–10% quality for 30–40% less VRAM. Q8 is nearly indistinguishable from BF16 in blind tests. Q4 shows mild softening on fine details like text and fingers.\
""",
    },
    {
        "id": "flux1_kontext",
        "label": "FLUX.1 Kontext",
        "icon": "gpu",
        "sort_order": 4,
        "guide": "## FLUX.1 Kontext [dev]\n\nImage-to-image editing model from Black Forest Labs. Provide an input image and a text prompt describing the desired edit.\n\n**Requirements:** ~24GB VRAM (BF16)\n\n**Best for:** Editing existing images — changing styles, adding/removing elements, modifying attributes.",
    },
    {
        "id": "flux2_dev",
        "label": "FLUX.2 Dev",
        "icon": "gpu",
        "sort_order": 2,
        "guide": """\
## Prompting

FLUX.2 Dev uses a **Qwen3 text encoder** instead of CLIP+T5, which means it understands more complex compositional prompts and supports **multilingual input**.

- **Complex scenes work well** — FLUX.2 handles multi-subject compositions and spatial relationships better than FLUX.1
- **Multilingual** — prompts in English, Chinese, Japanese, Korean, and other languages produce good results
- **Natural language over tags** — sentence-form prompts outperform comma-separated tag lists

## Parameters

- **CFG Scale**: 1.0 is recommended. Higher values tend to over-saturate colors and reduce naturalness.
- **Steps**: 28 is a good default. The model converges faster than FLUX.1.
- **Sampler**: `euler` works best. Avoid ancestral samplers (`euler_a`) as they add unnecessary noise.\
""",
    },
    {
        "id": "z_image",
        "label": "Z-Image",
        "icon": "gpu",
        "sort_order": 3,
        "guide": """\
## Prompting

Z-Image uses a bundled **Qwen3-4B text encoder** — no separate encoder download is needed. It handles both English and Chinese prompts natively.

- **Descriptive prompts** work best, similar to FLUX models
- **Chinese and English** are both first-class — mix languages if useful
- **Style keywords** like "photorealistic", "anime", "oil painting" are effective

## Variants

- **Z-Image Turbo** (9 steps) is optimized for speed — great for rapid iteration and previews
- **Z-Image Standard** (28 steps) favors quality with more detailed outputs

## Parameters

- **CFG Scale**: 1.0 recommended for both variants
- **Sampler**: `euler` default works well across both variants\
""",
    },
    {
        "id": "qwen_image",
        "label": "Qwen-Image",
        "icon": "gpu",
        "sort_order": 5,
        "guide": """\
## Prompting

Qwen-Image excels at **text rendering in images** and complex multi-subject compositions. Uses a Qwen3 encoder with multilingual support.

- **Text rendering** — this model can accurately render text within generated images, useful for signs, labels, and typographic art
- **Complex compositions** — handles detailed scenes with multiple subjects and spatial relationships
- **Multilingual** — strong support for Chinese, English, and other languages via the Qwen3 encoder

## Parameters

- **CFG Scale**: 1.0 recommended
- **Steps**: 28 is a good default
- **VRAM**: High requirements (~48 GB for BF16). Quantized variants significantly reduce this.\
""",
    },
    {
        "id": "fal_cloud",
        "label": "FAL.ai Cloud",
        "icon": "cloud",
        "sort_order": 10,
        "guide": """\
## Overview

FAL.ai Cloud models run on remote servers — **no local GPU needed**. Generation quality matches local models since the same architectures are used server-side.

## Setup

- Requires a **FAL API key** — configure it in Settings → API Keys
- Aspect ratio is set via **preset strings** (e.g., "square_hd", "portrait_16_9") instead of pixel dimensions

## Usage Tips

- **Latency varies** — typical generation takes 3–15 seconds depending on queue depth and model
- **No VRAM constraints** — you can use full-precision models regardless of your local hardware
- **Steps/CFG may be ignored** — some cloud endpoints use fixed parameters; the values in your bundle are hints only\
""",
    },
]


@trace_method
def get_default_bundle_types_as_dicts() -> list[dict]:
    """Return DEFAULT_BUNDLE_TYPES for database seeding."""
    return DEFAULT_BUNDLE_TYPES


@trace_method
def get_default_bundles_as_dicts() -> list[dict]:
    """Return DEFAULT_BUNDLES as plain dicts (for database seeding)."""
    return [asdict(b) for b in DEFAULT_BUNDLES]
