import { BrowserWindow, Updater, Utils } from "electrobun/bun";
import { mkdirSync, existsSync } from "fs";
import { join, resolve, dirname } from "path";

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

// ── Paths ────────────────────────────────────────────────────────
const customOutputDir = db
	.prepare("SELECT value FROM settings WHERE key = 'OUTPUT_DIR'")
	.get() as { value: string } | null;
const outputDir = customOutputDir?.value ?? join(dataDir, "output");
const stylesDir = join(dataDir, "styles");

// Ensure output and styles directories exist
for (const dir of [outputDir, stylesDir]) {
	if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
}

// ── Sidecar (Python inference engine) ────────────────────────────
// Resolve engine repo: check DB setting, env var, or sibling directory
function resolveEngineDir(): string | undefined {
	const fromDb = db.prepare("SELECT value FROM settings WHERE key = 'ENGINE_DIR'").get() as { value: string } | null;
	if (fromDb?.value && existsSync(fromDb.value)) return fromDb.value;
	if (process.env.ENGINE_DIR && existsSync(process.env.ENGINE_DIR)) return process.env.ENGINE_DIR;
	// Dev default: sibling repo next to project root
	// Electrobun runs from build/ dir, so detect project root from the build path
	const candidates: string[] = [];
	const buildIdx = import.meta.dir.indexOf("/build/");
	if (buildIdx >= 0) {
		const projectRoot = import.meta.dir.substring(0, buildIdx);
		candidates.push(resolve(projectRoot, "..", "rzem-ai-inference-engine"));
	}
	candidates.push(
		resolve(import.meta.dir, "..", "..", "..", "rzem-ai-inference-engine"),
		resolve(process.cwd(), "..", "rzem-ai-inference-engine"),
	);
	for (const candidate of candidates) {
		if (existsSync(join(candidate, "pyproject.toml"))) {
			console.log(`Engine repo found at: ${candidate}`);
			return candidate;
		}
	}
	console.warn("Engine repo not found. Set ENGINE_DIR env var or DB setting.");
	return undefined;
}

const sidecar = new SidecarManager({
	outputDir,
	port: 8100,
	engineDir: resolveEngineDir(),
});

// ── RPC ──────────────────────────────────────────────────────────
const appRPC = defineAppRPC(db, sidecar, { outputDir, stylesDir });

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
	renderer: "cef",  // Specify CEF renderer
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
	process.exit(0);
});

console.log("RZEM AI Inference started!");
