# Preview Generation Feature - Status

## Current Status: Phase 1 Complete, Phases 2-3 Pending

**Last Updated:** 2026-01-30

---

## Completed

### Phase 1: Final Preview Only ✅

- Preview generated after VAE decode, before PNG encoding
- Base64-encoded JPEG (quality 75) embedded in progress events
- 80×80px thumbnail displayed in QueueJobCard
- ~5% overhead (~300ms)
- Committed in: `01d47a6 feat: add preview generation and AI chatbot assistant`

**Files modified:**
- `src-tauri/src/inference/progress.rs` - Added `preview_data` field
- `src-tauri/src/inference/flux_pipeline/generation.rs` - Added `encode_jpeg_base64()` and preview generation
- `src/stores/queue.ts` - Added `previewData` field and event listener
- `src/components/generation/QueueJobCard.vue` - Added preview thumbnail display

---

## Pending

### Phase 2: Configuration UI

Add user settings to control preview behavior:

```typescript
interface PreviewSettings {
  enabled: boolean;
  mode: 'final' | 'progressive';
  progressiveInterval: number;  // Every N steps (0 = disabled)
  jpegQuality: number;          // 50-95
}
```

**Files to create/modify:**
- `src-tauri/src/settings/mod.rs` - Add `PreviewSettings` struct
- `src-tauri/src/lib.rs` - Add `get_preview_settings` / `set_preview_settings` commands
- `src/stores/settings.ts` - Create settings store (if not exists)
- `src/views/SettingsView.vue` - Add preview settings panel

**Estimated effort:** 4-5 hours

---

### Phase 3: Progressive Previews During Denoising

Generate previews at intermediate denoising steps (not just final).

**Challenge:** Denoising loop callback doesn't have access to:
1. Current latent tensor state
2. VAE decoder reference

**Solution:** Refactor `denoise()` callback signature in `flux.rs`:

```rust
// Current
on_step: Option<F> where F: Fn(usize, usize)  // (step, total)

// New - expose latent tensor
on_step: Option<F> where F: Fn(usize, usize, &Tensor)  // (step, total, latent)
```

**Key implementation in `flux.rs` denoising loop:**
```rust
// After each step, if interval matches:
if step % preview_interval == 0 {
    let unpacked = flux::sampling::unpack(&img, height, width)?;
    let preview_img = vae.decode(&unpacked)?;
    let rgb = vae.tensor_to_rgb(&preview_img)?;
    let preview = encode_jpeg_base64(&rgb, width, height, quality)?;
    on_progress(GenerationProgress::preview(preview));
}
```

**Files to modify:**
- `src-tauri/src/models/flux.rs` - Modify all 3 sampler implementations (Euler, EulerA, DPM++2M)
- `src-tauri/src/inference/flux_pipeline/generation.rs` - Pass VAE context, implement interval logic

**Performance impact:**
| Mode | Overhead |
|------|----------|
| Final only (current) | ~5% |
| Every 2 steps (4-step Schnell) | 30-40% |
| Every step | 60-80% |

**Estimated effort:** 12-15 hours

---

## Related: Chatbot Relocation (Uncommitted)

The chatbot panel was relocated to slide in from the left, between the sidebar and main content:

- Wand icon (✨) in prompt area toggles panel
- Slides in/out from left with 300ms animation
- Main content resizes to accommodate

**Files modified (uncommitted):**
- `src/components/generation/actions/PromptInput.vue` - Wand button
- `src/views/GenerateView.vue` - Panel positioning and animation

---

## Reference: Original Plan Location

Full implementation plan details were captured in conversation transcript:
`/home/alex/.claude/projects/-home-alex-Dev-Work-rzem-ai-inference/16801a0f-4cee-4fef-b331-a98e9171918a.jsonl`

Search for "Phase 3: Progressive" to find the detailed design.
