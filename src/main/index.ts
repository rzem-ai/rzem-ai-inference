/**
 * Electron main process entry point.
 * Replaces src/bun/index.ts (Electrobun).
 */

import { app, BrowserWindow, Menu, ipcMain } from "electron";
import { autoUpdater } from "electron-updater";
import { mkdirSync, existsSync } from "fs";
import { join } from "path";
import path from "path";

import { initDatabase } from "./database";
import { EngineClient } from "./engine-client";
import { createDiscoveryService, type DiscoveryService } from "./discovery";
import { registerIpcHandlers } from "./ipc";
import { createFalService } from "./services/fal";
import { initAutoUpdater } from "./updater";

// ── Data directory ───────────────────────────────────────────────
const dataDir = app.getPath("userData");
if (!existsSync(dataDir)) {
	mkdirSync(dataDir, { recursive: true });
}

// ── Database ─────────────────────────────────────────────────────
const dbPath = join(dataDir, "inference.db");
let db: ReturnType<typeof initDatabase>;

// ── Paths ────────────────────────────────────────────────────────
let outputDir: string;
let stylesDir: string;

// ── Engine client & discovery ────────────────────────────────────
let engineClient: EngineClient;
let discoveryService: DiscoveryService;
let mainWindow: BrowserWindow | null = null;

// ── Window creation ──────────────────────────────────────────────
function createWindow() {
	mainWindow = new BrowserWindow({
		title: "RZEM AI Inference",
		width: 1400,
		height: 900,
		x: 100,
		y: 100,
		webPreferences: {
			preload: path.join(__dirname, "preload.js"),
			contextIsolation: true,
			nodeIntegration: false,
		},
	});

	// Load the app
	const VITE_DEV_PORT = 1978;
	const VITE_DEV_URL = `http://localhost:${VITE_DEV_PORT}`;

	if (process.env.NODE_ENV === "development" || process.env.VITE_DEV_SERVER_URL) {
		const devUrl = process.env.VITE_DEV_SERVER_URL || VITE_DEV_URL;
		mainWindow.loadURL(devUrl).catch(() => {
			// Vite dev server not running — fall back to built assets
			console.log("Vite dev server not running — using built assets.");
			mainWindow!.loadFile(join(__dirname, "..", "renderer", "index.html"));
		});
	} else {
		mainWindow.loadFile(join(__dirname, "..", "renderer", "index.html"));
	}

	mainWindow.on("closed", () => {
		mainWindow = null;
	});
}

// ── Application menu ─────────────────────────────────────────────
function createMenu() {
	const appName = "Inference";
	const template: Electron.MenuItemConstructorOptions[] = [
		{
			label: appName,
			submenu: [
				{
					label: `About ${appName}`,
					click: () => app.showAboutPanel(),
				},
				{ type: "separator" },
				{ label: `Hide ${appName}`, role: "hide" },
				{ role: "hideOthers" },
				{ role: "unhide" },
				{ type: "separator" },
				{ label: `Quit ${appName}`, role: "quit" },
			],
		},
		{
			label: "Edit",
			submenu: [
				{ role: "undo" },
				{ role: "redo" },
				{ type: "separator" },
				{ role: "cut" },
				{ role: "copy" },
				{ role: "paste" },
				{ role: "selectAll" },
			],
		},
		{
			label: "View",
			submenu: [
				{ role: "reload" },
				{ role: "forceReload" },
				{ role: "toggleDevTools" },
				{ type: "separator" },
				{ role: "resetZoom" },
				{ role: "zoomIn" },
				{ role: "zoomOut" },
				{ type: "separator" },
				{ role: "togglefullscreen" },
			],
		},
		{
			label: "Window",
			submenu: [{ role: "minimize" }, { role: "zoom" }],
		},
	];

	const menu = Menu.buildFromTemplate(template);
	Menu.setApplicationMenu(menu);
}

// ── App lifecycle ────────────────────────────────────────────────
app.whenReady().then(() => {
	// Initialize database
	db = initDatabase(dbPath);
	console.log(`Database: ${dbPath}`);

	// Resolve paths
	const customOutputDir = db
		.prepare("SELECT value FROM settings WHERE key = 'OUTPUT_DIR'")
		.get() as { value: string } | null;
	outputDir = customOutputDir?.value ?? join(dataDir, "output");
	stylesDir = join(dataDir, "styles");

	for (const dir of [outputDir, stylesDir]) {
		if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
	}

	// Create engine client (reads stored host/port or defaults to localhost:8100)
	const storedHost = (db.prepare("SELECT value FROM settings WHERE key = 'ENGINE_HOST'").get() as { value: string } | null)?.value ?? "127.0.0.1";
	const storedPort = parseInt((db.prepare("SELECT value FROM settings WHERE key = 'ENGINE_PORT'").get() as { value: string } | null)?.value ?? "8100", 10);
	engineClient = new EngineClient({ host: storedHost, port: storedPort });
	discoveryService = createDiscoveryService();

	// Create FAL cloud service
	const falService = createFalService({ outputDir });

	// Register IPC handlers
	registerIpcHandlers(db, engineClient, { outputDir, stylesDir }, () => mainWindow, falService, discoveryService);

	// Set dock icon in dev mode (packaged app uses icon from .app bundle)
	if (!app.isPackaged && process.platform === "darwin") {
		app.dock?.setIcon(join(app.getAppPath(), "resources", "icon.png"));
	}

	// Create window
	app.setAboutPanelOptions({
		applicationName: "Inference",
		applicationVersion: app.getVersion(),
		version: "",
		copyright: "RZEM AI",
		iconPath: join(app.getAppPath(), "resources", "icon.png"),
	});
	createMenu();
	createWindow();

	// Auto-updater (production only)
	initAutoUpdater(() => mainWindow);

	// Start mDNS discovery — auto-connect when an engine is found on the network
	discoveryService.start();
	discoveryService.onServerUp(async (server) => {
		if (!engineClient.ready) {
			console.log(`Auto-connecting to discovered engine: ${server.name} at ${server.host}:${server.port}...`);
			try {
				await engineClient.connect(server.host, server.port);
				db.prepare("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value").run("ENGINE_HOST", server.host);
				db.prepare("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value").run("ENGINE_PORT", String(server.port));
			} catch (err: any) {
				console.log(`Failed to auto-connect to discovered engine: ${err.message}`);
			}
		}
	});

	// Auto-update: check after app has fully loaded
	if (app.isPackaged) {
		setTimeout(() => {
			autoUpdater.checkForUpdatesAndNotify().catch(() => {});
		}, 5000);

		autoUpdater.on("update-available", (info) => {
			mainWindow?.webContents.send("updateAvailable", { version: info.version });
		});

		autoUpdater.on("download-progress", (progress) => {
			mainWindow?.webContents.send("updateProgress", { percent: Math.round(progress.percent) });
		});

		autoUpdater.on("update-downloaded", (info) => {
			mainWindow?.webContents.send("updateDownloaded", { version: info.version });
		});
	}

	app.on("activate", () => {
		if (BrowserWindow.getAllWindows().length === 0) createWindow();
	});
});

app.on("window-all-closed", () => {
	if (process.platform !== "darwin") {
		app.quit();
	}
});

app.on("before-quit", () => {
	discoveryService?.stop();
	engineClient?.disconnect();
	db?.close();
});

console.log("RZEM AI Inference starting...");
