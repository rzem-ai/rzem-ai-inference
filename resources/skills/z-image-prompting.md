---
name: z-image-prompting
description: How to write prompts for Z-Image (Standard and Turbo)
when_to_use: When the user is on a Z-Image bundle and asks for help writing or improving a prompt
model_families: [z_image]
tags: [prompting, z-image]
---

# Prompting Z-Image

Z-Image bundles a **Qwen3-4B text encoder** internally — no separate encoder download needed. It's bilingual (English and Chinese first-class) and handles descriptive prompts very similarly to FLUX.

## Prompt style

- **Descriptive sentences** work best, not comma-separated tags.
- **English and Chinese** are both first-class. Mix languages freely if useful.
- **Style keywords land well.** "photorealistic", "anime", "oil painting", "watercolor", "studio ghibli style" all produce clear stylistic shifts.
- Avoid SDXL-style weight syntax like `(token:1.3)` — Z-Image doesn't parse it.

## Variants

- **Z-Image Turbo (9 steps)** — optimized for speed. Use for rapid iteration, previews, batch exploration, or anything where you'd run many generations.
- **Z-Image Standard (28 steps)** — favors quality with more detailed outputs. Use for final renders where the extra time is worth it.

If the user is iterating on a prompt and getting frustrated by wait times, suggest switching to Turbo until the prompt feels right, then back to Standard for the final.

## Parameters

- **CFG Scale**: 1.0 recommended for both variants.
- **Sampler**: `euler` default works well across both.
- **Steps**: 9 (Turbo) or 28 (Standard) — these are tuned for the variant, don't override unless the user has a specific reason.
