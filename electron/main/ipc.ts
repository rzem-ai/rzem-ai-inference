/**
 * Electron IPC handler registration.
 * Replaces src/bun/rpc.ts (Electrobun BrowserView.defineRPC).
 *
 * Each RPC method becomes an ipcMain.handle channel.
 * Push messages use mainWindow.webContents.send().
 */

import { ipcMain, type BrowserWindow } from "electron";
import electronUpdater from "electron-updater";
const { autoUpdater } = electronUpdater;
import type Database from "better-sqlite3";
import type { EngineClient } from "./engine-client";
import type { DiscoveryService } from "./discovery";
import type { FalService } from "./services/fal";
import { batchParseData, batchRenderTemplate } from "./services/batch";
import { createStylesHandlers } from "./services/styles";
import { createSettingsHandlers } from "./services/settings";
import { createChatService } from "./services/chat";
import { createSkillsService } from "./services/skills";
import { createFilesHandlers } from "./services/files";
import { createWorkflowService } from "./services/workflow";
import { DEFAULT_BUNDLES, DEFAULT_BUNDLE_TYPES } from "./services/bundles";
import { statSync, renameSync, unlinkSync, existsSync, readFileSync, readdirSync } from "fs";
import { join, basename } from "path";

// ── Helpers ──────────────────────────────────────────────────────

function keysToSnake(obj: unknown): unknown {
	if (Array.isArray(obj)) return obj.map(keysToSnake);
	if (obj && typeof obj === "object" && !(obj instanceof Date)) {
		const result: Record<string, unknown> = {};
		for (const [key, value] of Object.entries(obj as Record<string, unknown>)) {
			const snakeKey = key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`);
			result[snakeKey] = keysToSnake(value);
		}
		return result;
	}
	return obj;
}

// ── Image helpers ────────────────────────────────────────────────

function formatTimestamp(): string {
	const d = new Date();
	const pad = (n: number) => String(n).padStart(2, "0");
	return `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}_${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`;
}

function deriveUpscalePath(originalPath: string, scaleFactor: number, outputDir: string): string {
	const base = basename(originalPath, ".png");
	let candidate = join(outputDir, `${base}_${scaleFactor}x.png`);
	let n = 1;
	while (existsSync(candidate)) {
		candidate = join(outputDir, `${base}_${scaleFactor}x_${n}.png`);
		n++;
	}
	return candidate;
}

async function processOutputImage(
	sourcePath: string,
	outputDir: string,
	jobId: string,
): Promise<{ imagePath: string; coverPath: string | null }> {
	const timestamp = formatTimestamp();
	const stem = `${timestamp}_${jobId}`;
	const imagePath = join(outputDir, `${stem}.png`);
	const coverPath = join(outputDir, `${stem}_cover.webp`);

	if (existsSync(sourcePath)) {
		try {
			renameSync(sourcePath, imagePath);
		} catch {
			return { imagePath: sourcePath, coverPath: null };
		}
	} else {
		const match = readdirSync(outputDir).find((f: string) => f.endsWith(`_${jobId}.png`));
		if (match) {
			const existingPath = join(outputDir, match);
			const existingCover = existingPath.replace(/\.png$/, "_cover.webp");
			return { imagePath: existingPath, coverPath: existsSync(existingCover) ? existingCover : null };
		}
		return { imagePath: sourcePath, coverPath: null };
	}

	// Generate half-res WebP thumbnail
	try {
		const sharp = (await import("sharp")).default;
		await sharp(imagePath)
			.resize({ width: 512, withoutEnlargement: true })
			.webp({ quality: 85 })
			.toFile(coverPath);
		return { imagePath, coverPath };
	} catch (err) {
		console.error(`[cover] Failed to generate cover for ${jobId}:`, err);
		return { imagePath, coverPath: null };
	}
}

// ── Shared types ─────────────────────────────────────────────────

type Row = Record<string, unknown>;
type SQLValue = string | number | null | bigint | boolean;
type Res = Record<string, unknown>;

// ── Handler factory ──────────────────────────────────────────────

export function registerIpcHandlers(
	db: Database.Database,
	engineClient: EngineClient,
	config: { outputDir: string; stylesDir: string },
	getWindow: () => BrowserWindow | null,
	falService: FalService,
	discoveryService: DiscoveryService,
) {
	const styles = createStylesHandlers(db, config.stylesDir);
	const settings = createSettingsHandlers(db, engineClient, config.outputDir);
	const skills = createSkillsService();
	const chat = createChatService(db, skills);
	const files = createFilesHandlers(db, config, getWindow);
	const workflow = createWorkflowService({ db, engineClient, outputDir: config.outputDir, chatService: chat, falService });

	// Initialize HF_TOKEN from database if set
	const hfKey = db.prepare("SELECT value FROM settings WHERE key = 'HF_API_KEY'").get() as { value: string } | null;
	if (hfKey?.value) {
		process.env.HF_TOKEN = hfKey.value;
	}

	// Event buffer for polling compatibility
	const inferenceEventBuffer: Res[] = [];

	// Job metadata tracking
	const jobMeta = new Map<string, {
		params: Record<string, unknown>;
		bundleId?: string;
		styleId?: string;
		rawPrompt?: string;
		startTime: number;
	}>();

	// Helper to send push messages to the renderer
	function sendToRenderer(channel: string, ...args: unknown[]) {
		try {
			const win = getWindow();
			if (win && !win.isDestroyed()) {
				win.webContents.send(channel, ...args);
			}
		} catch {
			// Window may not be ready yet
		}
	}

	// Wire chat events → push messages to renderer
	chat.onEvent((event) => {
		sendToRenderer("chatEvent", {
			event: event.type,
			data: event.data,
		});
	});

	// Shared handler for job completion — persists image to DB
	async function handleJobCompleted(jobId: string, data: Res) {
		const meta = jobMeta.get(jobId);
		const sourcePath = join(config.outputDir, `${jobId}.png`);
		const { imagePath, coverPath } = await processOutputImage(sourcePath, config.outputDir, jobId);
		try {
			const p = meta?.params ?? {};
			const imageId = crypto.randomUUID();
			const now = Math.floor(Date.now() / 1000);
			const genTimeMs = meta ? Math.round(performance.now() - meta.startTime) : null;
			let fileSize: number | null = null;
			try { fileSize = statSync(imagePath).size; } catch { /* ignore */ }
			const negativePrompt = (p.negative_prompt as string) ?? (p.negativePrompt as string) ?? null;
			const modelConfig = JSON.stringify({
				transformer_model: p.transformer_model ?? p.transformerModel,
				transformer_type: p.transformer_type ?? p.transformerType,
				vae_model: p.vae_model ?? p.vaeModel,
				clip_tokenizer: p.clip_tokenizer ?? p.clipTokenizer,
				clip_encoder: p.clip_encoder ?? p.clipEncoder,
				t5_tokenizer: p.t5_tokenizer ?? p.t5Tokenizer,
				t5_encoder: p.t5_encoder ?? p.t5Encoder,
				sampler: p.sampler, scheduler: p.scheduler,
				input_image_path: p.input_image_path ?? p.inputImagePath,
				fal_endpoint: p.fal_endpoint ?? p.falEndpoint,
				bundle_id: meta?.bundleId, style_id: meta?.styleId,
			});
			db.prepare(
				`INSERT INTO images (id, file_path, thumbnail_path, prompt, raw_prompt, negative_prompt, width, height, file_size, steps, cfg_scale, seed, bundle_id, model_config, loras, generation_time_ms, created_at, updated_at)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
			).run(
				imageId, imagePath, coverPath,
				(p.prompt as string) ?? "",
				meta?.rawPrompt ?? null,
				negativePrompt,
				(p.width as number) ?? 1024,
				(p.height as number) ?? 1024,
				fileSize,
				(p.steps as number) ?? 20,
				(p.cfg_scale as number) ?? (p.cfgScale as number) ?? 1.0,
				(data.seed as number) ?? -1,
				meta?.bundleId ?? null,
				modelConfig,
				p.loras ? JSON.stringify(p.loras) : null,
				genTimeMs,
				now, now,
			);
		} catch (err) {
			console.error("Failed to persist image:", err);
		}
		// Clean up preview images
		try {
			const previews = readdirSync(config.outputDir).filter((f: string) => f.startsWith(`${jobId}_preview_`));
			for (const preview of previews) {
				try { unlinkSync(join(config.outputDir, preview)); } catch { /* ignore */ }
			}
		} catch { /* ignore */ }
		jobMeta.delete(jobId);
		return imagePath;
	}

	// Shared handler to emit events to buffer + renderer
	function emitInferenceEvent(eventType: string, jobId: string, data: Res) {
		const mapped: Res = { type: eventType, data };
		inferenceEventBuffer.push(mapped);
		if (inferenceEventBuffer.length > 500) {
			inferenceEventBuffer.splice(0, inferenceEventBuffer.length - 500);
		}
		sendToRenderer("inferenceEvent", { event: eventType, jobId, data });
	}

	// Wire engine events → both push messages and polling buffer
	engineClient.onEvent(async (event) => {
		const eventType = (event.event as string) ?? "unknown";
		const jobId = (event.job_id as string) ?? "";
		const data = (event.data as Res) ?? {};

		let mappedData = data;
		if (eventType === "job_completed" && jobId) {
			const resolvedImagePath = await handleJobCompleted(jobId, data);
			mappedData = { ...data, image_path: resolvedImagePath };
		} else if (eventType === "job_failed" || eventType === "job_cancelled") {
			jobMeta.delete(jobId);
		}

		emitInferenceEvent(eventType, jobId, mappedData);
	});

	// Wire FAL service events (cloud inference path).
	falService.onEvent(async (event) => {
		const eventType = (event.event as string) ?? "unknown";
		const jobId = (event.job_id as string) ?? "";
		const data = (event.data as Res) ?? {};

		let mappedData = data;
		if (eventType === "job_completed" && jobId) {
			const resolvedImagePath = await handleJobCompleted(jobId, data);
			mappedData = { ...data, image_path: resolvedImagePath };
		} else if (eventType === "job_failed" || eventType === "job_cancelled") {
			jobMeta.delete(jobId);
		}

		emitInferenceEvent(eventType, jobId, mappedData);
	});

	// ── Register all IPC handlers ────────────────────────────────

	// System
	ipcMain.handle("healthCheck", () => ({ status: "ok" }));

	// Settings (key-value)
	ipcMain.handle("getSetting", (_e, { key }: { key: string }) => {
		const row = db.prepare("SELECT value FROM settings WHERE key = ?").get(key) as { value: string } | null;
		return { status: "success", value: row?.value ?? null };
	});
	ipcMain.handle("setSetting", (_e, { key, value }: { key: string; value: string }) => {
		db.prepare("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value").run(key, value);
		if (key === "HF_API_KEY" && value) {
			process.env.HF_TOKEN = value;
		}
		return { status: "success" };
	});

	// Bundle Types
	ipcMain.handle("getBundleTypes", () => {
		const rows = db.prepare("SELECT * FROM bundle_types ORDER BY sort_order").all() as Row[];
		return { status: "success", bundleTypes: rows };
	});

	// Bundles
	function parseBundle(row: Row): Row {
		for (const k of ["fal_aspectratio", "loras"] as const) {
			if (typeof row[k] === "string") {
				try { row[k] = JSON.parse(row[k] as string); } catch { /* leave as-is */ }
			}
		}
		return row;
	}

	ipcMain.handle("getBundles", (_e, { includeHidden } = {}) => {
		const q = includeHidden ? "SELECT * FROM bundles ORDER BY transformer_type, tier" : "SELECT * FROM bundles WHERE hidden = 0 ORDER BY transformer_type, tier";
		return { status: "success", bundles: (db.prepare(q).all() as Row[]).map(parseBundle) };
	});
	ipcMain.handle("getBundle", (_e, { bundleId }: { bundleId: string }) => {
		const row = db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId) as Row | null;
		if (!row) return { status: "error", message: "Bundle not found" };
		return { status: "success", bundle: parseBundle(row) };
	});
	ipcMain.handle("getBundlesForType", (_e, { transformerType }: { transformerType: string }) => {
		return { status: "success", bundles: (db.prepare("SELECT * FROM bundles WHERE transformer_type = ? AND hidden = 0 ORDER BY tier").all(transformerType) as Row[]).map(parseBundle) };
	});
	ipcMain.handle("createBundle", (_e, params) => {
		const p = params as Record<string, any>;
		db.prepare(`INSERT INTO bundles (id, label, description, transformer_type, tier, transformer_model, vae_model, clip_tokenizer, clip_encoder, t5_tokenizer, t5_encoder, t5_encoder_config, qwen3_tokenizer, qwen3_encoder, steps, cfg_scale, sampler, scheduler, vram_estimate_gb, is_default, source, fal_endpoint, fal_aspectratio) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`).run(
			p.id, p.label, p.description ?? "", p.transformerType, p.tier ?? "balanced", p.transformerModel, p.vaeModel,
			p.clipTokenizer ?? null, p.clipEncoder ?? null, p.t5Tokenizer ?? null, p.t5Encoder ?? null, p.t5EncoderConfig ?? null,
			p.qwen3Tokenizer ?? null, p.qwen3Encoder ?? null, p.steps ?? 20, p.cfgScale ?? 1.0, p.sampler ?? "euler",
			p.scheduler ?? "normal", p.vramEstimateGb ?? 0.0, p.isDefault ?? 1, p.source ?? "local", p.falEndpoint ?? null, p.falAspectratio ?? null,
		);
		return { status: "success", bundle: db.prepare("SELECT * FROM bundles WHERE id = ?").get(p.id) as Row };
	});
	ipcMain.handle("updateBundle", (_e, params) => {
		const { bundleId, ...fields } = params as Record<string, any>;
		const sets: string[] = [];
		const vals: SQLValue[] = [];
		const map: Record<string, string> = { label: "label", description: "description", tier: "tier", transformerModel: "transformer_model", vaeModel: "vae_model", steps: "steps", cfgScale: "cfg_scale", sampler: "sampler", scheduler: "scheduler", vramEstimateGb: "vram_estimate_gb", clipTokenizer: "clip_tokenizer", clipEncoder: "clip_encoder", t5Tokenizer: "t5_tokenizer", t5Encoder: "t5_encoder", t5EncoderConfig: "t5_encoder_config", qwen3Tokenizer: "qwen3_tokenizer", qwen3Encoder: "qwen3_encoder", source: "source", falEndpoint: "fal_endpoint", falAspectratio: "fal_aspectratio" };
		for (const [k, col] of Object.entries(map)) { if (k in fields) { sets.push(`${col} = ?`); vals.push(fields[k] as SQLValue); } }
		if (!sets.length) return { status: "error", message: "No fields to update" };
		vals.push(bundleId as string);
		db.prepare(`UPDATE bundles SET ${sets.join(", ")} WHERE id = ?`).run(...vals);
		return { status: "success", bundle: db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId) as Row };
	});
	ipcMain.handle("deleteBundle", (_e, { bundleId }) => { db.prepare("DELETE FROM bundles WHERE id = ?").run(bundleId); return { status: "success" }; });
	ipcMain.handle("toggleBundleFavorite", (_e, { bundleId }) => {
		db.prepare("UPDATE bundles SET favorite = CASE WHEN favorite = 0 THEN 1 ELSE 0 END WHERE id = ?").run(bundleId);
		return { status: "success", bundle: db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId) as Row };
	});
	ipcMain.handle("toggleBundleHidden", (_e, { bundleId }) => {
		db.prepare("UPDATE bundles SET hidden = CASE WHEN hidden = 0 THEN 1 ELSE 0 END WHERE id = ?").run(bundleId);
		return { status: "success", bundle: db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId) as Row };
	});
	ipcMain.handle("resetDefaultBundles", () => {
		db.prepare("DELETE FROM bundles WHERE is_default = 1").run();
		const insertBundle = db.prepare(`INSERT OR REPLACE INTO bundles (id, label, description, transformer_type, tier, transformer_model, vae_model, clip_tokenizer, clip_encoder, t5_tokenizer, t5_encoder, t5_encoder_config, qwen3_tokenizer, qwen3_encoder, steps, cfg_scale, sampler, scheduler, vram_estimate_gb, is_default, source, fal_endpoint, fal_aspectratio) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`);
		for (const b of DEFAULT_BUNDLES) {
			insertBundle.run(b.id, b.label, b.description, b.transformer_type, b.tier, b.transformer_model, b.vae_model, b.clip_tokenizer, b.clip_encoder, b.t5_tokenizer, b.t5_encoder, b.t5_encoder_config, b.qwen3_tokenizer, b.qwen3_encoder, b.steps, b.cfg_scale, b.sampler, b.scheduler, b.vram_estimate_gb, b.is_default, b.source, b.fal_endpoint, b.fal_aspectratio);
		}
		const insertType = db.prepare("INSERT OR REPLACE INTO bundle_types (id, label, icon, sort_order, guide) VALUES (?, ?, ?, ?, ?)");
		for (const t of DEFAULT_BUNDLE_TYPES) {
			insertType.run(t.id, t.label, t.icon, t.sort_order, t.guide);
		}
		return { status: "success", bundles: db.prepare("SELECT * FROM bundles WHERE is_default = 1").all() as Row[] };
	});

	// Gallery
	ipcMain.handle("getGalleryImages", (_e, params = {}) => {
		const p = params as Record<string, any>;
		const limit = p.limit ?? 50;
		const offset = p.offset ?? 0;
		const sortBy = p.sortBy ?? "created_at";
		const sortOrder = p.sortOrder === "asc" ? "ASC" : "DESC";
		let where = "WHERE 1=1";
		const bind: SQLValue[] = [];
		if (p.favoritesOnly) where += " AND i.favorite = 1";
		if (p.search) { where += " AND i.prompt LIKE ?"; bind.push(`%${p.search}%`); }
		if (p.folderId) { where += " AND i.id IN (SELECT image_id FROM image_folders WHERE folder_id = ?)"; bind.push(p.folderId); }
		if (p.tagId) { where += " AND i.id IN (SELECT image_id FROM image_tags WHERE tag_id = ?)"; bind.push(p.tagId); }
		const cnt = db.prepare(`SELECT COUNT(*) as total FROM images i ${where}`).get(...bind) as { total: number };
		const safe = ["created_at", "seed", "steps", "width", "height"].includes(sortBy) ? sortBy : "created_at";
		const images = db.prepare(`SELECT * FROM images i ${where} ORDER BY ${safe} ${sortOrder} LIMIT ? OFFSET ?`).all(...bind, limit, offset) as Row[];
		return { status: "success", images, total: cnt.total };
	});
	ipcMain.handle("getImage", (_e, { imageId }) => {
		const img = db.prepare("SELECT * FROM images WHERE id = ?").get(imageId) as Row | null;
		if (!img) return { status: "error", message: "Image not found" };
		return { status: "success", image: img };
	});
	ipcMain.handle("toggleFavorite", (_e, { imageId }) => {
		db.prepare("UPDATE images SET favorite = CASE WHEN favorite = 0 THEN 1 ELSE 0 END WHERE id = ?").run(imageId);
		return { status: "success", image: db.prepare("SELECT * FROM images WHERE id = ?").get(imageId) as Row };
	});
	ipcMain.handle("deleteImage", (_e, { imageId }) => {
		const img = db.prepare("SELECT file_path, thumbnail_path FROM images WHERE id = ?").get(imageId) as { file_path?: string; thumbnail_path?: string } | null;
		if (img) {
			if (img.file_path && existsSync(img.file_path)) try { unlinkSync(img.file_path); } catch { /* ignore */ }
			if (img.thumbnail_path && existsSync(img.thumbnail_path)) try { unlinkSync(img.thumbnail_path); } catch { /* ignore */ }
		}
		db.prepare("DELETE FROM images WHERE id = ?").run(imageId);
		return { status: "success" };
	});
	ipcMain.handle("upscaleImage", async (_e, { imageId, scaleFactor }: { imageId: string; scaleFactor: 2 | 3 | 4 }) => {
		if (![2, 3, 4].includes(scaleFactor)) return { status: "error", message: "Invalid scale factor" };

		const original = db.prepare("SELECT * FROM images WHERE id = ?").get(imageId) as Row | null;
		if (!original) return { status: "error", message: "Image not found" };
		if (!existsSync(original.file_path as string)) return { status: "error", message: "Source file not found" };

		const newImagePath = deriveUpscalePath(original.file_path as string, scaleFactor, config.outputDir);
		const newCoverPath = newImagePath.replace(/\.png$/, "_cover.webp");

		const sharpMod = await import("sharp");
		const sharp = sharpMod.default;

		await sharp(original.file_path as string)
			.resize({
				width: (original.width as number) * scaleFactor,
				height: (original.height as number) * scaleFactor,
				kernel: "lanczos3",
				fit: "fill",
			})
			.png()
			.toFile(newImagePath);

		// Best-effort thumbnail
		sharp(newImagePath)
			.resize({ width: 512, withoutEnlargement: true })
			.webp({ quality: 85 })
			.toFile(newCoverPath)
			.catch(() => {});

		const fileSize = statSync(newImagePath).size;
		const newId = crypto.randomUUID();
		const now = Math.floor(Date.now() / 1000);

		db.prepare(`INSERT INTO images
			(id, file_path, thumbnail_path, prompt, raw_prompt, negative_prompt,
			 width, height, file_size, steps, cfg_scale, seed, bundle_id, model_config, loras,
			 generation_time_ms, created_at, updated_at)
			VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)`)
			.run(newId, newImagePath, newCoverPath,
				original.prompt, original.raw_prompt, original.negative_prompt,
				(original.width as number) * scaleFactor,
				(original.height as number) * scaleFactor,
				fileSize, original.steps, original.cfg_scale, original.seed,
				original.bundle_id, original.model_config, original.loras,
				null, now, now);

		const row = db.prepare("SELECT * FROM images WHERE id = ?").get(newId);
		return { status: "success", image: row };
	});
	ipcMain.handle("getImageBase64", (_e, { imagePath }) => {
		try {
			if (!imagePath || !existsSync(imagePath)) {
				return { status: "error", message: `File not found: ${imagePath}` };
			}
			const ext = imagePath.split(".").pop()?.toLowerCase() ?? "png";
			const mime = ext === "jpg" || ext === "jpeg" ? "image/jpeg" : ext === "webp" ? "image/webp" : "image/png";
			const bytes = readFileSync(imagePath);
			const b64 = Buffer.from(bytes).toString("base64");
			return { status: "success", dataUrl: `data:${mime};base64,${b64}` };
		} catch (err) {
			return { status: "error", message: String(err) };
		}
	});

	// Folders
	ipcMain.handle("getFolders", () => ({ status: "success", folders: db.prepare("SELECT * FROM folders ORDER BY sort_order, name").all() as Row[] }));
	ipcMain.handle("createFolder", (_e, params) => {
		const p = params as Record<string, any>;
		const now = Math.floor(Date.now() / 1000);
		db.prepare("INSERT INTO folders (id, name, parent_id, color, icon, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)").run(p.id, p.name, p.parentId ?? null, p.color ?? null, p.icon ?? null, p.sortOrder ?? 0, now, now);
		return { status: "success", folder: db.prepare("SELECT * FROM folders WHERE id = ?").get(p.id) as Row };
	});
	ipcMain.handle("updateFolder", (_e, params) => {
		const { folderId, ...fields } = params as Record<string, any>;
		const sets: string[] = []; const vals: SQLValue[] = [];
		const map: Record<string, string> = { name: "name", parentId: "parent_id", color: "color", icon: "icon", sortOrder: "sort_order" };
		for (const [k, col] of Object.entries(map)) { if (k in fields) { sets.push(`${col} = ?`); vals.push(fields[k] as SQLValue); } }
		if (sets.length) { sets.push("updated_at = ?"); vals.push(Math.floor(Date.now() / 1000)); vals.push(folderId as string); db.prepare(`UPDATE folders SET ${sets.join(", ")} WHERE id = ?`).run(...vals); }
		return { status: "success", folder: db.prepare("SELECT * FROM folders WHERE id = ?").get(folderId) as Row };
	});
	ipcMain.handle("deleteFolder", (_e, { folderId }) => { db.prepare("DELETE FROM folders WHERE id = ?").run(folderId); return { status: "success" }; });
	ipcMain.handle("addImageToFolder", (_e, { imageId, folderId }) => { db.prepare("INSERT OR IGNORE INTO image_folders (image_id, folder_id, added_at) VALUES (?, ?, ?)").run(imageId, folderId, Math.floor(Date.now() / 1000)); return { status: "success" }; });
	ipcMain.handle("removeImageFromFolder", (_e, { imageId, folderId }) => { db.prepare("DELETE FROM image_folders WHERE image_id = ? AND folder_id = ?").run(imageId, folderId); return { status: "success" }; });

	// Tags
	ipcMain.handle("getTags", () => ({ status: "success", tags: db.prepare("SELECT * FROM tags ORDER BY name").all() as Row[] }));
	ipcMain.handle("createTag", (_e, { name, color, category }) => {
		db.prepare("INSERT INTO tags (name, color, category) VALUES (?, ?, ?)").run(name, color ?? null, category ?? null);
		return { status: "success", tag: db.prepare("SELECT * FROM tags WHERE name = ?").get(name) as Row };
	});
	ipcMain.handle("updateTag", (_e, params) => {
		const p = params as Record<string, any>;
		const sets: string[] = []; const vals: SQLValue[] = [];
		if (p.name !== undefined) { sets.push("name = ?"); vals.push(p.name); }
		if (p.color !== undefined) { sets.push("color = ?"); vals.push(p.color ?? null); }
		if (p.category !== undefined) { sets.push("category = ?"); vals.push(p.category ?? null); }
		if (sets.length) { vals.push(p.tagId); db.prepare(`UPDATE tags SET ${sets.join(", ")} WHERE id = ?`).run(...vals); }
		return { status: "success", tag: db.prepare("SELECT * FROM tags WHERE id = ?").get(p.tagId) as Row };
	});
	ipcMain.handle("deleteTag", (_e, { tagId }) => { db.prepare("DELETE FROM tags WHERE id = ?").run(tagId); return { status: "success" }; });
	ipcMain.handle("addTagToImage", (_e, { imageId, tagId }) => { db.prepare("INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?, ?)").run(imageId, tagId); return { status: "success" }; });
	ipcMain.handle("removeTagFromImage", (_e, { imageId, tagId }) => { db.prepare("DELETE FROM image_tags WHERE image_id = ? AND tag_id = ?").run(imageId, tagId); return { status: "success" }; });
	ipcMain.handle("getImageTags", (_e, { imageId }) => ({ status: "success", tags: db.prepare("SELECT t.* FROM tags t JOIN image_tags it ON t.id = it.tag_id WHERE it.image_id = ?").all(imageId) as Row[] }));

	// Styles (delegated)
	ipcMain.handle("getStyles", (_e, params = {}) => styles.getStyles(params as any));
	ipcMain.handle("getStyle", (_e, params) => styles.getStyle(params));
	ipcMain.handle("createStyle", (_e, params) => styles.createStyle(params as any));
	ipcMain.handle("updateStyle", (_e, params) => styles.updateStyle(params));
	ipcMain.handle("deleteStyle", (_e, params) => styles.deleteStyle(params));
	ipcMain.handle("toggleStyleFavorite", (_e, params) => styles.toggleStyleFavorite(params));
	ipcMain.handle("getStyleCategories", () => styles.getStyleCategories());
	ipcMain.handle("getStyleBundleTypeCounts", () => styles.getStyleBundleTypeCounts());
	ipcMain.handle("getStylesDir", () => styles.getStylesDir());
	ipcMain.handle("listFilterThumbnails", () => styles.listFilterThumbnails());
	ipcMain.handle("getFilterThumbnail", (_e, params) => styles.getFilterThumbnail(params));
	ipcMain.handle("getStyleExamples", (_e, params) => styles.getStyleExamples(params));
	ipcMain.handle("createStyleExample", (_e, params) => styles.createStyleExample(params as any));
	ipcMain.handle("deleteStyleExample", (_e, params) => styles.deleteStyleExample(params));
	ipcMain.handle("getStyleLoras", (_e, params) => styles.getStyleLoras(params));
	ipcMain.handle("setStyleLoras", (_e, params) => styles.setStyleLoras(params as any));
	ipcMain.handle("getStyleTags", (_e, params) => styles.getStyleTags(params));
	ipcMain.handle("setStyleTags", (_e, params) => styles.setStyleTags(params as any));
	ipcMain.handle("getLoras", () => styles.getLoras());
	ipcMain.handle("createLora", (_e, params) => styles.createLora(params as any));

	// Batch
	ipcMain.handle("batchParseData", (_e, { content }) => batchParseData(content) as unknown as Res);
	ipcMain.handle("batchRenderTemplate", (_e, { template, rows }) => batchRenderTemplate(template, rows) as unknown as Res);

	// Inference (proxied to remote engine)
	ipcMain.handle("getGpuInfo", async () => {
		if (!engineClient.ready) return { status: "error", message: "Engine not ready" };
		try {
			const info = await engineClient.fetchJson<{ device_type: string; device_name: string | null; total_vram_gb: number | null }>("/gpu-info");
			return { status: "success", deviceType: info.device_type, deviceName: info.device_name, totalVramGb: info.total_vram_gb };
		} catch (err) { return { status: "error", message: String(err) }; }
	});
	// startEngine/stopEngine — no local process to manage. Connect/disconnect to remote.
	ipcMain.handle("startEngine", async () => {
		try { await engineClient.connect(); return { status: "success" }; }
		catch (err) { return { status: "error", message: String(err) }; }
	});
	ipcMain.handle("stopEngine", () => { engineClient.disconnect(); return { status: "success" }; });
	ipcMain.handle("engineReady", () => ({ status: "success", ready: engineClient.ready }));
	ipcMain.handle("submitJob", async (_e, params) => {
		try {
			const p = params as Record<string, any>;
			const transformerType = p.transformer_type ?? p.transformerType;

			for (const k of ["fal_aspectratio", "falAspectratio", "loras"]) {
				if (typeof p[k] === "string") {
					try { p[k] = JSON.parse(p[k]); } catch { /* leave as-is */ }
				}
			}

			// ── Cloud route: FAL ──
			if (transformerType === "fal_cloud" || transformerType === "fal_cloud_edit") {
				const falRow = db.prepare("SELECT value FROM settings WHERE key = ?").get("FAL_KEY") as { value: string } | null;
				if (!falRow?.value) return { status: "error", message: "FAL API key not configured" };

				let falEndpoint = p.fal_endpoint ?? p.falEndpoint ?? "";
				if (!falEndpoint) {
					const bundleId = p.bundle_id ?? p.bundleId;
					if (bundleId) {
						const bundle = db.prepare("SELECT fal_endpoint FROM bundles WHERE id = ?").get(bundleId) as { fal_endpoint: string } | null;
						falEndpoint = bundle?.fal_endpoint ?? "";
					}
				}
				if (!falEndpoint) return { status: "error", message: "No FAL endpoint configured for this bundle" };

				const jobId = crypto.randomUUID();
				jobMeta.set(jobId, {
					params: p,
					bundleId: p.bundleId ?? p.bundle_id,
					styleId: p.styleId ?? p.style_id,
					rawPrompt: p.rawPrompt ?? p.raw_prompt,
					startTime: performance.now(),
				});

				// Fire and forget — events will flow through falService.onEvent
				falService.submitJob(jobId, falRow.value, {
					prompt: p.prompt ?? "",
					width: p.width ?? 1024,
					height: p.height ?? 1024,
					steps: p.steps,
					cfg_scale: p.cfg_scale ?? p.cfgScale,
					seed: p.seed,
					negative_prompt: p.negative_prompt ?? p.negativePrompt,
					fal_endpoint: falEndpoint,
					fal_aspectratio: p.fal_aspectratio ?? p.falAspectratio,
					input_image_path: p.input_image_path ?? p.inputImagePath,
				});

				return { status: "success", jobId };
			}

			// ── Remote engine route ──
			if (!engineClient.ready) return { status: "error", message: "Engine not ready" };

			const result = await engineClient.fetchJson<{ job_id: string }>("/jobs", { method: "POST", body: JSON.stringify(keysToSnake(p)) });
			jobMeta.set(result.job_id, {
				params: p,
				bundleId: p.bundleId ?? p.bundle_id,
				styleId: p.styleId ?? p.style_id,
				rawPrompt: p.rawPrompt ?? p.raw_prompt,
				startTime: performance.now(),
			});
			return { status: "success", jobId: result.job_id };
		} catch (err) { return { status: "error", message: String(err) }; }
	});
	ipcMain.handle("cancelJob", async (_e, { jobId }) => {
		if (!engineClient.ready) return { status: "error", message: "Engine not ready" };
		try { await engineClient.fetch(`/jobs/${jobId}`, { method: "DELETE" }); return { status: "success" }; }
		catch (err) { return { status: "error", message: String(err) }; }
	});

	// VRAM
	ipcMain.handle("getVramUsage", async () => {
		if (!engineClient.ready) return { status: "success", available: false };
		try { const data = await engineClient.fetchJson<Res>("/vram"); return { status: "success", available: true, ...data }; }
		catch { return { status: "success", available: false }; }
	});
	ipcMain.handle("clearVramCache", async () => {
		if (!engineClient.ready) return { status: "error", message: "Engine not ready" };
		try { await engineClient.fetch("/vram/clear", { method: "POST" }); return { status: "success" }; }
		catch (err) { return { status: "error", message: String(err) }; }
	});

	// Settings pages
	ipcMain.handle("getEngineStatus", () => settings.getEngineStatus());
	ipcMain.handle("getCudaVersion", () => settings.getCudaVersion());
	ipcMain.handle("resetEngine", () => settings.resetEngine());
	ipcMain.handle("getCacheInfo", () => settings.getCacheInfo());
	ipcMain.handle("deleteCachedModel", (_e, params) => settings.deleteCachedModel(params));
	ipcMain.handle("getDataPaths", () => settings.getDataPaths());
	ipcMain.handle("getDiskUsage", () => settings.getDiskUsage());
	ipcMain.handle("getSslVerificationDisabled", () => settings.getSslVerificationDisabled());
	ipcMain.handle("setSslVerificationDisabled", (_e, params) => settings.setSslVerificationDisabled(params));
	ipcMain.handle("completeSetup", (_e, params) => settings.completeSetup(params as any));

	// Chat
	ipcMain.handle("chatIsConfigured", () => chat.isConfigured());
	ipcMain.handle("chatSetApiKey", (_e, params) => chat.setApiKey(params));
	ipcMain.handle("chatCreateConversation", (_e, params = {}) => chat.createConversation(params));
	ipcMain.handle("chatGetConversations", () => chat.getConversations());
	ipcMain.handle("chatGetMessages", (_e, params) => chat.getMessages(params));
	ipcMain.handle("chatDeleteConversation", (_e, params) => chat.deleteConversation(params));
	ipcMain.handle("chatGetProviderInfo", () => chat.getProviderInfo());
	ipcMain.handle("chatSendMessage", (_e, params) => chat.sendMessage(params as Record<string, unknown>));

	// Polling
	ipcMain.handle("pollEvents", () => {
		const batch = inferenceEventBuffer.splice(0, inferenceEventBuffer.length);
		return { status: "success", events: batch };
	});
	ipcMain.handle("pollChatEvents", () => ({ status: "success", events: chat.drainEvents() as unknown as Res[] }));

	// Debug
	ipcMain.handle("getDebugImages", () => ({ status: "success", output: null, previews: {} }));

	// Workflows
	ipcMain.handle("getWorkflows", () => ({
		status: "success",
		workflows: db.prepare("SELECT id, name, description, created_at, updated_at FROM workflows ORDER BY updated_at DESC").all() as Row[],
	}));
	ipcMain.handle("getWorkflow", (_e, { workflowId }) => {
		const row = db.prepare("SELECT * FROM workflows WHERE id = ?").get(workflowId) as Row | null;
		if (!row) return { status: "error", message: "Workflow not found" };
		return { status: "success", workflow: row };
	});
	ipcMain.handle("saveWorkflow", (_e, params) => {
		const p = params as Record<string, any>;
		const now = Math.floor(Date.now() / 1000);
		db.prepare("INSERT INTO workflows (id, name, description, graph_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET name = excluded.name, description = excluded.description, graph_json = excluded.graph_json, updated_at = excluded.updated_at").run(p.workflowId, p.name, p.description, p.graphJson, now, now);
		return { status: "success" };
	});
	ipcMain.handle("deleteWorkflow", (_e, { workflowId }) => {
		db.prepare("DELETE FROM workflows WHERE id = ?").run(workflowId);
		return { status: "success" };
	});
	ipcMain.handle("runWorkflow", (_e, params) => {
		const p = params as Record<string, any>;
		try {
			const runId = workflow.runWorkflow(p.graphJson);
			return { status: "success", run_id: runId };
		} catch (err) {
			return { status: "error", message: String(err) };
		}
	});
	ipcMain.handle("cancelWorkflow", (_e, params) => {
		const p = params as Record<string, any>;
		const cancelled = workflow.cancelWorkflow(p.runId);
		if (!cancelled) return { status: "error", message: "No active workflow run" };
		return { status: "success" };
	});
	ipcMain.handle("pollWorkflowEvents", () => ({ status: "success", events: workflow.pollEvents() as unknown as Res[] }));

	// File browsing
	ipcMain.handle("browseOutputDirectory", () => files.browseOutputDirectory());
	ipcMain.handle("browseLoraFiles", () => files.browseLoraFiles());
	ipcMain.handle("browseImageFile", (_e, params = {}) => files.browseImageFile(params as Record<string, unknown>));
	ipcMain.handle("browseInputImage", () => files.browseInputImage());
	ipcMain.handle("browseWorkflowImage", (_e, params = {}) => files.browseWorkflowImage(params as Record<string, unknown>));
	ipcMain.handle("saveClipboardImage", (_e, params) => files.saveClipboardImage(params as Record<string, unknown>));
	ipcMain.handle("saveImageAs", (_e, params) => files.saveImageAs(params as Record<string, unknown>));
	ipcMain.handle("batchSaveImages", (_e, params) => files.batchSaveImages(params as Record<string, unknown>));
	ipcMain.handle("importStyleImage", (_e, params) => files.importStyleImage(params as Record<string, unknown>));
	ipcMain.handle("browseMetadataFiles", () => files.browseMetadataFiles());
	ipcMain.handle("browseAndImportMetadata", () => files.browseAndImportMetadata());
	ipcMain.handle("importCivitaiMetadata", (_e, params) => files.importCivitaiMetadata(params as Record<string, unknown>));
	ipcMain.handle("browseInvokeaiDb", () => files.browseInvokeaiDb());
	ipcMain.handle("importInvokeaiLoras", (_e, params) => files.importInvokeaiLoras(params as Record<string, unknown>));

	// Style AI
	ipcMain.handle("reviewStyles", async (_e, params) => {
		try {
			const p = params as Record<string, any>;
			const styleIds = p.styleIds as string[] ?? [];
			if (!styleIds.length) return { status: "error", message: "No styles selected" };
			const allStyles = db.prepare("SELECT * FROM styles").all() as Row[];
			const selected = allStyles.filter((s) => styleIds.includes(s.id as string));
			if (!selected.length) return { status: "error", message: "No matching styles found" };
			const allTags = db.prepare("SELECT * FROM tags").all() as Row[];
			const tagNames = allTags.map((t) => t.name as string);
			const styleData = selected.map((s) => {
				const styleTags = db.prepare("SELECT t.name FROM tags t JOIN style_tags st ON t.id = st.tag_id WHERE st.style_id = ?").all(s.id as string) as { name: string }[];
				return { id: s.id, name: s.name, prompt: (s.prompt_template as string ?? "").slice(0, 200), tags: styleTags.map((t) => t.name) };
			});
			const prompt = `Review these image generation styles and suggest better names and tags. Existing tags: [${tagNames.join(", ")}]. Prefer reusing existing tags.\n\nStyles:\n${JSON.stringify(styleData, null, 2)}\n\nRespond with JSON array: [{"id": "...", "name": "suggested name", "tags": ["tag1", "tag2"], "reason": "why"}]`;
			const result = await chat.complete([{ role: "user", content: prompt }]);
			const jsonMatch = result.match(/\[[\s\S]*\]/);
			if (!jsonMatch) return { status: "error", message: "Failed to parse AI response" };
			const suggestions = JSON.parse(jsonMatch[0]);
			const validSuggestions = suggestions.filter((s: any) => styleIds.includes(s.id));
			return { status: "success", suggestions: validSuggestions };
		} catch (err) { return { status: "error", message: String(err) }; }
	});
	ipcMain.handle("applyStyleReview", (_e, params) => {
		try {
			const p = params as Record<string, any>;
			const changes = p.changes as Array<{ styleId: string; name?: string; tags?: string[] }> ?? [];
			for (const change of changes) {
				if (change.name) {
					db.prepare("UPDATE styles SET name = ?, updated_at = ? WHERE id = ?").run(change.name, Math.floor(Date.now() / 1000), change.styleId);
				}
				if (change.tags) {
					const tagIds: number[] = [];
					for (const tagName of change.tags) {
						let tag = db.prepare("SELECT id FROM tags WHERE name = ?").get(tagName) as { id: number } | null;
						if (!tag) {
							db.prepare("INSERT INTO tags (name, category) VALUES (?, 'style')").run(tagName);
							tag = db.prepare("SELECT id FROM tags WHERE name = ?").get(tagName) as { id: number } | null;
						}
						if (tag) tagIds.push(tag.id);
					}
					db.prepare("DELETE FROM style_tags WHERE style_id = ?").run(change.styleId);
					const insert = db.prepare("INSERT OR IGNORE INTO style_tags (style_id, tag_id) VALUES (?, ?)");
					for (const tagId of tagIds) insert.run(change.styleId, tagId);
				}
			}
			return { status: "success" };
		} catch (err) { return { status: "error", message: String(err) }; }
	});
	ipcMain.handle("generateStyleFromFilters", async (_e, params) => {
		try {
			const p = params as Record<string, any>;
			const filterNames = (p.filters ?? p.filterNames) as string[] ?? [];
			if (!filterNames.length) return { status: "error", message: "No filters selected" };
			const prompt = `Create an image generation style combining these visual filters: ${filterNames.join(", ")}.\n\nRespond with JSON: {"name": "style name", "description": "...", "prompt_template": "a {prompt} with style details...", "negative_prompt": "...", "category": "custom"}.\n\nThe prompt_template MUST contain {prompt} as a placeholder.`;
			const result = await chat.complete([{ role: "user", content: prompt }]);
			const cleaned = result.replace(/```json\n?/g, "").replace(/```\n?/g, "").trim();
			const styleData = JSON.parse(cleaned);
			if (!styleData.prompt_template?.includes("{prompt}")) {
				styleData.prompt_template = `a {prompt}, ${styleData.prompt_template ?? filterNames.join(", ")}`;
			}
			return { status: "success", styleData };
		} catch (err) { return { status: "error", message: String(err) }; }
	});
	ipcMain.handle("createStyleFromImage", async (_e, params) => {
		try {
			const p = params as Record<string, any>;
			const imagePath = p.imagePath as string;
			if (!imagePath) return { status: "error", message: "No image path provided" };
			const bytes = readFileSync(imagePath);
			const b64 = Buffer.from(bytes).toString("base64");
			const ext = imagePath.split(".").pop()?.toLowerCase() ?? "png";
			const mediaType = ext === "jpg" || ext === "jpeg" ? "image/jpeg" : ext === "webp" ? "image/webp" : "image/png";
			const result = await chat.completeWithVision([{
				role: "user",
				content: [
					{ type: "image", source: { type: "base64", media_type: mediaType as any, data: b64 } },
					{ type: "text", text: 'Analyze this image and create a reusable generation style. Respond with JSON: {"name": "style name", "description": "...", "prompt_template": "a {prompt} in this style...", "category": "custom"}. The prompt_template MUST contain {prompt}.' },
				],
			}]);
			const cleaned = result.replace(/```json\n?/g, "").replace(/```\n?/g, "").trim();
			const styleData = JSON.parse(cleaned);
			const styleId = crypto.randomUUID();
			const now = Math.floor(Date.now() / 1000);
			db.prepare("INSERT INTO styles (id, name, description, prompt_template, category, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)").run(styleId, styleData.name, styleData.description ?? "", styleData.prompt_template, styleData.category ?? "custom", now, now);
			const style = db.prepare("SELECT * FROM styles WHERE id = ?").get(styleId) as Row;
			return { status: "success", style };
		} catch (err) { return { status: "error", message: String(err) }; }
	});

	// Discovery & connection
	ipcMain.handle("getDiscoveredServers", () => ({ status: "success", servers: discoveryService.getServers() }));
	ipcMain.handle("connectToServer", async (_e, { host, port }: { host: string; port: number }) => {
		try {
			await engineClient.connect(host, port);
			db.prepare("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value").run("ENGINE_HOST", host);
			db.prepare("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value").run("ENGINE_PORT", String(port));
			return { status: "success", mode: "remote", connected_server: { host, port } };
		} catch (err) {
			return { status: "error", message: `Failed to connect to ${host}:${port}: ${String(err)}` };
		}
	});
	ipcMain.handle("disconnectFromServer", () => {
		engineClient.disconnect();
		return { status: "success", mode: "disconnected", connected_server: null };
	});
	ipcMain.handle("getConnectionMode", () => ({
		status: "success",
		mode: engineClient.ready ? "remote" : "disconnected",
		connected_server: engineClient.ready ? { host: engineClient.host, port: engineClient.port } : null,
	}));

	// Auto-update
	ipcMain.handle("installUpdate", () => {
		autoUpdater.quitAndInstall();
	});
}
