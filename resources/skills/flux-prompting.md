---
name: flux-prompting
description: How to write prompts for FLUX.1 and FLUX.2 models
when_to_use: When the user is on a FLUX bundle and asks for help writing, improving, or understanding a prompt
model_families: [flux1_dev, flux1_kontext, flux2_dev]
tags: [prompting, flux]
---

# Prompting FLUX models

FLUX models respond best to **descriptive, natural-language prompts**. Write as if you're describing a photograph or painting to someone who can't see it — full sentences, scene-as-photograph framing, 50–200 words.

## Core principles

- **Be specific and descriptive.** "A woman with red hair standing in a sunlit wheat field at golden hour, shot on medium format film" works far better than "woman in field".
- **Negative prompts barely work.** FLUX has minimal response to "what to avoid" — describe what you *want* instead. Don't waste tokens on boilerplate negatives.
- **Quality modifiers help, sparingly.** Phrases like "highly detailed", "professional photograph", "cinematic" can nudge quality up. Don't stack ten of them.
- **Avoid SDXL-style tag prompts.** Comma-separated tags with weights like `(masterpiece:1.3), best quality, 8k, ...` actively hurt FLUX output. If a user pastes one, rewrite it as a sentence.
- **Composition vocabulary lands.** Camera shot type, lens, lighting direction, time of day, and mood all influence the output meaningfully.

## Parameters

- **CFG Scale**: 3.0–4.5 for FLUX.1 Dev. **1.0** for FLUX.2 Dev (higher values over-saturate). Higher CFG increases prompt adherence but reduces naturalness.
- **Steps**: 28–40 for FLUX.1 Dev. 28 is a good default for FLUX.2 (it converges faster).
- **Sampler**: `euler` is the safe default. `dpmpp_2m` with `karras` scheduler can produce slightly sharper results on FLUX.1. Avoid ancestral samplers (`euler_a`) on FLUX.2.

## FLUX.1 vs FLUX.2

FLUX.1 uses CLIP + T5 text encoders. FLUX.2 uses Qwen3, which means:

- **Better complex compositions.** FLUX.2 handles multi-subject scenes and spatial relationships ("a cat to the left of a dog, both looking at a window") much better.
- **Multilingual.** FLUX.2 understands prompts in English, Chinese, Japanese, Korean, and other languages natively. Mix freely if useful.

## Quantization

GGUF variants (Q4, Q8) trade ~5–10% quality for 30–40% less VRAM. Q8 is nearly indistinguishable from BF16 in blind tests. Q4 shows mild softening on fine details like text, fingers, and small textures.

## FLUX.1 Kontext

Kontext is an image-to-image edit model. Provide an input image plus a text prompt describing the desired edit (style change, element addition/removal, attribute modification). Same prompting style — descriptive sentences, not tags.
