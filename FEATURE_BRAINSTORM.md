# Feature Brainstorm — rzem-ai-inference

> Generated 2026-02-20 by a 4-agent analysis team exploring UX, backend, creative workflow, and gallery systems.

---

## Executive Summary

Four specialized agents analyzed the codebase independently, examining frontend UX, backend capabilities, creative workflows (styles/bundles/LoRAs), and the gallery/image management system. The findings converge on **five strategic themes**:

1. **Comparison & Iteration Tools** — Users need A/B views and quick variation workflows to iterate faster
2. **Organization & Discovery** — Folder hierarchy, smart collections, ratings, and better LoRA/style browsing
3. **Style-Bundle Integration** — Bridge the gap between aesthetic styles and model configurations
4. **Analytics & History** — Surface generation insights, parameter trends, and timeline views
5. **Batch & Queue Enhancements** — Expand batch generation, add persistent queues, and style matrices

---

## Tier 1: High Impact, Low Effort (Quick Wins)

These features leverage existing schema/infrastructure and can be built with minimal changes.

### 1.1 Folder Hierarchy UI

**What**: Expand the flat folder list into a collapsible tree view with drag-drop reorganization.

**Why**: The `folders` table already has a `parent_id` column supporting nesting — this is purely a frontend gap. Every image management tool (Eagle, Lightroom, Bridge) uses folder trees.

**Scope**: Frontend only. Add tree rendering to gallery sidebar, drag-drop reordering.

---

### 1.2 Star Ratings (1-5)

**What**: Replace or supplement the binary `favorite` flag with a 1-5 star rating system.

**Why**: Binary favorites don't scale — users need granularity to curate large galleries. Rating filters ("show 4+ stars") enable quality-based workflows.

**Scope**: Add `rating INTEGER DEFAULT 0` to images table, 2 API methods, star widget in gallery cards + detail dialog.

---

### 1.3 Inline Gallery Metadata

**What**: Show key metadata (generation time, model name, step count) as small badges/chips directly on gallery cards.

**Why**: Currently all metadata is buried in a modal dialog. Surfacing 2-3 key stats on the card reduces clicks and aids visual scanning.

**Scope**: Frontend only — data already available in gallery store.

---

### 1.4 Sampler/Scheduler Tooltips

**What**: Add brief explanations and model-specific recommendations to sampler and scheduler dropdowns.

**Why**: Users don't know the difference between "euler" and "euler_a" or when to use "karras" scheduler. Quick tooltips reduce trial-and-error.

**Scope**: Frontend only — static tooltip content.

---

### 1.5 "Recommended Bundle" Field on Styles

**What**: Add an optional `preferred_bundle_id` to styles, displayed as a hint during generation.

**Why**: Styles and bundles are independent — users don't know which model works best with a given style. A soft recommendation bridges this gap without enforcing it.

**Scope**: 1 column addition to `styles` table, small UI hint in StyleSelect.

---

### 1.6 Folder Colors & Icons

**What**: Surface the existing `color` and `icon` columns in the folder UI.

**Why**: The database schema already stores these — they're just not rendered. Color-coded folders improve visual navigation at zero backend cost.

**Scope**: Frontend only.

---

## Tier 2: High Impact, Medium Effort

Features that require meaningful work but deliver substantial value.

### 2.1 A/B Image Comparison View

**What**: Side-by-side comparison mode for 2-4 images with synchronized pan/zoom, showing parameter diffs.

**Why**: This is the #1 gap identified across all four analyses. Image generation is inherently iterative — users constantly need to compare outputs. Currently they open detail dialogs one at a time and try to remember differences.

**Scope**: New lightbox/modal component, parameter diff rendering, selection UX in gallery.

---

### 2.2 Quick Variations UI

**What**: "Make Variations" dialog that locks the prompt and lets users tweak one parameter (seed, steps, CFG, sampler) to generate a 2x2 or 3x3 grid of variants.

**Why**: The most common workflow is "I like this, but want to explore nearby parameter space." Currently users must manually change params and regenerate one at a time.

**Scope**: New dialog component, extends batch generation logic, auto-groups results.

---

### 2.3 Generation Series / Auto-Grouping

**What**: Automatically group related generations (same prompt hash, same seed family, same session) into browsable series.

**Why**: After a generation session, users have dozens of related images scattered in chronological order with no grouping. Series let users see their exploration path.

**Scope**: Add `series_id` or `prompt_hash` column, grouping logic in gallery store, collapsible group headers in grid.

---

### 2.4 Smart Collections (Saved Searches)

**What**: Save filter/search combinations as named virtual folders. Examples: "FLUX.2 portraits", "High-step quality renders", "Today's generations."

**Why**: Users repeatedly apply the same filters. Saved searches turn one-time filters into permanent organizational tools.

**Scope**: New `collections` table (name, filter JSON), collection list in gallery sidebar, load/save filter state.

---

### 2.5 Prompt Snippets Library

**What**: Save, tag, and quick-insert reusable prompt fragments. Examples: "cinematic lighting, 8k, photorealistic", "watercolor style, soft edges."

**Why**: Users type the same quality/style modifiers repeatedly. A snippet library with keyboard shortcuts (e.g., type `/cin` → expand to "cinematic lighting") speeds up prompt writing.

**Scope**: New `snippets` table, autocomplete integration in Tiptap editor, snippet management UI.

---

### 2.6 Visual Style Browser

**What**: Replace the text-heavy style list with a visual carousel/grid showing style preview thumbnails and example outputs.

**Why**: Styles are visual by nature — a text list of names doesn't communicate aesthetic. Users should browse styles the way they browse a color palette.

**Scope**: Style cards with thumbnail display, example image carousel, preview-before-apply.

---

### 2.7 LoRA Strength Presets

**What**: Named strength presets for LoRAs — "Subtle" (0.3), "Balanced" (0.7), "Strong" (1.0), "Maximum" (1.5) — plus per-LoRA recommended ranges.

**Why**: LoRA strength is the most trial-and-error parameter. Users don't know that 0.3 vs 0.8 can mean the difference between "hint of style" and "overwhelming artifact." Quick presets reduce guesswork.

**Scope**: Frontend preset buttons on LoRA strength slider, optional `recommended_strength` field on LoRA records.

---

### 2.8 Capture Examples During Generation

**What**: One-click "Save as Example" button on generated images that creates a style example with full generation metadata (seed, steps, sampler, bundle).

**Why**: Currently examples only come from CivitAI imports. Users should build their own example galleries from successful generations, creating a feedback loop for style refinement.

**Scope**: New API method, button in image detail dialog and history strip, links to existing `examples` table.

---

### 2.9 Post Image to Bluesky

**What**: One-click "Share to Bluesky" button from the image detail dialog or gallery context menu. Compose a post with the generated image attached, pre-filled with prompt text (truncated/edited), and optional alt text.

**Why**: Sharing generated images to social platforms is a core part of the AI art workflow. Bluesky's open AT Protocol makes integration straightforward — no opaque API approval process. Users currently must manually download, open Bluesky, upload, and type context. A direct share flow keeps them in the app.

**Scope**: Backend OAuth or app password auth flow for Bluesky (AT Protocol), new `post_to_bluesky()` API method, image upload via blob endpoint, compose dialog in frontend with editable post text and alt text. Store auth credentials in settings table.

---

### 2.10 Batch Generation with Style Matrix

**What**: Extend the batch dialog to support a prompt x style matrix — generate each prompt with multiple styles for systematic comparison.

**Why**: The current batch system varies only seed. Style comparison is one of the most valuable creative workflows ("which style makes this prompt look best?").

**Scope**: Extend BatchDialog UI, modify batch submission to iterate over styles, auto-tag results.

---

## Tier 3: High Impact, High Effort (Roadmap)

Strategic features that require significant development but define the product's direction.

### 3.1 Persistent Job Queue with Priority

**What**: A visible, persistent queue where users can add, reorder, pause, resume, and cancel individual jobs. Jobs survive app restarts.

**Why**: Currently only one job runs at a time with no queue visibility. Power users want to set up 20 generations and walk away. Queue management is table-stakes for production image generation tools.

**Scope**: New `job_queue` table, queue management API, queue panel in UI, background job processing.

---

### 3.2 Style Composition & Blending

**What**: Select 2-3 styles and blend their prompt templates with adjustable weights. "40% portrait style + 60% impressionist."

**Why**: Real creative work often combines aesthetics. Currently styles are mutually exclusive — you pick one. Blending unlocks complex aesthetics without creating hundreds of single-purpose styles.

**Scope**: Template merging logic, weight UI, new composite style type, LoRA combination from multiple styles.

---

### 3.3 Style Versioning & Branching

**What**: Save style edits as new versions, compare old vs new, rollback. "Save as variant" creates a branch without losing the original.

**Why**: Styles are iteratively refined, but editing is destructive. One bad change means manually recreating from memory. Versioning adds a safety net.

**Scope**: New `style_versions` table, version history UI, diff view, rollback logic.

---

### 3.4 LoRA Discovery & Organization

**What**: In-app LoRA browser with tagging, categories, preview thumbnails, recommended strengths, and search. Browse without leaving the generation page.

**Why**: Currently LoRA management requires filesystem browsing or external sites. An integrated browser with metadata and previews makes LoRAs a first-class creative tool.

**Scope**: Extend `loras` table with metadata, sidebar browser component, preview generation pipeline, tag system.

---

### 3.5 Generation Analytics Dashboard

**What**: Timeline view with daily/weekly summaries, parameter usage heatmaps, model/LoRA performance stats, generation time trends.

**Why**: Users generate hundreds of images but have no way to analyze patterns. "Which sampler gives me the best results?" "How much time did I spend generating this week?" Analytics turn generation data into creative insights.

**Scope**: New analytics page, aggregation queries, chart components (likely Chart.js or similar), date range filtering.

---

### 3.6 Workflow Snapshots & Sessions

**What**: Save the entire generation state (bundle + style + params + prompt + LoRAs) as a named snapshot. Group generations into named sessions ("Portrait Exploration", "Landscape Study").

**Why**: Users lose their "creative context" between sessions. Snapshots let them pick up exactly where they left off. Sessions organize related work without manual folder management.

**Scope**: New `snapshots` and `sessions` tables, save/load state logic, session browser UI.

---

### 3.7 Cloud GPU Deployment (RunPod / Cloud Providers)

**What**: One-click deployment of the inference engine to cloud GPU platforms (RunPod, etc.), with lifecycle management, cost tracking, secure tunneling, and automatic connection from the desktop app.

**Why**: Not everyone has a local GPU capable of running large diffusion models. Users with integrated graphics, older GPUs, or laptops need cloud GPU access. Currently the app supports remote servers on the LAN via Zeroconf discovery — but cloud GPUs aren't on the LAN. This feature bridges that gap: the app itself manages the full cloud lifecycle so users never touch a terminal, SSH session, or cloud dashboard.

**The Core Problem**: How do you let a non-technical user rent a GPU in the cloud, deploy a Docker image to it, connect securely, generate images, and shut it down — all without leaving the app? And how do you do this *safely* so they don't accidentally leave a $2/hour GPU running overnight?

---

#### Deployment Model: RunPod Pods (not Serverless)

**Why Pods over Serverless**: The existing `RemoteInferenceService` relies on a persistent WebSocket connection for real-time progress events (step-by-step previews, job status). RunPod Serverless endpoints are stateless request-response — they don't support long-lived WebSocket connections. Pods are persistent VMs with full networking, making them compatible with the existing HTTP REST + WebSocket protocol unchanged.

| Aspect | RunPod Pods | RunPod Serverless |
|---|---|---|
| WebSocket support | Yes (full networking) | No (stateless HTTP only) |
| Cold start | ~5s with FlashBoot | ~1-12s depending on container |
| Billing | Per-hour while running | Per-second of compute |
| Session state | Maintained | Stateless per request |
| Cost risk | High if left running | Low (auto-scales to zero) |
| Protocol compatibility | Drop-in with existing RemoteInferenceService | Would require new polling-based protocol |

**Verdict**: Start with Pods for compatibility. Consider a Serverless adapter later for users who prefer pay-per-image pricing (would require replacing WebSocket events with polling or webhooks).

---

#### Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│  Desktop App (pywebview)                            │
│                                                     │
│  ┌─────────────┐  ┌──────────────────────────────┐  │
│  │ CloudAPI     │  │ InferenceServiceManager      │  │
│  │ (new mixin)  │  │ (existing)                   │  │
│  │              │  │                              │  │
│  │ • provision  │  │  local ←→ remote ←→ cloud    │  │
│  │ • status     │  │  (extends remote with cloud  │  │
│  │ • teardown   │  │   lifecycle awareness)       │  │
│  │ • cost track │  │                              │  │
│  └──────┬───────┘  └──────────────┬───────────────┘  │
│         │                         │                  │
│    RunPod REST API          HTTP + WebSocket         │
│    (pod lifecycle)          (inference protocol)     │
│         │                         │                  │
└─────────┼─────────────────────────┼──────────────────┘
          │                         │
          ▼                         ▼
   ┌──────────────┐      ┌──────────────────┐
   │ RunPod API   │      │ Cloud Pod        │
   │ rest.runpod  │      │ (Docker container │
   │ .io/v1       │◄────►│  running          │
   │              │      │  inference engine) │
   └──────────────┘      └──────────────────┘
```

**Key Insight**: The cloud deployment feature is a *lifecycle management layer* on top of the existing remote inference architecture. The inference protocol (HTTP REST + WebSocket) stays identical — only the provisioning, teardown, and connection setup are new.

---

#### Security Architecture (Critical)

This is the most sensitive part of the feature. Users are storing API keys that control paid cloud resources.

**1. API Key Storage**

```
NEVER: Plaintext in bundles.json, settings table, or config files
NEVER: Hardcoded or logged

INSTEAD:
├── Linux: libsecret (GNOME Keyring / KWallet) via `keyring` Python package
├── macOS: Keychain Access via `keyring` Python package
└── Windows: Windows Credential Locker via `keyring` Python package

Fallback: OS-encrypted config file with restrictive permissions (0600)
```

The `keyring` Python package provides a cross-platform abstraction over OS-native credential stores. The RunPod API key is stored as a secret, never in the SQLite database or JSON files.

**2. Scoped API Keys (Least Privilege)**

RunPod supports scoped API keys with per-endpoint permissions. The app should guide users to create a *restricted* key with only pod management permissions — not full account access. The onboarding flow should include:

- Link to RunPod's API key creation page
- Instructions to select "Restricted" permission level
- Validation that the key has sufficient (but not excessive) permissions

**3. Secure Connection to the Pod**

The inference engine server inside the pod needs to be accessible but not exposed to the open internet:

| Approach | Security | Complexity | Latency |
|---|---|---|---|
| **RunPod's built-in proxy** | Good — traffic routed through RunPod's HTTPS proxy | Low | +10-30ms |
| **WireGuard/Tailscale tunnel** | Excellent — encrypted P2P tunnel, no public ports | Medium | +5ms |
| **Direct HTTPS + auth token** | Moderate — requires TLS cert management, token auth | Medium | Lowest |
| **SSH tunnel** | Good — standard, well-understood | Medium | +10ms |

**Recommended approach**: Use **RunPod's built-in HTTPS proxy** initially (simplest), with an **auth token** injected as a pod environment variable that the desktop app must present on every request. This avoids exposing raw ports to the internet while keeping the implementation simple.

```
Pod startup:
  1. App generates a random 256-bit auth token
  2. Token passed as RZEM_AUTH_TOKEN environment variable when creating pod
  3. Engine server validates token on every HTTP request and WS handshake
  4. App stores token in memory only (not persisted) — lost on app restart, regenerated on reconnect
```

**4. Cost Protection & Runaway Prevention**

This is the #1 safety concern. A user who forgets to shut down a pod could rack up significant charges.

| Protection | Implementation |
|---|---|
| **Auto-shutdown timer** | Default 30-minute idle timeout. Pod terminates if no jobs submitted within window. Configurable by user. |
| **Session budget cap** | User sets max spend per session (e.g., $5). App estimates cost from RunPod pricing API and warns before exceeding. |
| **Active cost ticker** | Real-time cost display in the UI: "$0.47 spent · Running for 14 min · ~$2.01/hr" |
| **Shutdown confirmation** | When user closes the app, prominent warning if a cloud pod is still running. "Your RunPod GPU is still running at $2.01/hr. Shut it down?" |
| **App crash safety** | On next app launch, check for orphaned pods via RunPod API and offer to terminate them. |
| **Heartbeat watchdog** | Pod-side watchdog: if no heartbeat from desktop app for 5 minutes (app crashed/network lost), pod self-terminates. |

---

#### User Flow

**First-time setup:**
1. User navigates to Settings → Cloud GPU
2. Enters RunPod API key (stored in OS keychain)
3. App validates key permissions via RunPod API
4. Key status shown: "Connected to RunPod · 3 GPU types available"

**Deploying a pod:**
1. User clicks "Launch Cloud GPU" in connection panel
2. Selects GPU tier: A4000 ($0.38/hr), RTX 4090 ($0.69/hr), A100 ($1.64/hr), etc.
3. App shows estimated cost: "~$0.69/hr · Auto-shutdown after 30 min idle"
4. User confirms → App calls RunPod REST API:
   - `POST /v1/pods` with Docker image, GPU type, env vars (auth token, model config)
   - Polls pod status until `RUNNING`
5. **Notification**: Status bar shows "Cloud GPU starting..." → "Connected to RTX 4090 on RunPod"
6. `InferenceServiceManager` switches to remote mode automatically
7. User generates images normally — no difference from LAN server

**During session:**
- Status bar: "☁ RunPod RTX 4090 · $1.38 spent · 2h 01m · 47 images"
- Periodic heartbeat from app to pod (keeps watchdog alive)
- All inference events stream over WebSocket as usual

**Teardown:**
1. User clicks "Stop Cloud GPU" or closes app
2. App calls `POST /v1/pods/{pod-id}/stop`
3. Manager switches back to local mode
4. Final cost summary: "Session complete · $2.76 for 2h 01m · 47 images generated"

---

#### Docker Image Strategy

The inference engine needs to be packaged as a Docker image that RunPod can pull:

```dockerfile
# Minimal container for rzem-ai-inference-engine serve mode
FROM pytorch/pytorch:2.x-cuda12.x-cudnn9-runtime

# Install engine + serve dependencies only (no pywebview, no frontend)
COPY engine/ /app/engine/
RUN pip install -e /app/engine[serve]

# Pre-download default model weights (optional, reduces cold start)
# RUN python -c "from engine import preload; preload('flux-schnell')"

EXPOSE 8188
ENV RZEM_AUTH_TOKEN=""

CMD ["rzem-ai-inference-engine", "serve", "--host", "0.0.0.0", "--port", "8188"]
```

**Image hosting options:**
- Docker Hub (public, free for open-source)
- GitHub Container Registry (tied to the repo, free for public images)
- RunPod's container registry (if available)

**Cold start optimization**: Pre-bake the most common model weights into the Docker image. This increases image size (~10-20GB) but eliminates the 5-10 minute model download on first boot. Alternatively, use RunPod network volumes to persist downloaded models across pod restarts.

---

#### Backend Components (New)

| Component | Purpose |
|---|---|
| `backend/services/cloud_provider.py` | Abstract base for cloud GPU providers (RunPod first, extensible to Vast.ai, Lambda, etc.) |
| `backend/services/runpod_provider.py` | RunPod-specific implementation: pod CRUD, status polling, cost estimation |
| `backend/api/cloud.py` | CloudAPI mixin: `deploy_cloud_gpu()`, `get_cloud_status()`, `stop_cloud_gpu()`, `get_cloud_cost()` |
| `backend/services/cloud_watchdog.py` | Heartbeat sender + orphaned pod detector on startup |

**Integration with existing code:**
- `InferenceServiceManager` gains a `connect_cloud(provider, gpu_tier)` method that provisions first, then calls existing `connect_remote(host, port)`
- `RemoteInferenceService` unchanged — cloud is just another remote server
- `DiscoveryAPI` extended to show cloud pods alongside LAN servers in the UI

---

#### Frontend Components (New)

| Component | Purpose |
|---|---|
| `pages/settings/CloudGPU.vue` | API key setup, provider selection, GPU tier browser |
| `components/CloudStatusBar.vue` | Persistent cost ticker + connection status in header |
| `stores/cloud.ts` | Cloud state: provider config, active pod, cost tracking, heartbeat |

---

#### Extensibility: Other Providers

The provider abstraction should support adding other cloud GPU platforms:

| Provider | API Style | Pod-like Support | Notes |
|---|---|---|---|
| **RunPod** | REST | Yes (Pods) | First target, best GPU marketplace |
| **Vast.ai** | REST | Yes (Instances) | Often cheapest, community GPUs |
| **Lambda Cloud** | REST | Yes (Instances) | Simple API, limited availability |
| **Replicate** | REST | Serverless only | Would need polling adapter |
| **fal.ai** | REST | Serverless only | Already partially supported in bundles |

Each provider implements a `CloudProvider` protocol: `provision()`, `status()`, `terminate()`, `estimate_cost()`.

---

#### Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| User leaves pod running | Financial ($$) | Auto-shutdown timer, app-close warning, heartbeat watchdog, orphan detection on startup |
| API key leaked | Account compromise | OS keychain storage, never log/persist in DB, scoped keys with minimal permissions |
| Pod exposed to internet | Unauthorized GPU usage | Auth token on all requests, RunPod proxy (no direct port exposure) |
| Network interruption | Lost connection mid-generation | Reconnect logic, pod keeps running (watchdog has 5-min grace), images saved server-side |
| Docker image too large | Slow cold start | Base image without models + RunPod network volumes for model persistence |
| RunPod API changes | Feature breakage | Version-pinned API calls, provider abstraction isolates changes |

---

#### Scope Estimate

| Layer | Effort |
|---|---|
| Cloud provider abstraction + RunPod implementation | Medium |
| Auth token injection + engine-side validation | Low |
| API key management (OS keychain) | Low-Medium |
| CloudAPI mixin + frontend settings page | Medium |
| Cost tracking + status bar | Medium |
| Watchdog + orphan detection | Medium |
| Docker image + CI pipeline for builds | Medium |
| Testing with real RunPod account | Ongoing |

**Total**: 3-4 weeks of focused development. The inference protocol itself requires zero changes — the entire feature is orchestration and UX around the existing `RemoteInferenceService`.

---

### 3.8 Image Post-Processing Pipeline

**What**: Apply transformations to generated images — upscaling, format conversion, metadata embedding, watermarking, batch resize.

**Why**: Currently users must use external tools for any post-processing. Even basic operations like "export as JPG at 80% quality" require leaving the app.

**Scope**: Post-processing service, transformation UI, export presets, format support (JPG, WebP, PNG with metadata).

---

## Tier 4: Polish & Quality of Life

Lower-priority features that round out the experience.

### 4.1 User Preference Defaults

Set default model, aspect ratio, style, and sampler so the app starts ready-to-generate.

### 4.2 Generation Time Estimation

Show estimated generation time before submitting based on historical data for similar parameters.

### 4.3 Tag Categories in Gallery Filter

Leverage existing `category` column on tags to group filters (e.g., "Subject", "Quality", "Project").

### 4.4 Keyboard Shortcuts & Power User Mode

Customizable hotkeys for common actions (generate, next image, toggle comparison, apply style).

### 4.5 Export Styles as JSON

Share style definitions (template + LoRA refs + examples) as portable JSON files.

### 4.6 Thumbnail Caching

Pre-generate and cache thumbnails at image creation time instead of loading full base64 on demand.

### 4.7 Model Comparison Modal

Side-by-side model specs, VRAM requirements, and capabilities for informed bundle selection.

### 4.8 Image Duplicate Detection

Perceptual hashing to find visually similar or identical images for cleanup.

### 4.9 Batch Metadata Editing

Edit tags, ratings, folders, and custom fields across multiple selected images in one operation.

### 4.10 Performance Profiling Dashboard

Track model speed, VRAM usage per configuration, and cost comparisons for cloud vs local.

---

## Architecture Notes

### What's Already Built (Leverage These)

| Capability | Status | Location |
|---|---|---|
| Folder hierarchy (parent_id) | Schema ready, UI missing | `folders` table |
| Tag categories & colors | Schema ready, UI partial | `tags` table |
| Polymorphic examples | Schema ready | `examples` table (entity_type/entity_id) |
| Model config snapshots | Fully working | `images.model_config` JSON |
| LoRA tracking per image | Fully working | `images.loras` JSON |
| Local/remote inference switching | Fully working | InferenceServiceManager |
| Batch CSV templates | Fully working | BatchAPI |
| CivitAI style import | Fully working | StylesAPI |
| Thread-safe DB | Fully working | threading.Lock + WAL mode |

### Schema Changes Needed

| Feature | Change | Complexity |
|---|---|---|
| Star ratings | `ALTER TABLE images ADD rating INTEGER DEFAULT 0` | Trivial |
| Recommended bundle | `ALTER TABLE styles ADD preferred_bundle_id TEXT` | Trivial |
| Smart collections | New `collections` table (name, filters JSON) | Low |
| Generation series | `ALTER TABLE images ADD series_id TEXT` | Low |
| Prompt snippets | New `snippets` table | Low |
| Workflow snapshots | New `snapshots` table | Medium |
| Style versions | New `style_versions` table | Medium |
| Job queue | New `job_queue` table | Medium |
| Bluesky sharing | Bluesky credentials in `settings` table | Low |
| Cloud GPU deployment | New `cloud_sessions` table (provider, pod_id, cost, timestamps) | Medium |

### Key Technical Constraints

- **No backward compatibility needed** — app not yet released, can delete DB for schema changes
- **PyWebView bridge** — all API methods must follow camelCase→snake_case convention via ApiMeta
- **Hash router required** — pywebview has no SPA fallback
- **Synchronous SQLite** — thread-safe with Lock, fast enough for all proposed features
- **Event polling at 200ms** — adequate for queue updates, may need optimization for real-time comparison

---

## Recommended Implementation Order

### Phase 1: Quick Wins (1-2 weeks)

1. Folder hierarchy UI (leverages existing schema)
2. Star ratings (trivial schema + UI)
3. Inline gallery metadata badges
4. Sampler/scheduler tooltips
5. Recommended bundle on styles
6. Folder colors & icons

### Phase 2: Core Iteration (3-4 weeks)

7. A/B image comparison view
2. Quick variations UI
3. Prompt snippets library
4. Capture examples during generation
5. LoRA strength presets

### Phase 3: Organization (2-3 weeks)

12. Smart collections (saved searches)
2. Generation series / auto-grouping
3. Visual style browser
4. Batch style matrix

### Phase 4: Advanced Creative & Cloud (4-6 weeks)

16. Persistent job queue
2. Style composition & blending
3. LoRA discovery & organization
4. Workflow snapshots & sessions
5. Cloud GPU deployment (RunPod) — leverages existing remote infrastructure

### Phase 5: Analytics & Polish (3-4 weeks)

20. Generation analytics dashboard
2. Post-processing pipeline
3. Style versioning
4. User preference defaults
5. Export/sharing features (including Bluesky posting)

---

## Cross-Cutting Themes

### Iteration Speed

The single biggest improvement area. Users need to go from "I generated an image" to "I explored 20 variations" in minutes, not hours. A/B comparison, quick variations, and prompt snippets all serve this goal.

### Organization at Scale

The app works fine for 50 images but will struggle at 500+. Star ratings, smart collections, series grouping, and nested folders prepare for scale.

### Creative Discovery

Styles and LoRAs are powerful but opaque. Visual browsers, strength presets, recommendations, and analytics help users discover what works without blind trial-and-error.

### Reproducibility

The app already captures excellent metadata (model_config snapshots, LoRA arrays). Workflow snapshots, style versioning, and session management build on this foundation to make every generation reproducible and shareable.

### Accessibility Beyond Hardware

Not every user has an RTX 4090. Cloud GPU deployment democratizes access — users with integrated graphics or older laptops can still use the full app by renting cloud GPUs on demand. The existing `RemoteInferenceService` already handles the protocol; cloud deployment adds lifecycle management around it. The safety guardrails (auto-shutdown, cost tracking, orphan detection) are essential to prevent bill shock.
