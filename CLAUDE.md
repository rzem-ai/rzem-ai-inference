# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working
with code in this repository.

## What This Is

Desktop AI image generation app built with **Electron** + **Vue 3** +
**Vite** (via `electron-vite`). The Electron main process owns the
SQLite database, IPC, discovery and the HTTP/WebSocket client for an
**externally-running** Python inference engine (the sibling
`rzem-ai-inference-engine` repo). This app does **not** spawn or manage
the engine process — it connects over the network to whatever engine is
advertised via mDNS or configured manually.

Cloud image generation through FAL is handled in-process via
`@fal-ai/client` and does not touch the local engine.

## Commands

```bash
# Development
npm run dev            # electron-vite dev (renderer HMR + Electron)
npm run type-check     # vue-tsc (web) + tsc (node/electron) — no emit

# Build & run
npm run build          # type-check web then electron-vite build
npm start              # electron-vite preview (run built output)

# Packaging
npm run package        # build + local installers for host platform
npm run package:linux  # AppImage, deb, snap
npm run package:mac    # dmg (notarized)
npm run package:win    # NSIS
npm run release        # build and publish to GitHub Releases
```

Tests live in `tests/` and are written for **Vitest**, but vitest is
**not currently declared in `package.json`** and there is no
`"test"` npm script. If you need to run them you will have to install
vitest and wire up a script. The current test imports also point at
`../src/main/services/...` which no longer matches the
`electron/main/services/...` layout — assume they are stale until
someone refreshes them.

Prettier (`.prettierrc.cjs` + `prettier-plugin-tailwindcss`) is
configured; there is no separate lint step.

## Architecture

```
Electron App
├── electron/main/                    Main process (TypeScript → Node)
│   ├── index.ts                      Entry: data dir, DB, engine client,
│   │                                 discovery, IPC, window, menu,
│   │                                 auto-updater
│   ├── database.ts                   better-sqlite3 schema + migrations
│   ├── engine-client.ts              HTTP + WebSocket client for the
│   │                                 Python engine (no process mgmt)
│   ├── discovery.ts                  mDNS browser (_rzem-ai._tcp) via
│   │                                 bonjour-service
│   ├── ipc.ts                        ~114 ipcMain.handle channels
│   ├── updater.ts                    electron-updater wiring
│   └── services/
│       ├── batch.ts                  CSV parsing + template rendering
│       ├── bundles.ts                Default bundle catalogue
│       ├── chat.ts                   Anthropic SDK streaming + tool use
│       ├── fal.ts                    FAL cloud generation (@fal-ai/client)
│       ├── files.ts                  Native file dialogs
│       ├── settings.ts               Engine status, paths, cache
│       ├── skills.ts                 Loads resources/skills/*.md for the
│       │                             chat agent (frontmatter + body)
│       ├── styles.ts                 Styles CRUD, LoRA, tags, AI features
│       └── workflow.ts               Workflow DAG executor
│
├── electron/preload/
│   ├── index.ts                      contextBridge → window.electronAPI
│   └── api.d.ts                      Public API surface types
│
├── src/                              Renderer (Vue 3, Vite-built)
│   ├── App.vue, main.ts              Root + bootstrap
│   ├── bridge.ts                     Proxy adapter: snake_case API calls
│   │                                 → camelCase IPC channel names
│   ├── components/                   Shared components
│   ├── composables/                  e.g. usePywebview
│   ├── extensions/, plugins/         Tiptap + PrimeVue plugins
│   ├── pages/                        create, edit, gallery, models,
│   │                                 settings, styles, workflow
│   ├── router/                       vue-router, hash history
│   ├── stores/                       Pinia (Options API)
│   ├── theme/                        Custom PrimeVue "Glass" preset
│   └── types/                        pywebview.d.ts + friends
│
└── resources/                        Bundled via extraResources
    ├── icons/, icon.png              Tray + window icons
    └── skills/                       Markdown skills (flux, sdxl, z-image,
                                      qwen, composition, lighting)
```

### Build layout (electron-vite)

- `electron/main` → `out/main/` (Node, `externalizeDepsPlugin` keeps
  native deps out of the bundle).
- `electron/preload` → `out/preload/` (sandboxed bridge).
- `src/` → `out/renderer/` (Vue app; renderer root is project root,
  entry is `index.html`).

### IPC bridge pattern

Renderer stores were written against a snake_case Python API
(legacy from pywebview / Electrobun days). `src/bridge.ts` wraps the
exposed `window.electronAPI.invoke` behind a `Proxy` that:

1. Converts snake_case method names (`get_bundles`) to camelCase
   channel names (`getBundles`).
2. Converts response keys back camelCase → snake_case so existing
   stores keep working unchanged.

All handlers return `{ status: "success", ... }` or
`{ status: "error", message: "..." }`.

### Event system

The engine pushes events (progress, `job_completed`, etc.) over a
WebSocket. `engine-client.ts` re-emits them; the main process forwards
to the renderer via `webContents.send("inferenceEvent", ...)` and also
maintains a polling buffer so stores can reconcile on reconnect. Image
persistence happens in the main process when `job_completed` arrives.

### Engine discovery & connection

- `discovery.ts` runs a `bonjour-service` browser for
  `_rzem-ai._tcp` and exposes a server list + up/down callbacks.
- The user selects a server (Settings → Network / Remote Servers) or
  enters host/port manually.
- `engine-client.ts` manages the REST/WebSocket lifecycle and
  healthcheck. Nothing spawns a subprocess.

## Key Constraints

- **Hash router**: production loads `file://out/renderer/index.html`,
  so `vue-router` must use `createWebHashHistory`.
- **Vite port 1978**: hardcoded with `strictPort: true` in
  `electron.vite.config.ts`.
- **Main process owns the database**: the engine is stateless; only
  the main process writes to `inference.db` in `app.getPath("userData")`.
- **Preload-only bridge**: all renderer ↔ main communication must go
  through `electron/preload/index.ts` via `contextBridge`. Never enable
  `nodeIntegration`.
- **Native deps unpacked**: `better-sqlite3`, `sharp`,
  `electron-updater` are listed in `electron-builder.yml` under
  `asarUnpack`. Adding other native modules? Add them there too.
- **Renderer path alias**: `@` → `./src` (web tsconfig + vite config).
  The main process has no `@` alias.

## TypeScript configs

- `tsconfig.json` — root, references the two below.
- `tsconfig.web.json` — renderer (`src/**` + `electron/preload/api.d.ts`).
  Extends `@vue/tsconfig/tsconfig.dom.json`.
- `tsconfig.node.json` — main process (`electron/**/*.ts` +
  `electron.vite.config.ts`).

There is no `tsconfig.main.json`; don't invent one.

## Packaging & release

Config lives in `electron-builder.yml`.

- `appId: com.rzem.ai.inference`, product name **Inference**.
- Targets: Linux (AppImage, deb, snap), macOS (dmg, notarized,
  hardened runtime, entitlements in `resources/`), Windows (NSIS).
- Publishes to GitHub Releases at `rzem-ai/rzem-ai-inference`;
  `electron-updater` reads from the same feed at runtime.
- CI in `.github/workflows/` builds on tag push (`v*`) or manual
  dispatch.

---

## Vue 3 Coding Standards

### Core Principles

- **Composition API for components**: `<script setup lang="ts">`.
- **Options API for Pinia stores**: `state`, `getters`, `actions`.
- **Block order**: `<template>`, `<script>`, `<style>`.
- **TypeScript first**: prefer `interface` for object shapes.
- **Named exports** over default exports.
- **Named functions** for methods; arrow functions for callbacks.
- **Comments explain WHY, not WHAT** — default to no comment.

### Component patterns

```vue
<template>
  <ImageCard @image-click="handleImageClick" :image-id />
</template>

<script setup lang="ts">
defineProps<{
  imageId: string;
  width: number;
  height?: number;
}>();

const emit = defineEmits<{
  imageClick: [imageId: string];
  update: [id: string, value: number];
}>();

const prompt = defineModel<string>();
const width = defineModel<number>('width');

function handleClick() {
  emit('imageClick', 'img-123');
}
</script>

<style scoped>
</style>
```

### Styling

- **Primary**: PrimeVue components.
- **Layout**: Tailwind utilities.
- **Custom**: scoped CSS only when PrimeVue/Tailwind won't cut it.

### Tailwind rules

- **No arbitrary pixel values**. Use standard classes:
  `text-[13px]` → `text-base`, `min-h-[60px]` → `min-h-15`.
- **No fractional spacing**. `gap-2.5` → `gap-2`, `mt-0.5` → `mt-1`.
- **Tailwind v4 important suffix**: `opacity-100!`, not `!opacity-100`.
- **Tailwind v4 bare values**: `aspect-4/3`, `columns-2`, not
  bracket syntax.

### Pinia (Options API)

```typescript
export const useQueueStore = defineStore('queue', {
  state: () => ({
    jobs: [] as GenerationJob[],
  }),
  getters: {
    pendingJobs(state): GenerationJob[] {
      return state.jobs.filter(j => j.status === 'pending');
    },
  },
  actions: {
    async loadJobs() {
      this.jobs = await api.get_all_jobs();
    },
  },
});
```

Components consume stores via `storeToRefs()` for reactive refs and
call actions directly. Event listeners are set up and torn down in
actions, invoked from components on mount/unmount.

### Common anti-patterns

**Don't**: call `invoke()` in computed properties, forget to
unsubscribe from events, mutate props directly, mutate store state
from components, use `const props =` unless you actually reference
`props` in script.

**Do**: load once into reactive state, use composables with
`onUnmounted` cleanup, emit update events for two-way binding, call
store actions to mutate state.
