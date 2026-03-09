import { BrowserWindow, Updater, Utils } from "electrobun/bun";
import { mkdirSync, existsSync } from "fs";
import { join } from "path";

import { initDatabase } from "./database";
import { SidecarManager } from "./sidecar";
import { defineAppRPC } from "./rpc";

// ── Data directory ───────────────────────────────────────────────
const dataDir = Utils.paths.userData;
if (!existsSync(dataDir)) {
	mkdirSync(dataDir, { recursive: true });
}

// ── Database ─────────────────────────────────────────────────────
const dbPath = join(dataDir, "inference.db");
const db = initDatabase(dbPath);
console.log(`Database: ${dbPath}`);

// ── Sidecar (Python inference engine) ────────────────────────────
const outputDir = db
	.prepare("SELECT value FROM settings WHERE key = 'OUTPUT_DIR'")
	.get() as { value: string } | null;

const sidecar = new SidecarManager({
	outputDir: outputDir?.value ?? join(dataDir, "output"),
	port: 8100,
});

// ── RPC ──────────────────────────────────────────────────────────
const appRPC = defineAppRPC(db, sidecar);

// ── Window ───────────────────────────────────────────────────────
const VITE_DEV_PORT = 1978;
const VITE_DEV_URL = `http://localhost:${VITE_DEV_PORT}`;

async function getMainViewUrl(): Promise<string> {
	const channel = await Updater.localInfo.channel();
	if (channel === "dev") {
		try {
			await fetch(VITE_DEV_URL, { method: "HEAD" });
			console.log(`HMR enabled: Using Vite dev server at ${VITE_DEV_URL}`);
			return VITE_DEV_URL;
		} catch {
			console.log("Vite dev server not running — using built assets.");
		}
	}
	return "views://mainview/index.html";
}

const url = await getMainViewUrl();

const mainWindow = new BrowserWindow({
	title: "RZEM AI Inference",
	url,
	rpc: appRPC,
	frame: {
		width: 1400,
		height: 900,
		x: 100,
		y: 100,
	},
});

// ── Lifecycle ────────────────────────────────────────────────────

// Start sidecar after window is created
sidecar.start().catch((err) => {
	console.error("Failed to start sidecar:", err);
});

// Clean shutdown
mainWindow.on("close", () => {
	sidecar.stop();
	db.close();
});

console.log("RZEM AI Inference started!");
