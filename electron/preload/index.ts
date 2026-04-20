/**
 * Electron preload script — exposes a safe IPC bridge to the renderer.
 *
 * The renderer calls `window.electronAPI.invoke(channel, args)` which maps
 * to `ipcRenderer.invoke(channel, args)` in the main process.
 *
 * Push messages from main → renderer use `ipcRenderer.on(channel, callback)`.
 */

import { contextBridge, ipcRenderer } from "electron";

contextBridge.exposeInMainWorld("electronAPI", {
	invoke: (channel: string, args?: unknown) =>
		ipcRenderer.invoke(channel, args),

	on: (channel: string, callback: (...args: unknown[]) => void) => {
		const listener = (_event: Electron.IpcRendererEvent, ...args: unknown[]) =>
			callback(...args);
		ipcRenderer.on(channel, listener);
		return () => {
			ipcRenderer.removeListener(channel, listener);
		};
	},
});
