/**
 * File browsing service: native dialogs and file operations.
 * Ported from pywebview dialog calls in backend/api/*.py.
 *
 * Uses Electrobun's openFileDialog() for native file/folder selection.
 */

import type Database from "bun:sqlite";
import { Utils } from "electrobun/bun";
import { existsSync, mkdirSync, copyFileSync, readFileSync, writeFileSync, readdirSync, unlinkSync, statSync } from "fs";
import { join, basename, extname, dirname } from "path";

type Row = Record<string, unknown>;

interface ApiResponse {
	status: "success" | "error";
	message?: string;
	[key: string]: unknown;
}

const IMAGE_FILTER = "*.png *.jpg *.jpeg *.webp *.bmp *.tiff";
const LORA_FILTER = "*.safetensors *.ckpt *.pt";
const METADATA_FILTER = "*.metadata.json";

export function createFilesHandlers(
	db: Database,
	config: { outputDir: string; stylesDir: string },
) {
	return {
		// ── Output directory ─────────────────────────────────────

		browseOutputDirectory: async (): Promise<ApiResponse> => {
			try {
				const result = await Utils.openFileDialog({
					startingFolder: config.outputDir,
					canChooseFiles: false,
					canChooseDirectory: true,
					allowsMultipleSelection: false,
				});
				if (!result || result.length === 0) {
					return { status: "success", changed: false };
				}
				const chosen = result[0]!;
				mkdirSync(chosen, { recursive: true });
				db.prepare(
					"INSERT INTO settings (key, value) VALUES ('OUTPUT_DIR', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
				).run(chosen);
				config.outputDir = chosen;
				return { status: "success", changed: true, outputDir: chosen };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── LoRA file selection ──────────────────────────────────

		browseLoraFiles: async (): Promise<ApiResponse> => {
			try {
				const result = await Utils.openFileDialog({
					startingFolder: "~/",
					allowedFileTypes: LORA_FILTER,
					canChooseFiles: true,
					canChooseDirectory: false,
					allowsMultipleSelection: true,
				});
				if (!result || result.length === 0) {
					return { status: "success", loras: [] };
				}

				const loras: Row[] = [];
				for (const filepath of result) {
					if (!existsSync(filepath)) continue;
					const name = basename(filepath, extname(filepath));
					const size = statSync(filepath).size;
					const id = crypto.randomUUID();
					db.prepare(
						"INSERT INTO loras (id, name, path, file_size) VALUES (?, ?, ?, ?)",
					).run(id, name, filepath, size);
					const row = db.prepare("SELECT * FROM loras WHERE id = ?").get(id) as Row;
					if (row) loras.push(row);
				}
				return { status: "success", loras };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── Image selection (for styles) ─────────────────────────

		browseImageFile: async (params: Record<string, unknown>): Promise<ApiResponse> => {
			try {
				const styleId = (params.styleId as string) ?? "";
				const result = await Utils.openFileDialog({
					startingFolder: "~/",
					allowedFileTypes: IMAGE_FILTER,
					canChooseFiles: true,
					canChooseDirectory: false,
					allowsMultipleSelection: false,
				});
				if (!result || result.length === 0) {
					return { status: "success", path: null, thumbnailPath: null };
				}
				const source = result[0]!;
				const stored = storeStyleImage(source, config.stylesDir, styleId);
				if (!stored) {
					return { status: "error", message: "Failed to process image" };
				}
				return { status: "success", path: stored.fullPath, thumbnailPath: stored.thumbPath };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── Input image (for img2img/kontext) ────────────────────

		browseInputImage: async (): Promise<ApiResponse> => {
			try {
				const result = await Utils.openFileDialog({
					startingFolder: "~/",
					allowedFileTypes: IMAGE_FILTER,
					canChooseFiles: true,
					canChooseDirectory: false,
					allowsMultipleSelection: false,
				});
				if (!result || result.length === 0) {
					return { status: "success", path: null };
				}
				return { status: "success", path: result[0] };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── Workflow image ───────────────────────────────────────

		browseWorkflowImage: async (_params: Record<string, unknown>): Promise<ApiResponse> => {
			try {
				const result = await Utils.openFileDialog({
					startingFolder: "~/",
					allowedFileTypes: IMAGE_FILTER,
					canChooseFiles: true,
					canChooseDirectory: false,
					allowsMultipleSelection: false,
				});
				if (!result || result.length === 0) {
					return { status: "success", path: null };
				}
				return { status: "success", path: result[0] };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── Save clipboard image ─────────────────────────────────

		saveClipboardImage: (params: Record<string, unknown>): ApiResponse => {
			try {
				const dataUrl = params.dataUrl as string;
				if (!dataUrl?.startsWith("data:")) {
					return { status: "error", message: "Invalid data URL" };
				}
				const match = dataUrl.match(/^data:image\/(\w+);base64,(.+)$/);
				if (!match) {
					return { status: "error", message: "Invalid image data URL format" };
				}
				const ext = match[1] === "jpeg" ? "jpg" : match[1]!;
				const b64data = match[2]!;

				// Cap at ~50MB base64 (~37MB decoded)
				if (b64data.length > 50_000_000) {
					return { status: "error", message: "Image too large (max 50MB)" };
				}

				const tmpDir = join(dirname(config.outputDir), "tmp");
				mkdirSync(tmpDir, { recursive: true });

				// Clean old temp files
				try {
					for (const old of readdirSync(tmpDir)) {
						if (old.startsWith("kontext_input_")) {
							unlinkSync(join(tmpDir, old));
						}
					}
				} catch { /* ignore */ }

				const raw = Buffer.from(b64data, "base64");
				const filename = `kontext_input_${crypto.randomUUID().slice(0, 8)}.${ext}`;
				const path = join(tmpDir, filename);
				writeFileSync(path, raw);
				return { status: "success", path };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── Save image as (single file) ──────────────────────────
		// Electrobun has no save dialog — pick a folder and save there

		saveImageAs: async (params: Record<string, unknown>): Promise<ApiResponse> => {
			try {
				const filePath = params.filePath as string;
				if (!filePath || !existsSync(filePath)) {
					return { status: "error", message: "Source file not found" };
				}
				const result = await Utils.openFileDialog({
					startingFolder: config.outputDir,
					canChooseFiles: false,
					canChooseDirectory: true,
					allowsMultipleSelection: false,
				});
				if (!result || result.length === 0) {
					return { status: "success", saved: false };
				}
				const destDir = result[0]!;
				const dest = join(destDir, basename(filePath));
				copyFileSync(filePath, dest);
				return { status: "success", saved: true, path: dest };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── Batch save images ────────────────────────────────────

		batchSaveImages: async (params: Record<string, unknown>): Promise<ApiResponse> => {
			try {
				const imageIds = params.imageIds as string[];
				if (!imageIds?.length) {
					return { status: "success", savedCount: 0 };
				}
				const result = await Utils.openFileDialog({
					startingFolder: config.outputDir,
					canChooseFiles: false,
					canChooseDirectory: true,
					allowsMultipleSelection: false,
				});
				if (!result || result.length === 0) {
					return { status: "success", savedCount: 0 };
				}
				const destDir = result[0]!;
				let savedCount = 0;
				for (const id of imageIds) {
					const img = db.prepare("SELECT file_path FROM images WHERE id = ?").get(id) as { file_path: string } | null;
					if (img?.file_path && existsSync(img.file_path)) {
						copyFileSync(img.file_path, join(destDir, basename(img.file_path)));
						savedCount++;
					}
				}
				return { status: "success", savedCount, folder: destDir };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── Import style image ───────────────────────────────────

		importStyleImage: async (params: Record<string, unknown>): Promise<ApiResponse> => {
			try {
				const source = params.source as string;
				const styleId = (params.styleId as string) ?? "";
				if (!source) {
					return { status: "error", message: "No source provided" };
				}
				const stored = storeStyleImage(source, config.stylesDir, styleId);
				if (!stored) {
					return { status: "error", message: "Failed to process image" };
				}
				return { status: "success", path: stored.fullPath, thumbnailPath: stored.thumbPath };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── Browse metadata files ────────────────────────────────

		browseMetadataFiles: async (): Promise<ApiResponse> => {
			try {
				const result = await Utils.openFileDialog({
					startingFolder: "~/",
					allowedFileTypes: METADATA_FILTER,
					canChooseFiles: true,
					canChooseDirectory: false,
					allowsMultipleSelection: true,
				});
				return { status: "success", paths: result ?? [] };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── Browse and import metadata ───────────────────────────

		browseAndImportMetadata: async (): Promise<ApiResponse> => {
			try {
				const result = await Utils.openFileDialog({
					startingFolder: "~/",
					allowedFileTypes: METADATA_FILTER,
					canChooseFiles: true,
					canChooseDirectory: false,
					allowsMultipleSelection: true,
				});
				if (!result || result.length === 0) {
					return { status: "success", styles: [], errors: [] };
				}
				// For now, return paths — CivitAI import logic is in the styles service
				return { status: "success", paths: result, styles: [], errors: [] };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		// ── CivitAI metadata import ─────────────────────────────

		importCivitaiMetadata: (params: Record<string, unknown>): ApiResponse => {
			try {
				const filePath = params.filePath as string | undefined;
				const jsonContent = params.jsonContent as string | undefined;

				let meta: Record<string, unknown>;
				if (filePath) {
					meta = JSON.parse(readFileSync(filePath, "utf-8"));
				} else if (jsonContent) {
					meta = JSON.parse(jsonContent);
				} else {
					return { status: "error", message: "No file path or JSON content provided" };
				}

				// Extract basic metadata
				const modelName = (meta.name as string) ?? "Imported Style";
				const description = ((meta.description as string) ?? "").replace(/<[^>]+>/g, "");
				const styleId = crypto.randomUUID();
				const now = Math.floor(Date.now() / 1000);

				// Create style record
				db.prepare(
					"INSERT INTO styles (id, name, description, category, created_at, updated_at) VALUES (?, ?, ?, 'imported', ?, ?)",
				).run(styleId, modelName, description, now, now);

				const style = db.prepare("SELECT * FROM styles WHERE id = ?").get(styleId) as Row;
				return { status: "success", style };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},
	};
}

// ── Image processing helper ─────────────────────────────────────

/**
 * Copy a local image to the styles directory with a unique name.
 * Creates a simple copy (no resize/thumbnail — would need sharp for that).
 */
function storeStyleImage(
	source: string,
	stylesDir: string,
	styleId?: string,
): { fullPath: string; thumbPath: string } | null {
	try {
		if (!existsSync(source)) return null;
		mkdirSync(stylesDir, { recursive: true });

		const stem = styleId
			? `${styleId}_${crypto.randomUUID().slice(0, 12)}`
			: crypto.randomUUID().slice(0, 12);
		const ext = extname(source) || ".png";
		const fullPath = join(stylesDir, `${stem}${ext}`);
		copyFileSync(source, fullPath);

		// Use the same path for thumbnail for now (sharp would be needed for proper thumbnails)
		const thumbPath = fullPath;
		return { fullPath, thumbPath };
	} catch {
		return null;
	}
}
