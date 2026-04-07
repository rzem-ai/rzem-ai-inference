---
name: qwen-image-prompting
description: How to write prompts for Qwen-Image
when_to_use: When the user is on a Qwen-Image bundle and asks for help writing or improving a prompt
model_families: [qwen_image]
tags: [prompting, qwen]
---

# Prompting Qwen-Image

Qwen-Image's two standout strengths are **text rendering inside images** and **complex multi-subject compositions**. It uses a Qwen3 text encoder with strong multilingual support.

## What Qwen-Image is unusually good at

- **Text rendering.** Signs, labels, posters, book covers, typographic art — Qwen-Image can place specific words inside the image with high accuracy. If the user wants legible text, suggest Qwen-Image and write the desired text in quotes inside the prompt.
- **Multi-subject scenes.** Detailed scenes with several distinct subjects, props, and spatial relationships hold together better than on most diffusion models.
- **Multilingual.** First-class support for Chinese, English, Japanese, Korean, and others via the Qwen3 encoder. Mix languages if useful.

## Prompt style

Write **descriptive, natural-language sentences** — same general approach as FLUX, not SDXL tag style. Be explicit about subject placement when composing multi-element scenes ("on the left", "in the foreground", "behind the table"). Quote any text that needs to appear in the image: `a hand-painted sign reading "FRESH BREAD"`.

## Parameters

- **CFG Scale**: 1.0 recommended.
- **Steps**: 28 is a good default.
- **VRAM**: High in BF16 (~48 GB). Quantized variants reduce this significantly with minimal quality loss.

## When to suggest Qwen-Image over FLUX

- The user wants legible text in the image.
- The user is composing a complex multi-subject scene with specific spatial relationships.
- The user is prompting in a language other than English and FLUX.2 isn't available.
