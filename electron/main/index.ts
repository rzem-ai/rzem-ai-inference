/**
 * Electron main process entry point.
 */

import { app, BrowserWindow, Menu } from "electron";
import { mkdirSync, existsSync } from "fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

import { initDatabase } from "./database";
import { EngineClient } from "./engine-client";
import { createDiscoveryService, type DiscoveryService } from "./discovery";
import { registerIpcHandlers } from "./ipc";
import { createFalService } from "./services/fal";
import { backfillOutputThumbnails } from "./services/thumbnails";
import { initAutoUpdater } from "./updater";

// ESM doesn't have __dirname. Resolve from import.meta.url for
// electron-vite's bundled output.
const __dirname = dirname(fileURLToPath(import.meta.url));

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
			preload: join(__dirname, "../preload/index.mjs"),
			sandbox: false,
			contextIsolation: true,
			nodeIntegration: false,
		},
	});

	// electron-vite injects ELECTRON_RENDERER_URL in dev. In production we
	// load the bundled HTML from out/renderer.
	const devUrl = process.env["ELECTRON_RENDERER_URL"];
	if (devUrl) {
		void mainWindow.loadURL(devUrl);
	} else {
		void mainWindow.loadFile(join(__dirname, "../renderer/index.html"));
	}

	mainWindow.on("closed", () => {
		mainWindow = null;
	});
}

// ── Application menu ─────────────────────────────────────────────
function createMenu() {
	const appName = "RZEM AI Inference";
	const template: Electron.MenuItemConstructorOptions[] = [
		{
			label: appName,
			submenu: [
				{ label: `About ${appName}`, click: () => app.showAboutPanel() },
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

	// Create engine client (reads stored host/port or defaults to localhost:8100).
	// The engine itself is always remote — we never spawn a local Python sidecar.
	const storedHost = (db.prepare("SELECT value FROM settings WHERE key = 'ENGINE_HOST'").get() as { value: string } | null)?.value ?? "127.0.0.1";
	const storedPort = parseInt((db.prepare("SELECT value FROM settings WHERE key = 'ENGINE_PORT'").get() as { value: string } | null)?.value ?? "8100", 10);
	engineClient = new EngineClient({ host: storedHost, port: storedPort });
	discoveryService = createDiscoveryService();

	// Shared config — single object so runtime output dir changes propagate to all services
	const config = { outputDir, stylesDir };

	// Create FAL cloud service
	const falService = createFalService(config);

	// Register IPC handlers
	registerIpcHandlers(db, engineClient, config, () => mainWindow, falService, discoveryService);

	// Set dock icon in dev mode (packaged app uses icon from .app bundle)
	if (!app.isPackaged && process.platform === "darwin") {
		app.dock?.setIcon(join(app.getAppPath(), "resources", "icon.png"));
	}

	app.setAboutPanelOptions({
		applicationName: "RZEM AI Inference",
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

	app.on("activate", () => {
		if (BrowserWindow.getAllWindows().length === 0) createWindow();
	});

	// Backfill freedesktop thumbnails for existing images so GNOME/Files
	// shows previews without invoking its own thumbnailer (which has
	// historically marked our outputs as un-thumbnailable via the fail cache).
	void backfillOutputThumbnails(outputDir);
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
