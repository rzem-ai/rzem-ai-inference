/**
 * File browsing service: native dialogs and file operations.
 * Migrated from src/bun/services/files.ts — electrobun → electron dialog.
 */

import type Database from "better-sqlite3";
import BetterSqlite3 from "better-sqlite3";
import { dialog, type BrowserWindow } from "electron";
import {
	existsSync, mkdirSync, copyFileSync, readFileSync, writeFileSync,
	readdirSync, unlinkSync, statSync,
} from "fs";
import { join, basename, extname, dirname } from "path";

type Row = Record<string, unknown>;

interface ApiResponse {
	status: "success" | "error";
	message?: string;
	[key: string]: unknown;
}

const IMAGE_EXTENSIONS = [
	{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tiff"] },
];
const LORA_EXTENSIONS = [
	{ name: "LoRA Models", extensions: ["safetensors", "ckpt", "pt"] },
];
const METADATA_EXTENSIONS = [
	{ name: "Styles & Images", extensions: ["json", "png", "jpg", "jpeg", "webp"] },
	{ name: "Metadata JSON", extensions: ["json"] },
	{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] },
];

export function createFilesHandlers(
	db: Database.Database,
	config: { outputDir: string; stylesDir: string },
	getWindow: () => BrowserWindow | null,
) {
	return {
		browseOutputDirectory: async (): Promise<ApiResponse> => {
			try {
				const win = getWindow();
				const result = await dialog.showOpenDialog(win!, {
					defaultPath: config.outputDir,
					properties: ["openDirectory"],
				});
				if (result.canceled || result.filePaths.length === 0) {
					return { status: "success", changed: false };
				}
				const chosen = result.filePaths[0]!;
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

		browseLoraFiles: async (): Promise<ApiResponse> => {
			try {
				const win = getWindow();
				const result = await dialog.showOpenDialog(win!, {
					filters: LORA_EXTENSIONS,
					properties: ["openFile", "multiSelections"],
				});
				if (result.canceled || result.filePaths.length === 0) {
					return { status: "success", loras: [] };
				}

				const loras: Row[] = [];
				for (const filepath of result.filePaths) {
					if (!existsSync(filepath)) continue;
					const name = basename(filepath, extname(filepath));
					const size = statSync(filepath).size;
					const id = crypto.randomUUID();
					db.prepare("INSERT INTO loras (id, name, path, size_bytes, created_at) VALUES (?, ?, ?, ?, ?)").run(
						id, name, filepath, size, Math.floor(Date.now() / 1000),
					);
					const row = db.prepare("SELECT * FROM loras WHERE id = ?").get(id) as Row;
					if (row) loras.push(row);
				}
				return { status: "success", loras };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		browseImageFile: async (params: Record<string, unknown>): Promise<ApiResponse> => {
			try {
				const styleId = (params.styleId as string) ?? "";
				const win = getWindow();
				const result = await dialog.showOpenDialog(win!, {
					filters: IMAGE_EXTENSIONS,
					properties: ["openFile"],
				});
				if (result.canceled || result.filePaths.length === 0) {
					return { status: "success", path: null, thumbnailPath: null };
				}
				const source = result.filePaths[0]!;
				const stored = await storeStyleImage(source, config.stylesDir, styleId);
				if (!stored) {
					return { status: "error", message: "Failed to process image" };
				}
				return { status: "success", path: stored.fullPath, thumbnailPath: stored.thumbPath };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		browseInputImage: async (): Promise<ApiResponse> => {
			try {
				const win = getWindow();
				const result = await dialog.showOpenDialog(win!, {
					filters: IMAGE_EXTENSIONS,
					properties: ["openFile"],
				});
				if (result.canceled || result.filePaths.length === 0) {
					return { status: "success", path: null };
				}
				return { status: "success", path: result.filePaths[0] };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		browseWorkflowImage: async (_params: Record<string, unknown>): Promise<ApiResponse> => {
			try {
				const win = getWindow();
				const result = await dialog.showOpenDialog(win!, {
					filters: IMAGE_EXTENSIONS,
					properties: ["openFile"],
				});
				if (result.canceled || result.filePaths.length === 0) {
					return { status: "success", path: null };
				}
				return { status: "success", path: result.filePaths[0] };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

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

				if (b64data.length > 50_000_000) {
					return { status: "error", message: "Image too large (max 50MB)" };
				}

				const tmpDir = join(dirname(config.outputDir), "tmp");
				mkdirSync(tmpDir, { recursive: true });

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

		saveImageAs: async (params: Record<string, unknown>): Promise<ApiResponse> => {
			try {
				const filePath = params.filePath as string;
				if (!filePath || !existsSync(filePath)) {
					return { status: "error", message: "Source file not found" };
				}
				const win = getWindow();
				const result = await dialog.showSaveDialog(win!, {
					defaultPath: join(config.outputDir, basename(filePath)),
					filters: IMAGE_EXTENSIONS,
				});
				if (result.canceled || !result.filePath) {
					return { status: "success", saved: false };
				}
				copyFileSync(filePath, result.filePath);
				return { status: "success", saved: true, path: result.filePath };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		batchSaveImages: async (params: Record<string, unknown>): Promise<ApiResponse> => {
			try {
				const imageIds = params.imageIds as string[];
				if (!imageIds?.length) {
					return { status: "success", savedCount: 0 };
				}
				const win = getWindow();
				const result = await dialog.showOpenDialog(win!, {
					defaultPath: config.outputDir,
					properties: ["openDirectory"],
				});
				if (result.canceled || result.filePaths.length === 0) {
					return { status: "success", savedCount: 0 };
				}
				const destDir = result.filePaths[0]!;
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

		importStyleImage: async (params: Record<string, unknown>): Promise<ApiResponse> => {
			try {
				const source = params.source as string;
				const styleId = (params.styleId as string) ?? "";
				if (!source) {
					return { status: "error", message: "No source provided" };
				}
				const stored = await storeStyleImage(source, config.stylesDir, styleId);
				if (!stored) {
					return { status: "error", message: "Failed to process image" };
				}
				return { status: "success", path: stored.fullPath, thumbnailPath: stored.thumbPath };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		browseMetadataFiles: async (): Promise<ApiResponse> => {
			try {
				const win = getWindow();
				const result = await dialog.showOpenDialog(win!, {
					filters: METADATA_EXTENSIONS,
					properties: ["openFile", "multiSelections"],
				});
				return { status: "success", paths: result.filePaths ?? [] };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		browseAndImportMetadata: async (): Promise<ApiResponse> => {
			try {
				const win = getWindow();
				const result = await dialog.showOpenDialog(win!, {
					filters: METADATA_EXTENSIONS,
					properties: ["openFile", "multiSelections"],
				});
				if (result.canceled || result.filePaths.length === 0) {
					return { status: "success", paths: [], styles: [], errors: [] };
				}
				return { status: "success", paths: result.filePaths, styles: [], errors: [] };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		browseInvokeaiDb: async (): Promise<ApiResponse> => {
			try {
				const win = getWindow();
				const result = await dialog.showOpenDialog(win!, {
					filters: [{ name: "InvokeAI Database", extensions: ["db", "sqlite", "sqlite3"] }],
					properties: ["openFile"],
				});
				if (result.canceled || result.filePaths.length === 0) {
					return { status: "success", path: null };
				}
				return { status: "success", path: result.filePaths[0] };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		importInvokeaiLoras: async (params: Record<string, unknown>): Promise<ApiResponse> => {
			const dbPath = params.dbPath as string | undefined;
			if (!dbPath || !existsSync(dbPath)) {
				return { status: "error", message: "InvokeAI database file not found" };
			}

			let srcDb: Database.Database;
			try {
				srcDb = new BetterSqlite3(dbPath, { readonly: true, fileMustExist: true });
			} catch (err) {
				return { status: "error", message: `Failed to open DB: ${String(err)}` };
			}

			try {
				// Detect which table holds models — InvokeAI has used both "models" and "model_config".
				const tables = srcDb
					.prepare("SELECT name FROM sqlite_master WHERE type='table'")
					.all() as Array<{ name: string }>;
				const tableNames = tables.map((t) => t.name);
				const modelTable = tableNames.includes("models")
					? "models"
					: tableNames.includes("model_config")
						? "model_config"
						: null;
				if (!modelTable) {
					return { status: "error", message: "No models table found in InvokeAI DB" };
				}

				const cols = srcDb.prepare(`PRAGMA table_info(${modelTable})`).all() as Array<{ name: string }>;
				const colNames = new Set(cols.map((c) => c.name));
				const hasConfig = colNames.has("config");

				type InvokeaiRow = {
					id?: string;
					name?: string;
					type?: string;
					base?: string;
					path?: string;
					description?: string | null;
					trigger_phrases?: string | null;
					cover_image?: string | null;
					config?: string | null;
				};

				const rows = srcDb
					.prepare(`SELECT * FROM ${modelTable}`)
					.all() as InvokeaiRow[];

				// Some InvokeAI versions encode type/base/description inside `config` JSON blob.
				const loraRows = rows
					.map((r) => {
						let merged: InvokeaiRow = { ...r };
						if (hasConfig && r.config) {
							try {
								const parsed = JSON.parse(r.config);
								merged = { ...parsed, ...r };
								if (!merged.description && parsed.description) merged.description = parsed.description;
								if (!merged.type && parsed.type) merged.type = parsed.type;
								if (!merged.trigger_phrases && parsed.trigger_phrases)
									merged.trigger_phrases = Array.isArray(parsed.trigger_phrases)
										? parsed.trigger_phrases.join(", ")
										: parsed.trigger_phrases;
								if (!merged.cover_image && parsed.cover_image) merged.cover_image = parsed.cover_image;
							} catch { /* ignore */ }
						}
						return merged;
					})
					.filter((r) => String(r.type ?? "").toLowerCase() === "lora")
					.filter((r) => typeof r.description === "string" && r.description.trim().length > 0);

				// Candidate roots for model cover images: <db>/../model_images, <db>/../../model_images.
				const dbDir = dirname(dbPath);
				const candidateImageDirs = [
					join(dbDir, "model_images"),
					join(dirname(dbDir), "model_images"),
				].filter((d) => existsSync(d));

				const coverExts = [".png", ".webp", ".jpg", ".jpeg"];
				const findCoverImage = (row: InvokeaiRow): string | null => {
					const cover = row.cover_image;
					if (cover && typeof cover === "string") {
						if (existsSync(cover)) return cover;
						// Might be a relative filename
						for (const d of candidateImageDirs) {
							const p = join(d, cover);
							if (existsSync(p)) return p;
						}
					}
					const id = row.id;
					if (id) {
						for (const d of candidateImageDirs) {
							for (const ext of coverExts) {
								const p = join(d, `${id}${ext}`);
								if (existsSync(p)) return p;
							}
						}
					}
					return null;
				};

				const now = Math.floor(Date.now() / 1000);
				let imported = 0;
				const errors: string[] = [];

				const insertLora = db.prepare(
					"INSERT INTO loras (id, name, path, trigger_words, base_model, size_bytes, strength, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
				);
				const insertStyle = db.prepare(
					"INSERT INTO styles (id, name, description, prompt_template, category, thumbnail_path, created_at, updated_at) VALUES (?, ?, ?, '', 'imported', ?, ?, ?)",
				);
				const linkStyleLora = db.prepare(
					"INSERT INTO style_loras (style_id, lora_id, strength, priority) VALUES (?, ?, ?, 0)",
				);

				for (const row of loraRows) {
					try {
						const name = row.name ?? "InvokeAI LoRA";
						const description = (row.description ?? "").replace(/<[^>]+>/g, "").trim();
						const triggerWords = row.trigger_phrases ?? null;
						const baseModel = row.base ?? null;
						const loraPath = row.path ?? null;

						let loraId: string | null = null;
						if (loraPath) {
							const existing = db
								.prepare("SELECT id FROM loras WHERE path = ?")
								.get(loraPath) as { id: string } | undefined;
							if (existing) {
								loraId = existing.id;
							} else {
								loraId = crypto.randomUUID();
								let sizeBytes: number | null = null;
								try { if (existsSync(loraPath)) sizeBytes = statSync(loraPath).size; } catch { /* ignore */ }
								insertLora.run(loraId, name, loraPath, triggerWords, baseModel, sizeBytes, 1.0, now);
							}
						}

						const styleId = crypto.randomUUID();
						let thumbnailPath: string | null = null;
						const cover = findCoverImage(row);
						if (cover) {
							const stored = await storeStyleImage(cover, config.stylesDir, styleId);
							if (stored) thumbnailPath = stored.thumbPath;
						}

						insertStyle.run(styleId, name, description, thumbnailPath, now, now);
						if (loraId) linkStyleLora.run(styleId, loraId, 1.0);
						imported++;
					} catch (err) {
						errors.push(`${row.name ?? row.id}: ${String(err)}`);
					}
				}

				return { status: "success", imported, total: loraRows.length, errors };
			} catch (err) {
				return { status: "error", message: String(err) };
			} finally {
				try { srcDb.close(); } catch { /* ignore */ }
			}
		},

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

				const modelName = (meta.name as string) ?? "Imported Style";
				const description = ((meta.description as string) ?? "").replace(/<[^>]+>/g, "");
				const styleId = crypto.randomUUID();
				const now = Math.floor(Date.now() / 1000);

				db.prepare(
					"INSERT INTO styles (id, name, description, prompt_template, category, created_at, updated_at) VALUES (?, ?, ?, '', 'imported', ?, ?)",
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

async function storeStyleImage(
	source: string,
	stylesDir: string,
	styleId?: string,
): Promise<{ fullPath: string; thumbPath: string } | null> {
	try {
		if (!existsSync(source)) return null;
		mkdirSync(stylesDir, { recursive: true });

		const stem = styleId
			? `${styleId}_${crypto.randomUUID().slice(0, 12)}`
			: crypto.randomUUID().slice(0, 12);
		const ext = extname(source) || ".png";
		const fullPath = join(stylesDir, `${stem}${ext}`);
		copyFileSync(source, fullPath);

		// Generate thumbnail
		const thumbPath = join(stylesDir, `${stem}_thumb.webp`);
		let thumbnailPath: string;
		try {
			const sharp = (await import("sharp")).default;
			await sharp(fullPath)
				.resize({ width: 512, withoutEnlargement: true })
				.webp({ quality: 85 })
				.toFile(thumbPath);
			thumbnailPath = thumbPath;
		} catch {
			thumbnailPath = fullPath;
		}

		return { fullPath, thumbPath: thumbnailPath };
	} catch {
		return null;
	}
}
