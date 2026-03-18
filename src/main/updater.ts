/**
 * Auto-updater — checks GitHub Releases for new versions and prompts the user.
 * Uses electron-updater which reads the `publish` config from package.json.
 *
 * Flow: check on launch → notify user → download in background → install on quit.
 */

import { autoUpdater } from "electron-updater";
import { dialog, type BrowserWindow } from "electron";

export function initAutoUpdater(getWindow: () => BrowserWindow | null): void {
	// Don't auto-update in dev
	if (process.env.NODE_ENV === "development" || process.env.VITE_DEV_SERVER_URL) {
		return;
	}

	autoUpdater.autoDownload = false;
	autoUpdater.autoInstallOnAppQuit = true;

	autoUpdater.on("update-available", (info) => {
		const win = getWindow();
		if (!win) return;

		dialog
			.showMessageBox(win, {
				type: "info",
				title: "Update Available",
				message: `Version ${info.version} is available. Download now?`,
				buttons: ["Download", "Later"],
				defaultId: 0,
			})
			.then(({ response }) => {
				if (response === 0) {
					autoUpdater.downloadUpdate();
					win.webContents.send("updateEvent", { event: "downloading" });
				}
			});
	});

	autoUpdater.on("update-downloaded", () => {
		const win = getWindow();
		if (!win) return;

		dialog
			.showMessageBox(win, {
				type: "info",
				title: "Update Ready",
				message: "Update downloaded. Restart now to install?",
				buttons: ["Restart", "Later"],
				defaultId: 0,
			})
			.then(({ response }) => {
				if (response === 0) {
					autoUpdater.quitAndInstall();
				}
			});
	});

	autoUpdater.on("error", (err) => {
		console.error("Auto-updater error:", err);
	});

	// Check for updates after a short delay so the window is ready
	setTimeout(() => {
		autoUpdater.checkForUpdates().catch((err) => {
			console.error("Update check failed:", err);
		});
	}, 5000);
}
