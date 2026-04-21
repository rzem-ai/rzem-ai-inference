# RZEM AI Inference

Desktop AI image generation app. Electron + Vue 3 frontend that talks to a
separately-running Python inference engine (the sibling
`rzem-ai-inference-engine` repo) over HTTP + WebSocket.

The engine is **not** bundled or spawned by this app — it runs as an
independent service and is located via mDNS (`_rzem-ai._tcp`) or configured
manually in Settings.

## Stack

- Electron 35 (main process, Node.js + TypeScript)
- Vue 3 + Vite + TypeScript (renderer)
- Pinia for state, PrimeVue 4 + Tailwind CSS 4 for UI
- better-sqlite3 for local storage (gallery, styles, settings)
- electron-vite build tooling, electron-builder for packaging
- electron-updater for auto-update via GitHub Releases

## Repository layout

```
electron/
  main/              Main process (Node). Entry: electron/main/index.ts
    database.ts      better-sqlite3 schema + migrations
    engine-client.ts HTTP + WebSocket client for the inference engine
    discovery.ts     mDNS browser for engines on the LAN
    ipc.ts           ipcMain.handle registrations (~114 RPC methods)
    updater.ts       electron-updater wiring
    services/        batch, bundles, chat, fal, files, settings,
                     skills, styles, workflow
  preload/           contextBridge → window.electronAPI (index.ts + api.d.ts)

src/                 Renderer (Vue). Built by electron-vite's renderer config
  App.vue, main.ts   App root + bootstrap
  bridge.ts          Proxy adapter: api.get_bundles() → invoke("getBundles")
  components/        Shared components (nav, dialogs, editor, etc.)
  pages/             Route pages: create, edit, gallery, models, settings,
                     styles, workflow
  stores/            Pinia stores (Options API)
  composables/, extensions/, plugins/, router/, theme/, types/

resources/           Bundled at build time via electron-builder
  skills/            Markdown skill files consumed by the chat agent
  icons/             Tray + window icons

tests/               Vitest tests (batch, bridge, chat, styles)
electron.vite.config.ts   electron-vite config (main, preload, renderer)
electron-builder.yml      Packaging + publish config
```

## Scripts

```bash
npm run dev           # electron-vite dev (HMR + Electron)
npm run build         # vue-tsc type-check web + electron-vite build
npm start             # electron-vite preview (run the built app)
npm run type-check    # type-check web + node (electron) tsconfigs
npm run package       # build and produce local distributables
npm run package:linux # AppImage / deb / snap
npm run package:mac   # dmg (notarized)
npm run package:win   # NSIS installer
npm run release       # build + publish to GitHub Releases
```

The renderer dev server is on port **1978** (hardcoded,
`strictPort: true`).

## Engine connection

The app does not manage the inference engine process. At runtime:

1. `discovery.ts` browses for `_rzem-ai._tcp` services on the LAN and
   reports them to the renderer.
2. The user picks a server (or enters host/port manually in Settings →
   Network / Remote Servers).
3. `engine-client.ts` opens an HTTP + WebSocket connection and streams
   inference events back to the renderer over IPC.

Cloud generation via FAL (`services/fal.ts`) bypasses the local engine
entirely and calls the FAL API directly using `@fal-ai/client`.

## IPC convention

All renderer → main calls go through `contextBridge` in `electron/preload`
and are exposed on `window.electronAPI`. Handlers return
`{ status: "success", ... }` or `{ status: "error", message: "..." }`.

The `bridge.ts` Proxy in the renderer converts the snake_case API the
Vue stores were written against (`api.get_bundles()`) into the camelCase
channel names registered in `electron/main/ipc.ts`.

## Building and releasing

Tagging a commit `v*` triggers `.github/workflows/` to build Linux
(AppImage / deb / snap), macOS (dmg, notarized) and Windows (NSIS), then
publish to the `rzem-ai/rzem-ai-inference` GitHub Releases feed that
`electron-updater` reads from.

## License

See `LICENSE`.
