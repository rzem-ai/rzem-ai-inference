/**
 * Electron main process entry point.
 * Replaces src/bun/index.ts (Electrobun).
 */

import { app, BrowserWindow, Menu, ipcMain } from "electron";
import { mkdirSync, existsSync } from "fs";
import { join, resolve } from "path";
import path from "path";

import { initDatabase } from "./database";
import { SidecarManager } from "./sidecar";
import { registerIpcHandlers } from "./ipc";
import { createFalService } from "./services/fal";

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

// ── Sidecar (Python inference engine) ────────────────────────────
function resolveEngineDir(): string | undefined {
	const fromDb = db
		.prepare("SELECT value FROM settings WHERE key = 'ENGINE_DIR'")
		.get() as { value: string } | null;
	if (fromDb?.value && existsSync(fromDb.value)) return fromDb.value;
	if (process.env.ENGINE_DIR && existsSync(process.env.ENGINE_DIR))
		return process.env.ENGINE_DIR;

	// Dev default: sibling repo next to project root
	const candidates: string[] = [
		resolve(app.getAppPath(), "..", "rzem-ai-inference-engine"),
		resolve(process.cwd(), "..", "rzem-ai-inference-engine"),
	];

	for (const candidate of candidates) {
		if (existsSync(join(candidate, "pyproject.toml"))) {
			console.log(`Engine repo found at: ${candidate}`);
			return candidate;
		}
	}
	console.warn(
		"Engine repo not found. Set ENGINE_DIR env var or DB setting.",
	);
	return undefined;
}

let sidecar: SidecarManager;
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
	const template: Electron.MenuItemConstructorOptions[] = [
		{
			label: "RZEM AI Inference",
			submenu: [
				{ role: "about" },
				{ type: "separator" },
				{ role: "hide" },
				{ role: "hideOthers" },
				{ role: "unhide" },
				{ type: "separator" },
				{ role: "quit" },
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

	// Resolve engine availability
	const engineDir = resolveEngineDir();
	const engineAvailable = engineDir !== undefined;

	// Create sidecar
	sidecar = new SidecarManager({
		outputDir,
		port: 8100,
		engineDir,
	});

	// Create FAL cloud service
	const falService = createFalService({ outputDir });

	// Register IPC handlers
	registerIpcHandlers(db, sidecar, { outputDir, stylesDir }, () => mainWindow, falService, engineAvailable);

	// Create window
	createMenu();
	createWindow();

	// Start sidecar after window is created (only if engine is installed)
	if (engineAvailable) {
		sidecar.start().catch((err) => {
			console.error("Failed to start sidecar:", err);
		});
	} else {
		console.log("Inference engine not installed — cloud-only mode");
	}

	app.on("activate", () => {
		if (BrowserWindow.getAllWindows().length === 0) createWindow();
	});
});

app.on("window-all-closed", () => {
	sidecar?.stop();
	db?.close();
	if (process.platform !== "darwin") {
		app.quit();
	}
});

app.on("before-quit", () => {
	sidecar?.stop();
	db?.close();
});

console.log("RZEM AI Inference starting...");
