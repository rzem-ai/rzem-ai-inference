/**
 * Default model bundle definitions and bundle type guides.
 * Ported from backend/bundles.py.
 */

// ── Shared encoder repos ──

const CLIP = "openai/clip-vit-large-patch14";
const T5 = "google/t5-v1_1-xxl";
const T5_NF4 = JSON.stringify({ load_in_4bit: true, bnb_4bit_quant_type: "nf4" });
const QWEN3 = "Qwen/Qwen3-0.6B";

export interface BundleData {
	id: string;
	label: string;
	description: string;
	transformer_type: string;
	tier: string;
	transformer_model: string;
	vae_model: string;
	clip_tokenizer: string | null;
	clip_encoder: string | null;
	t5_tokenizer: string | null;
	t5_encoder: string | null;
	t5_encoder_config: string | null;
	qwen3_tokenizer: string | null;
	qwen3_encoder: string | null;
	steps: number;
	cfg_scale: number;
	sampler: string;
	scheduler: string;
	vram_estimate_gb: number;
	is_default: number;
	source: string;
	fal_endpoint: string | null;
	fal_aspectratio: string | null;
}

export interface BundleTypeData {
	id: string;
	label: string;
	icon: string;
	sort_order: number;
	guide: string;
}

export const DEFAULT_BUNDLES: BundleData[] = [
	// ── FLUX.1 Dev ──
	{
		id: "flux1_dev_performance",
		label: "FLUX.1 Dev - Fast",
		description: "FLUX.1-dev Q4 quantized - lower VRAM, faster inference",
		transformer_type: "flux1_dev",
		tier: "performance",
		transformer_model: "city96/FLUX.1-dev-gguf/flux1-dev-Q4_K_S.gguf",
		vae_model: "black-forest-labs/FLUX.1-dev",
		clip_tokenizer: CLIP, clip_encoder: CLIP,
		t5_tokenizer: T5, t5_encoder: T5, t5_encoder_config: T5_NF4,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 28, cfg_scale: 3.5, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 10.0, is_default: 1, source: "local",
		fal_endpoint: null, fal_aspectratio: null,
	},
	{
		id: "flux1_dev_balanced",
		label: "FLUX.1 Dev - Balanced",
		description: "FLUX.1-dev Q8 quantized - good quality/VRAM trade-off",
		transformer_type: "flux1_dev",
		tier: "balanced",
		transformer_model: "city96/FLUX.1-dev-gguf/flux1-dev-Q8_0.gguf",
		vae_model: "black-forest-labs/FLUX.1-dev",
		clip_tokenizer: CLIP, clip_encoder: CLIP,
		t5_tokenizer: T5, t5_encoder: T5, t5_encoder_config: T5_NF4,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 30, cfg_scale: 3.5, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 15.5, is_default: 1, source: "local",
		fal_endpoint: null, fal_aspectratio: null,
	},
	{
		id: "flux1_dev_quality",
		label: "FLUX.1 Dev - Quality",
		description: "FLUX.1-dev BF16 full precision - best quality",
		transformer_type: "flux1_dev",
		tier: "quality",
		transformer_model: "black-forest-labs/FLUX.1-dev",
		vae_model: "black-forest-labs/FLUX.1-dev",
		clip_tokenizer: CLIP, clip_encoder: CLIP,
		t5_tokenizer: T5, t5_encoder: T5, t5_encoder_config: null,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 40, cfg_scale: 4.0, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 33.7, is_default: 1, source: "local",
		fal_endpoint: null, fal_aspectratio: null,
	},
	// ── FLUX.1 Kontext ──
	{
		id: "flux1_kontext_performance",
		label: "FLUX.1 Kontext - Fast",
		description: "FLUX.1-Kontext Q4 quantized - lower VRAM, faster inference",
		transformer_type: "flux1_kontext",
		tier: "performance",
		transformer_model: "bullerwins/FLUX.1-Kontext-dev-GGUF/flux1-kontext-dev-Q4_K_S.gguf",
		vae_model: "black-forest-labs/FLUX.1-Kontext-dev",
		clip_tokenizer: CLIP, clip_encoder: CLIP,
		t5_tokenizer: T5, t5_encoder: T5, t5_encoder_config: T5_NF4,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 28, cfg_scale: 2.5, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 10.0, is_default: 1, source: "local",
		fal_endpoint: null, fal_aspectratio: null,
	},
	{
		id: "flux1_kontext_quality",
		label: "FLUX.1 Kontext - Quality",
		description: "FLUX.1-Kontext BF16 full precision - best quality",
		transformer_type: "flux1_kontext",
		tier: "quality",
		transformer_model: "black-forest-labs/FLUX.1-Kontext-dev",
		vae_model: "black-forest-labs/FLUX.1-Kontext-dev",
		clip_tokenizer: CLIP, clip_encoder: CLIP,
		t5_tokenizer: T5, t5_encoder: T5, t5_encoder_config: null,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 40, cfg_scale: 2.5, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 33.7, is_default: 1, source: "local",
		fal_endpoint: null, fal_aspectratio: null,
	},
	// ── FLUX.2 Dev ──
	{
		id: "flux2_dev_quality",
		label: "FLUX.2 Dev",
		description: "FLUX.2-dev BF16 with Qwen3 text encoder",
		transformer_type: "flux2_dev",
		tier: "quality",
		transformer_model: "black-forest-labs/FLUX.2-dev",
		vae_model: "black-forest-labs/FLUX.2-dev",
		clip_tokenizer: null, clip_encoder: null,
		t5_tokenizer: null, t5_encoder: null, t5_encoder_config: null,
		qwen3_tokenizer: QWEN3, qwen3_encoder: QWEN3,
		steps: 28, cfg_scale: 1.0, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 23.2, is_default: 1, source: "local",
		fal_endpoint: null, fal_aspectratio: null,
	},
	// ── Z-Image ──
	{
		id: "z_image_turbo",
		label: "Z-Image Turbo",
		description: "Z-Image Turbo 9-step with Qwen3-4B text encoder",
		transformer_type: "z_image",
		tier: "performance",
		transformer_model: "Tongyi-MAI/Z-Image-Turbo",
		vae_model: "Tongyi-MAI/Z-Image-Turbo",
		clip_tokenizer: null, clip_encoder: null,
		t5_tokenizer: null, t5_encoder: null, t5_encoder_config: null,
		qwen3_tokenizer: "Tongyi-MAI/Z-Image-Turbo", qwen3_encoder: "Tongyi-MAI/Z-Image-Turbo",
		steps: 9, cfg_scale: 1.0, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 20.2, is_default: 1, source: "local",
		fal_endpoint: null, fal_aspectratio: null,
	},
	{
		id: "z_image_quality",
		label: "Z-Image",
		description: "Z-Image BF16 with Qwen3-4B text encoder",
		transformer_type: "z_image",
		tier: "quality",
		transformer_model: "Tongyi-MAI/Z-Image",
		vae_model: "Tongyi-MAI/Z-Image",
		clip_tokenizer: null, clip_encoder: null,
		t5_tokenizer: null, t5_encoder: null, t5_encoder_config: null,
		qwen3_tokenizer: "Tongyi-MAI/Z-Image", qwen3_encoder: "Tongyi-MAI/Z-Image",
		steps: 28, cfg_scale: 1.0, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 20.2, is_default: 1, source: "local",
		fal_endpoint: null, fal_aspectratio: null,
	},
	// ── Qwen-Image ──
	{
		id: "qwen_image_quality",
		label: "Qwen-Image",
		description: "Qwen-Image BF16 with Qwen3 text encoder",
		transformer_type: "qwen_image",
		tier: "quality",
		transformer_model: "unsloth/Qwen-Image-2512-GGUF/qwen-image-2512-Q4_K_S.gguf",
		vae_model: "Qwen/Qwen-Image-2512",
		clip_tokenizer: null, clip_encoder: null,
		t5_tokenizer: null, t5_encoder: null, t5_encoder_config: null,
		qwen3_tokenizer: "Qwen/Qwen-Image-2512", qwen3_encoder: "Qwen/Qwen-Image-2512",
		steps: 28, cfg_scale: 1.0, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 20.0, is_default: 1, source: "local",
		fal_endpoint: null, fal_aspectratio: null,
	},
	// ── FAL.ai Cloud ──
	{
		id: "fal_flux_dev",
		label: "FLUX.1 Dev",
		description: "FAL.ai cloud - FLUX.1 Dev, balanced quality and speed",
		transformer_type: "fal_cloud", tier: "balanced",
		transformer_model: "fal-ai/flux/dev", vae_model: "cloud",
		clip_tokenizer: null, clip_encoder: null,
		t5_tokenizer: null, t5_encoder: null, t5_encoder_config: null,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 28, cfg_scale: 3.5, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 0.0, is_default: 1, source: "cloud",
		fal_endpoint: "fal-ai/flux/dev",
		fal_aspectratio: JSON.stringify(["square_hd", "square", "portrait_4_3", "portrait_16_9", "landscape_4_3", "landscape_16_9"]),
	},
	{
		id: "fal_flux_pro",
		label: "FLUX.1 Pro",
		description: "FAL.ai cloud - FLUX.1 Pro, fastest cloud generation",
		transformer_type: "fal_cloud", tier: "performance",
		transformer_model: "fal-ai/flux-pro/v1.1", vae_model: "cloud",
		clip_tokenizer: null, clip_encoder: null,
		t5_tokenizer: null, t5_encoder: null, t5_encoder_config: null,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 35, cfg_scale: 3.5, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 0.0, is_default: 1, source: "cloud",
		fal_endpoint: "fal-ai/flux-pro/v1.1",
		fal_aspectratio: JSON.stringify(["square_hd", "square", "portrait_4_3", "portrait_16_9", "landscape_4_3", "landscape_16_9"]),
	},
	{
		id: "fal_flux2_dev",
		label: "FLUX.2 Dev",
		description: "FAL.ai cloud - FLUX.2 Dev, highest quality cloud generation",
		transformer_type: "fal_cloud", tier: "quality",
		transformer_model: "fal-ai/flux-2", vae_model: "cloud",
		clip_tokenizer: null, clip_encoder: null,
		t5_tokenizer: null, t5_encoder: null, t5_encoder_config: null,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 35, cfg_scale: 3.5, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 0.0, is_default: 1, source: "cloud",
		fal_endpoint: "fal-ai/flux-2",
		fal_aspectratio: JSON.stringify(["square_hd", "square", "portrait_4_3", "portrait_16_9", "landscape_4_3", "landscape_16_9"]),
	},
	{
		id: "fal_flux2_flex",
		label: "FLUX.2 Flex",
		description: "FAL.ai cloud - FLUX.2 Flex, highest quality cloud generation",
		transformer_type: "fal_cloud", tier: "quality",
		transformer_model: "fal-ai/flux-2-flex", vae_model: "cloud",
		clip_tokenizer: null, clip_encoder: null,
		t5_tokenizer: null, t5_encoder: null, t5_encoder_config: null,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 35, cfg_scale: 3.5, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 0.0, is_default: 1, source: "cloud",
		fal_endpoint: "fal-ai/flux-2-flex",
		fal_aspectratio: JSON.stringify(["square_hd", "square", "portrait_4_3", "portrait_16_9", "landscape_4_3", "landscape_16_9"]),
	},
	{
		id: "fal_flux2_pro",
		label: "FLUX.2 Pro",
		description: "FAL.ai cloud - FLUX.2 Pro, highest quality cloud generation",
		transformer_type: "fal_cloud", tier: "quality",
		transformer_model: "fal-ai/flux-2-pro", vae_model: "cloud",
		clip_tokenizer: null, clip_encoder: null,
		t5_tokenizer: null, t5_encoder: null, t5_encoder_config: null,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 0, cfg_scale: 0, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 0.0, is_default: 1, source: "cloud",
		fal_endpoint: "fal-ai/flux-2-pro",
		fal_aspectratio: JSON.stringify(["square_hd", "square", "portrait_4_3", "portrait_16_9", "landscape_4_3", "landscape_16_9"]),
	},
	{
		id: "fal_nano_banana",
		label: "Nano Banana",
		description: "FAL.ai cloud - Nano Banana",
		transformer_type: "fal_cloud", tier: "quality",
		transformer_model: "fal-ai/flux-2-pro", vae_model: "cloud",
		clip_tokenizer: null, clip_encoder: null,
		t5_tokenizer: null, t5_encoder: null, t5_encoder_config: null,
		qwen3_tokenizer: null, qwen3_encoder: null,
		steps: 0, cfg_scale: 0, sampler: "euler", scheduler: "normal",
		vram_estimate_gb: 0.0, is_default: 1, source: "cloud",
		fal_endpoint: "fal-ai/nano-banana",
		fal_aspectratio: JSON.stringify(["21:9", "16:9", "3:2", "4:3", "5:4", "1:1", "4:5", "3:4", "2:3", "9:16"]),
	},
];

export const DEFAULT_BUNDLE_TYPES: BundleTypeData[] = [
	{
		id: "flux1_dev",
		label: "FLUX.1 Dev",
		icon: "gpu",
		sort_order: 0,
		guide: `## Prompting

FLUX.1 Dev responds best to **descriptive, natural language prompts**. Write as if you're describing a photograph or painting to someone who can't see it.

- **Be specific and descriptive** — "a woman with red hair standing in a sunlit wheat field at golden hour" works better than "woman in field"
- **Negative prompts have minimal effect** — focus on describing what you *want* rather than what to avoid
- **Quality modifiers help** — phrases like "highly detailed", "professional photograph", "8k" can nudge quality up

## Parameters

- **CFG Scale**: 3.0–4.5 gives the best coherence vs. creativity balance. Higher values increase prompt adherence but can reduce naturalness.
- **Steps**: 28–40 is the sweet spot. Below 20 you'll see artifacts; above 50 gives diminishing returns.
- **Sampler**: \`euler\` is the default and works well. \`dpmpp_2m\` with \`karras\` scheduler can produce slightly sharper results.

## Quantization

Quantized variants (GGUF) trade ~5–10% quality for 30–40% less VRAM. Q8 is nearly indistinguishable from BF16 in blind tests. Q4 shows mild softening on fine details like text and fingers.`,
	},
	{
		id: "flux1_kontext",
		label: "FLUX.1 Kontext",
		icon: "gpu",
		sort_order: 4,
		guide: "## FLUX.1 Kontext [dev]\n\nImage-to-image editing model from Black Forest Labs. Provide an input image and a text prompt describing the desired edit.\n\n**Requirements:** ~24GB VRAM (BF16)\n\n**Best for:** Editing existing images — changing styles, adding/removing elements, modifying attributes.",
	},
	{
		id: "flux2_dev",
		label: "FLUX.2 Dev",
		icon: "gpu",
		sort_order: 2,
		guide: `## Prompting

FLUX.2 Dev uses a **Qwen3 text encoder** instead of CLIP+T5, which means it understands more complex compositional prompts and supports **multilingual input**.

- **Complex scenes work well** — FLUX.2 handles multi-subject compositions and spatial relationships better than FLUX.1
- **Multilingual** — prompts in English, Chinese, Japanese, Korean, and other languages produce good results
- **Natural language over tags** — sentence-form prompts outperform comma-separated tag lists

## Parameters

- **CFG Scale**: 1.0 is recommended. Higher values tend to over-saturate colors and reduce naturalness.
- **Steps**: 28 is a good default. The model converges faster than FLUX.1.
- **Sampler**: \`euler\` works best. Avoid ancestral samplers (\`euler_a\`) as they add unnecessary noise.`,
	},
	{
		id: "z_image",
		label: "Z-Image",
		icon: "gpu",
		sort_order: 3,
		guide: `## Prompting

Z-Image uses a bundled **Qwen3-4B text encoder** — no separate encoder download is needed. It handles both English and Chinese prompts natively.

- **Descriptive prompts** work best, similar to FLUX models
- **Chinese and English** are both first-class — mix languages if useful
- **Style keywords** like "photorealistic", "anime", "oil painting" are effective

## Variants

- **Z-Image Turbo** (9 steps) is optimized for speed — great for rapid iteration and previews
- **Z-Image Standard** (28 steps) favors quality with more detailed outputs

## Parameters

- **CFG Scale**: 1.0 recommended for both variants
- **Sampler**: \`euler\` default works well across both variants`,
	},
	{
		id: "qwen_image",
		label: "Qwen-Image",
		icon: "gpu",
		sort_order: 5,
		guide: `## Prompting

Qwen-Image excels at **text rendering in images** and complex multi-subject compositions. Uses a Qwen3 encoder with multilingual support.

- **Text rendering** — this model can accurately render text within generated images, useful for signs, labels, and typographic art
- **Complex compositions** — handles detailed scenes with multiple subjects and spatial relationships
- **Multilingual** — strong support for Chinese, English, and other languages via the Qwen3 encoder

## Parameters

- **CFG Scale**: 1.0 recommended
- **Steps**: 28 is a good default
- **VRAM**: High requirements (~48 GB for BF16). Quantized variants significantly reduce this.`,
	},
	{
		id: "fal_cloud",
		label: "FAL.ai Cloud",
		icon: "cloud",
		sort_order: 10,
		guide: `## Overview

FAL.ai Cloud models run on remote servers — **no local GPU needed**. Generation quality matches local models since the same architectures are used server-side.

## Setup

- Requires a **FAL API key** — configure it in Settings → API Keys
- Aspect ratio is set via **preset strings** (e.g., "square_hd", "portrait_16_9") instead of pixel dimensions

## Usage Tips

- **Latency varies** — typical generation takes 3–15 seconds depending on queue depth and model
- **No VRAM constraints** — you can use full-precision models regardless of your local hardware
- **Steps/CFG may be ignored** — some cloud endpoints use fixed parameters; the values in your bundle are hints only`,
	},
];
