// Renderer-side type declarations for the bridge exposed by
// `electron/preload/index.ts`. The runtime object lives at
// `window.electronAPI` after `contextBridge.exposeInMainWorld(...)`.

export interface ElectronAPI {
  invoke: (channel: string, args?: unknown) => Promise<unknown>;
  on: (channel: string, callback: (...args: unknown[]) => void) => () => void;
}

declare global {
  interface Window {
    electronAPI: ElectronAPI;
  }
}

export {};
