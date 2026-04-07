---
name: sdxl-prompting
description: Tag-style prompting for SDXL and Stable Diffusion 1.5 family models
when_to_use: When the user is working with an SDXL or SD 1.5 model and needs help with tag-style prompts, weights, or negative prompts
tags: [prompting, sdxl, sd15]
---

# Prompting SDXL / SD 1.5 family

SDXL and SD 1.5 use a fundamentally different prompt style than FLUX or Qwen — **comma-separated tags with optional weights**, not descriptive sentences. If a user pastes an SDXL prompt while on a FLUX bundle, rewrite it as a sentence. If a user pastes a FLUX prompt while on SDXL, rewrite it as tags.

## Tag style

Prompts are comma-separated phrases ordered roughly by importance. Earlier tags carry more weight. Example:

```
masterpiece, best quality, highres, 1girl, red hair, freckles, green eyes,
white blouse, sitting in a meadow, golden hour lighting, depth of field,
sharp focus, photorealistic, 35mm film
```

## Weight syntax

Wrap a token in parentheses with `:weight` to amplify or attenuate it:

- `(red hair:1.3)` — 30% stronger
- `(blurry:0.5)` — 50% weaker
- `((masterpiece))` — shorthand for ~1.21x (each layer of parens multiplies by 1.1)
- `[blurry]` — shorthand for ~0.91x (square brackets divide by 1.1)

Stay between 0.5 and 1.5 — values outside this range usually break the image.

## Quality tags

SDXL and SD 1.5 were trained on tagged datasets where "masterpiece, best quality, highres" cues correlate with higher-fidelity outputs. Adding them to the front of the prompt reliably nudges quality up. Booru-tag models also respond to tags like `8k, ultra detailed, sharp focus, cinematic lighting`.

## Negative prompts (real and important here)

Unlike FLUX, **negative prompts work strongly on SDXL/SD 1.5** and are essential for quality output. Standard negative starter:

```
worst quality, low quality, lowres, blurry, jpeg artifacts, watermark,
signature, text, ugly, bad anatomy, bad hands, extra fingers, deformed
```

Add scene-specific negatives as needed (e.g. `multiple people` if you only want one).

## Parameters

- **CFG Scale**: 6.0–9.0 sweet spot. Below 5 the image ignores the prompt; above 11 it over-bakes.
- **Steps**: 25–40 for SDXL, 20–30 for SD 1.5. DPM++ samplers with Karras scheduler are popular.

## Contrast with FLUX

| | FLUX | SDXL |
|---|---|---|
| Style | Natural-language sentences | Comma-separated tags |
| Negative prompt | Minimal effect | Essential |
| CFG range | 1.0–4.5 | 6.0–9.0 |
| Quality tags | Sparingly | "masterpiece, best quality" reliably helps |
| Weight syntax | Not parsed | `(token:1.3)` core feature |

If a user is on a Flux bundle and asks for "(masterpiece:1.3), best quality" tag prompting, gently redirect — that style hurts Flux output.
