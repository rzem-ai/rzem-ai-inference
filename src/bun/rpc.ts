/**
 * Electrobun RPC schema and handler wiring.
 *
 * This file defines the typed contract between the Bun main process and
 * the Vue webview, replacing the old pywebview js_api bridge. Handlers
 * delegate to service modules for domain-specific logic.
 */

import { BrowserView, type RPCSchema } from "electrobun/bun";
import type Database from "bun:sqlite";
import type { SidecarManager } from "./sidecar";
import { batchParseData, batchRenderTemplate } from "./services/batch";
import { createStylesHandlers } from "./services/styles";
import { createSettingsHandlers } from "./services/settings";
import { createChatService } from "./services/chat";
import { createFilesHandlers } from "./services/files";

// ── Shared types ─────────────────────────────────────────────────

type Row = Record<string, unknown>;
type SQLValue = string | number | null | bigint | boolean;
type Res = Record<string, unknown>;

// ── RPC Schema ───────────────────────────────────────────────────
// Using Res (Record<string, unknown>) for most responses to keep
// the schema concise. Type safety is enforced by handler implementations.

export type AppRPCSchema = {
	bun: RPCSchema<{
		requests: {
			// ── System ──
			healthCheck: { params: {}; response: Res };

			// ── Settings (key-value) ──
			getSetting: { params: { key: string }; response: Res };
			setSetting: { params: { key: string; value: string }; response: Res };

			// ── Bundle Types ──
			getBundleTypes: { params: {}; response: Res };

			// ── Bundles ──
			getBundles: { params: { includeHidden?: boolean }; response: Res };
			getBundle: { params: { bundleId: string }; response: Res };
			getBundlesForType: { params: { transformerType: string }; response: Res };
			createBundle: { params: Res; response: Res };
			updateBundle: { params: Res; response: Res };
			deleteBundle: { params: { bundleId: string }; response: Res };
			toggleBundleFavorite: { params: { bundleId: string }; response: Res };
			toggleBundleHidden: { params: { bundleId: string }; response: Res };
			resetDefaultBundles: { params: {}; response: Res };

			// ── Gallery ──
			getGalleryImages: { params: Res; response: Res };
			getImage: { params: { imageId: string }; response: Res };
			toggleFavorite: { params: { imageId: string }; response: Res };
			deleteImage: { params: { imageId: string }; response: Res };
			getImageBase64: { params: { imagePath: string }; response: Res };

			// ── Folders ──
			getFolders: { params: {}; response: Res };
			createFolder: { params: Res; response: Res };
			updateFolder: { params: Res; response: Res };
			deleteFolder: { params: { folderId: string }; response: Res };
			addImageToFolder: { params: { imageId: string; folderId: string }; response: Res };
			removeImageFromFolder: { params: { imageId: string; folderId: string }; response: Res };

			// ── Tags ──
			getTags: { params: {}; response: Res };
			createTag: { params: { name: string; color?: string; category?: string }; response: Res };
			updateTag: { params: Res; response: Res };
			deleteTag: { params: { tagId: number }; response: Res };
			addTagToImage: { params: { imageId: string; tagId: number }; response: Res };
			removeTagFromImage: { params: { imageId: string; tagId: number }; response: Res };
			getImageTags: { params: { imageId: string }; response: Res };

			// ── Styles ──
			getStyles: { params: Res; response: Res };
			getStyle: { params: { styleId: string }; response: Res };
			createStyle: { params: Res; response: Res };
			updateStyle: { params: Res; response: Res };
			deleteStyle: { params: { styleId: string }; response: Res };
			toggleStyleFavorite: { params: { styleId: string }; response: Res };
			getStyleCategories: { params: {}; response: Res };
			getStyleBundleTypeCounts: { params: {}; response: Res };
			getStylesDir: { params: {}; response: Res };
			listFilterThumbnails: { params: {}; response: Res };
			getFilterThumbnail: { params: { imageName: string }; response: Res };

			// ── Style Examples ──
			getStyleExamples: { params: { styleId: string }; response: Res };
			createStyleExample: { params: Res; response: Res };
			deleteStyleExample: { params: { exampleId: string }; response: Res };

			// ── Style ↔ LoRA ──
			getStyleLoras: { params: { styleId: string }; response: Res };
			setStyleLoras: { params: Res; response: Res };

			// ── Style ↔ Tag ──
			getStyleTags: { params: { styleId: string }; response: Res };
			setStyleTags: { params: Res; response: Res };

			// ── LoRAs ──
			getLoras: { params: {}; response: Res };
			createLora: { params: Res; response: Res };

			// ── Batch ──
			batchParseData: { params: { content: string }; response: Res };
			batchRenderTemplate: { params: { template: string; rows: Record<string, string>[] }; response: Res };

			// ── Inference (proxied to sidecar) ──
			getGpuInfo: { params: {}; response: Res };
			startEngine: { params: { device?: string; vramLimitGb?: number }; response: Res };
			stopEngine: { params: {}; response: Res };
			engineReady: { params: {}; response: Res };
			submitJob: { params: Res; response: Res };
			cancelJob: { params: { jobId: string }; response: Res };

			// ── VRAM ──
			getVramUsage: { params: {}; response: Res };
			clearVramCache: { params: {}; response: Res };

			// ── Settings Pages ──
			getEngineStatus: { params: {}; response: Res };
			getCudaVersion: { params: {}; response: Res };
			resetEngine: { params: {}; response: Res };
			getCacheInfo: { params: {}; response: Res };
			deleteCachedModel: { params: { repoId: string }; response: Res };
			getDataPaths: { params: {}; response: Res };
			getDiskUsage: { params: {}; response: Res };

			// ── SSL ──
			getSslVerificationDisabled: { params: {}; response: Res };
			setSslVerificationDisabled: { params: { disabled: boolean }; response: Res };

			// ── Setup ──
			completeSetup: { params: Res; response: Res };

			// ── Chat ──
			chatIsConfigured: { params: {}; response: Res };
			chatSetApiKey: { params: { apiKey: string; provider?: string }; response: Res };
			chatCreateConversation: { params: { title?: string }; response: Res };
			chatGetConversations: { params: {}; response: Res };
			chatGetMessages: { params: { conversationId: string }; response: Res };
			chatDeleteConversation: { params: { conversationId: string }; response: Res };
			chatGetProviderInfo: { params: {}; response: Res };
			chatSendMessage: { params: Res; response: Res };

			// ── Polling (compatibility) ──
			pollEvents: { params: {}; response: Res };
			pollChatEvents: { params: {}; response: Res };

			// ── Debug ──
			getDebugImages: { params: {}; response: Res };

			// ── Workflows ──
			getWorkflows: { params: Res; response: Res };
			getWorkflow: { params: { workflowId: string }; response: Res };
			saveWorkflow: { params: Res; response: Res };
			deleteWorkflow: { params: { workflowId: string }; response: Res };
			runWorkflow: { params: Res; response: Res };
			cancelWorkflow: { params: Res; response: Res };
			pollWorkflowEvents: { params: Res; response: Res };

			// ── File browsing ──
			browseOutputDirectory: { params: {}; response: Res };
			browseLoraFiles: { params: {}; response: Res };
			browseImageFile: { params: Res; response: Res };
			browseInputImage: { params: {}; response: Res };
			browseWorkflowImage: { params: Res; response: Res };
			saveClipboardImage: { params: Res; response: Res };
			saveImageAs: { params: Res; response: Res };
			batchSaveImages: { params: Res; response: Res };
			importStyleImage: { params: Res; response: Res };
			browseMetadataFiles: { params: {}; response: Res };
			browseAndImportMetadata: { params: {}; response: Res };
			importCivitaiMetadata: { params: Res; response: Res };

			// ── Style AI ──
			reviewStyles: { params: Res; response: Res };
			applyStyleReview: { params: Res; response: Res };
			generateStyleFromFilters: { params: Res; response: Res };
			createStyleFromImage: { params: Res; response: Res };

			// ── Discovery ──
			getDiscoveredServers: { params: {}; response: Res };
			connectToServer: { params: Res; response: Res };
			disconnectFromServer: { params: {}; response: Res };
			getConnectionMode: { params: {}; response: Res };
		};
		messages: {};
	}>;
	webview: RPCSchema<{
		requests: {};
		messages: {
			inferenceEvent: { event: string; jobId: string; data: Res };
			chatEvent: { event: string; data: Res };
			sidecarStatus: { ready: boolean; pid: number | null };
		};
	}>;
};

// ── Handler factory ──────────────────────────────────────────────

export function defineAppRPC(
	db: Database,
	sidecar: SidecarManager,
	config: { outputDir: string; stylesDir: string },
) {
	const styles = createStylesHandlers(db, config.stylesDir);
	const settings = createSettingsHandlers(db, sidecar, config.outputDir);
	const chat = createChatService(db);
	const files = createFilesHandlers(db, config);

	// Event buffer for polling compatibility (push events are preferred)
	const inferenceEventBuffer: Res[] = [];

	const rpc = BrowserView.defineRPC<AppRPCSchema>({
		maxRequestTime: 30_000,
		handlers: {
			requests: {
				// ── System ──
				healthCheck: () => ({ status: "ok" }),

				// ── Settings (key-value) ──
				getSetting: ({ key }) => {
					const row = db.prepare("SELECT value FROM settings WHERE key = ?").get(key) as { value: string } | null;
					return { status: "success", value: row?.value ?? null };
				},
				setSetting: ({ key, value }) => {
					db.prepare("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value").run(key, value);
					return { status: "success" };
				},

				// ── Bundle Types ──
				getBundleTypes: () => {
					const rows = db.prepare("SELECT * FROM bundle_types ORDER BY sort_order").all() as Row[];
					return { status: "success", bundleTypes: rows };
				},

				// ── Bundles ──
				getBundles: ({ includeHidden }) => {
					const q = includeHidden ? "SELECT * FROM bundles ORDER BY transformer_type, tier" : "SELECT * FROM bundles WHERE hidden = 0 ORDER BY transformer_type, tier";
					return { status: "success", bundles: db.prepare(q).all() as Row[] };
				},
				getBundle: ({ bundleId }) => {
					const row = db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId) as Row | null;
					if (!row) return { status: "error", message: "Bundle not found" };
					return { status: "success", bundle: row };
				},
				getBundlesForType: ({ transformerType }) => {
					return { status: "success", bundles: db.prepare("SELECT * FROM bundles WHERE transformer_type = ? AND hidden = 0 ORDER BY tier").all(transformerType) as Row[] };
				},
				createBundle: (params) => {
					const p = params as Record<string, any>;
					db.prepare(`INSERT INTO bundles (id, label, description, transformer_type, tier, transformer_model, vae_model, clip_tokenizer, clip_encoder, t5_tokenizer, t5_encoder, t5_encoder_config, qwen3_tokenizer, qwen3_encoder, steps, cfg_scale, sampler, scheduler, vram_estimate_gb, is_default, source, fal_endpoint, fal_aspectratio) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`).run(
						p.id, p.label, p.description ?? "", p.transformerType, p.tier ?? "balanced", p.transformerModel, p.vaeModel,
						p.clipTokenizer ?? null, p.clipEncoder ?? null, p.t5Tokenizer ?? null, p.t5Encoder ?? null, p.t5EncoderConfig ?? null,
						p.qwen3Tokenizer ?? null, p.qwen3Encoder ?? null, p.steps ?? 20, p.cfgScale ?? 1.0, p.sampler ?? "euler",
						p.scheduler ?? "normal", p.vramEstimateGb ?? 0.0, p.isDefault ?? 1, p.source ?? "local", p.falEndpoint ?? null, p.falAspectratio ?? null,
					);
					return { status: "success", bundle: db.prepare("SELECT * FROM bundles WHERE id = ?").get(p.id) as Row };
				},
				updateBundle: (params) => {
					const { bundleId, ...fields } = params as Record<string, any>;
					const sets: string[] = [];
					const vals: SQLValue[] = [];
					const map: Record<string, string> = { label: "label", description: "description", tier: "tier", transformerModel: "transformer_model", vaeModel: "vae_model", steps: "steps", cfgScale: "cfg_scale", sampler: "sampler", scheduler: "scheduler", vramEstimateGb: "vram_estimate_gb", clipTokenizer: "clip_tokenizer", clipEncoder: "clip_encoder", t5Tokenizer: "t5_tokenizer", t5Encoder: "t5_encoder", t5EncoderConfig: "t5_encoder_config", qwen3Tokenizer: "qwen3_tokenizer", qwen3Encoder: "qwen3_encoder", source: "source", falEndpoint: "fal_endpoint", falAspectratio: "fal_aspectratio" };
					for (const [k, col] of Object.entries(map)) { if (k in fields) { sets.push(`${col} = ?`); vals.push(fields[k] as SQLValue); } }
					if (!sets.length) return { status: "error", message: "No fields to update" };
					vals.push(bundleId as string);
					db.prepare(`UPDATE bundles SET ${sets.join(", ")} WHERE id = ?`).run(...vals);
					return { status: "success", bundle: db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId) as Row };
				},
				deleteBundle: ({ bundleId }) => { db.prepare("DELETE FROM bundles WHERE id = ?").run(bundleId); return { status: "success" }; },
				toggleBundleFavorite: ({ bundleId }) => {
					db.prepare("UPDATE bundles SET favorite = CASE WHEN favorite = 0 THEN 1 ELSE 0 END WHERE id = ?").run(bundleId);
					return { status: "success", bundle: db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId) as Row };
				},
				toggleBundleHidden: ({ bundleId }) => {
					db.prepare("UPDATE bundles SET hidden = CASE WHEN hidden = 0 THEN 1 ELSE 0 END WHERE id = ?").run(bundleId);
					return { status: "success", bundle: db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId) as Row };
				},
				resetDefaultBundles: () => {
					// TODO: implement with default bundle data
					return { status: "success", bundles: db.prepare("SELECT * FROM bundles WHERE is_default = 1").all() as Row[] };
				},

				// ── Gallery ──
				getGalleryImages: (params) => {
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
				},
				getImage: ({ imageId }) => {
					const img = db.prepare("SELECT * FROM images WHERE id = ?").get(imageId) as Row | null;
					if (!img) return { status: "error", message: "Image not found" };
					return { status: "success", image: img };
				},
				toggleFavorite: ({ imageId }) => {
					db.prepare("UPDATE images SET favorite = CASE WHEN favorite = 0 THEN 1 ELSE 0 END WHERE id = ?").run(imageId);
					return { status: "success", image: db.prepare("SELECT * FROM images WHERE id = ?").get(imageId) as Row };
				},
				deleteImage: ({ imageId }) => { db.prepare("DELETE FROM images WHERE id = ?").run(imageId); return { status: "success" }; },
				getImageBase64: ({ imagePath }) => {
					try {
						const file = Bun.file(imagePath);
						if (!file.size) return { status: "error", message: "File not found" };
						const ext = imagePath.split(".").pop()?.toLowerCase() ?? "png";
						const mime = ext === "jpg" || ext === "jpeg" ? "image/jpeg" : ext === "webp" ? "image/webp" : "image/png";
						// Read synchronously for simplicity
						const bytes = require("fs").readFileSync(imagePath);
						const b64 = Buffer.from(bytes).toString("base64");
						return { status: "success", dataUrl: `data:${mime};base64,${b64}` };
					} catch (err) {
						return { status: "error", message: String(err) };
					}
				},

				// ── Folders ──
				getFolders: () => ({ status: "success", folders: db.prepare("SELECT * FROM folders ORDER BY sort_order, name").all() as Row[] }),
				createFolder: (params) => {
					const p = params as Record<string, any>;
					const now = Math.floor(Date.now() / 1000);
					db.prepare("INSERT INTO folders (id, name, parent_id, color, icon, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)").run(p.id, p.name, p.parentId ?? null, p.color ?? null, p.icon ?? null, p.sortOrder ?? 0, now, now);
					return { status: "success", folder: db.prepare("SELECT * FROM folders WHERE id = ?").get(p.id) as Row };
				},
				updateFolder: (params) => {
					const { folderId, ...fields } = params as Record<string, any>;
					const sets: string[] = []; const vals: SQLValue[] = [];
					const map: Record<string, string> = { name: "name", parentId: "parent_id", color: "color", icon: "icon", sortOrder: "sort_order" };
					for (const [k, col] of Object.entries(map)) { if (k in fields) { sets.push(`${col} = ?`); vals.push(fields[k] as SQLValue); } }
					if (sets.length) { sets.push("updated_at = ?"); vals.push(Math.floor(Date.now() / 1000)); vals.push(folderId as string); db.prepare(`UPDATE folders SET ${sets.join(", ")} WHERE id = ?`).run(...vals); }
					return { status: "success", folder: db.prepare("SELECT * FROM folders WHERE id = ?").get(folderId) as Row };
				},
				deleteFolder: ({ folderId }) => { db.prepare("DELETE FROM folders WHERE id = ?").run(folderId); return { status: "success" }; },
				addImageToFolder: ({ imageId, folderId }) => { db.prepare("INSERT OR IGNORE INTO image_folders (image_id, folder_id, added_at) VALUES (?, ?, ?)").run(imageId, folderId, Math.floor(Date.now() / 1000)); return { status: "success" }; },
				removeImageFromFolder: ({ imageId, folderId }) => { db.prepare("DELETE FROM image_folders WHERE image_id = ? AND folder_id = ?").run(imageId, folderId); return { status: "success" }; },

				// ── Tags ──
				getTags: () => ({ status: "success", tags: db.prepare("SELECT * FROM tags ORDER BY name").all() as Row[] }),
				createTag: ({ name, color, category }) => {
					db.prepare("INSERT INTO tags (name, color, category) VALUES (?, ?, ?)").run(name, color ?? null, category ?? null);
					return { status: "success", tag: db.prepare("SELECT * FROM tags WHERE name = ?").get(name) as Row };
				},
				updateTag: (params) => {
					const p = params as Record<string, any>;
					const sets: string[] = []; const vals: SQLValue[] = [];
					if (p.name !== undefined) { sets.push("name = ?"); vals.push(p.name); }
					if (p.color !== undefined) { sets.push("color = ?"); vals.push(p.color ?? null); }
					if (p.category !== undefined) { sets.push("category = ?"); vals.push(p.category ?? null); }
					if (sets.length) { vals.push(p.tagId); db.prepare(`UPDATE tags SET ${sets.join(", ")} WHERE id = ?`).run(...vals); }
					return { status: "success", tag: db.prepare("SELECT * FROM tags WHERE id = ?").get(p.tagId) as Row };
				},
				deleteTag: ({ tagId }) => { db.prepare("DELETE FROM tags WHERE id = ?").run(tagId); return { status: "success" }; },
				addTagToImage: ({ imageId, tagId }) => { db.prepare("INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?, ?)").run(imageId, tagId); return { status: "success" }; },
				removeTagFromImage: ({ imageId, tagId }) => { db.prepare("DELETE FROM image_tags WHERE image_id = ? AND tag_id = ?").run(imageId, tagId); return { status: "success" }; },
				getImageTags: ({ imageId }) => ({ status: "success", tags: db.prepare("SELECT t.* FROM tags t JOIN image_tags it ON t.id = it.tag_id WHERE it.image_id = ?").all(imageId) as Row[] }),

				// ── Styles (delegated to service) ──
				getStyles: (params) => styles.getStyles(params as any),
				getStyle: (params) => styles.getStyle(params),
				createStyle: (params) => styles.createStyle(params as any),
				updateStyle: (params) => styles.updateStyle(params),
				deleteStyle: (params) => styles.deleteStyle(params),
				toggleStyleFavorite: (params) => styles.toggleStyleFavorite(params),
				getStyleCategories: () => styles.getStyleCategories(),
				getStyleBundleTypeCounts: () => styles.getStyleBundleTypeCounts(),
				getStylesDir: () => styles.getStylesDir(),
				listFilterThumbnails: () => styles.listFilterThumbnails(),
				getFilterThumbnail: (params) => styles.getFilterThumbnail(params),
				getStyleExamples: (params) => styles.getStyleExamples(params),
				createStyleExample: (params) => styles.createStyleExample(params as any),
				deleteStyleExample: (params) => styles.deleteStyleExample(params),
				getStyleLoras: (params) => styles.getStyleLoras(params),
				setStyleLoras: (params) => styles.setStyleLoras(params as any),
				getStyleTags: (params) => styles.getStyleTags(params),
				setStyleTags: (params) => styles.setStyleTags(params as any),
				getLoras: () => styles.getLoras(),
				createLora: (params) => styles.createLora(params as any),

				// ── Batch (delegated to service) ──
				batchParseData: ({ content }) => batchParseData(content) as unknown as Res,
				batchRenderTemplate: ({ template, rows }) => batchRenderTemplate(template, rows) as unknown as Res,

				// ── Inference (proxied to sidecar) ──
				getGpuInfo: async () => {
					if (!sidecar.ready) return { status: "error", message: "Engine not ready" };
					try {
						const info = await sidecar.fetchJson<{ device_type: string; device_name: string | null; total_vram_gb: number | null }>("/gpu-info");
						return { status: "success", deviceType: info.device_type, deviceName: info.device_name, totalVramGb: info.total_vram_gb };
					} catch (err) { return { status: "error", message: String(err) }; }
				},
				startEngine: async () => {
					try { await sidecar.start(); return { status: "success" }; }
					catch (err) { return { status: "error", message: String(err) }; }
				},
				stopEngine: () => { sidecar.stop(); return { status: "success" }; },
				engineReady: () => ({ status: "success", ready: sidecar.ready }),
				submitJob: async (params) => {
					if (!sidecar.ready) return { status: "error", message: "Engine not ready" };
					try {
						const result = await sidecar.fetchJson<{ job_id: string }>("/jobs", { method: "POST", body: JSON.stringify(params) });
						return { status: "success", jobId: result.job_id };
					} catch (err) { return { status: "error", message: String(err) }; }
				},
				cancelJob: async ({ jobId }) => {
					if (!sidecar.ready) return { status: "error", message: "Engine not ready" };
					try { await sidecar.fetch(`/jobs/${jobId}`, { method: "DELETE" }); return { status: "success" }; }
					catch (err) { return { status: "error", message: String(err) }; }
				},

				// ── VRAM (proxied to sidecar) ──
				getVramUsage: async () => {
					if (!sidecar.ready) return { status: "success", available: false };
					try { const data = await sidecar.fetchJson<Res>("/vram"); return { status: "success", available: true, ...data }; }
					catch { return { status: "success", available: false }; }
				},
				clearVramCache: async () => {
					if (!sidecar.ready) return { status: "error", message: "Engine not ready" };
					try { await sidecar.fetch("/vram/clear", { method: "POST" }); return { status: "success" }; }
					catch (err) { return { status: "error", message: String(err) }; }
				},

				// ── Settings pages (delegated to service) ──
				getEngineStatus: () => settings.getEngineStatus(),
				getCudaVersion: () => settings.getCudaVersion(),
				resetEngine: () => settings.resetEngine(),
				getCacheInfo: () => settings.getCacheInfo(),
				deleteCachedModel: (params) => settings.deleteCachedModel(params),
				getDataPaths: () => settings.getDataPaths(),
				getDiskUsage: () => settings.getDiskUsage(),
				getSslVerificationDisabled: () => settings.getSslVerificationDisabled(),
				setSslVerificationDisabled: (params) => settings.setSslVerificationDisabled(params),
				completeSetup: (params) => settings.completeSetup(params as any),

				// ── Chat (delegated to service) ──
				chatIsConfigured: () => chat.isConfigured(),
				chatSetApiKey: (params) => chat.setApiKey(params),
				chatCreateConversation: (params) => chat.createConversation(params),
				chatGetConversations: () => chat.getConversations(),
				chatGetMessages: (params) => chat.getMessages(params),
				chatDeleteConversation: (params) => chat.deleteConversation(params),
				chatGetProviderInfo: () => chat.getProviderInfo(),
				chatSendMessage: (params) => chat.sendMessage(params as Record<string, unknown>),

				// ── Polling (compatibility — push events preferred) ──
				pollEvents: () => {
					const batch = inferenceEventBuffer.splice(0, inferenceEventBuffer.length);
					return { status: "success", events: batch };
				},
				pollChatEvents: () => ({ status: "success", events: chat.drainEvents() as unknown as Res[] }),

				// ── Debug ──
				getDebugImages: () => ({ status: "success", output: null, previews: {} }),

				// ── Workflows (DB CRUD) ──
				getWorkflows: () => ({
					status: "success",
					workflows: db.prepare("SELECT id, name, description, created_at, updated_at FROM workflows ORDER BY updated_at DESC").all() as Row[],
				}),
				getWorkflow: ({ workflowId }: { workflowId: string }) => {
					const row = db.prepare("SELECT * FROM workflows WHERE id = ?").get(workflowId) as Row | null;
					if (!row) return { status: "error", message: "Workflow not found" };
					return { status: "success", workflow: row };
				},
				saveWorkflow: (params) => {
					const p = params as Record<string, any>;
					const now = Math.floor(Date.now() / 1000);
					db.prepare("INSERT INTO workflows (id, name, description, graph_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET name = excluded.name, description = excluded.description, graph_json = excluded.graph_json, updated_at = excluded.updated_at").run(p.workflowId, p.name, p.description, p.graphJson, now, now);
					return { status: "success" };
				},
				deleteWorkflow: ({ workflowId }: { workflowId: string }) => {
					db.prepare("DELETE FROM workflows WHERE id = ?").run(workflowId);
					return { status: "success" };
				},
				runWorkflow: () => ({ status: "error", message: "Workflow execution not yet implemented" }),
				cancelWorkflow: () => ({ status: "success" }),
				pollWorkflowEvents: () => ({ status: "success", events: [] as Res[] }),

				// ── File browsing (delegated to service) ──
				browseOutputDirectory: () => files.browseOutputDirectory(),
				browseLoraFiles: () => files.browseLoraFiles(),
				browseImageFile: (params) => files.browseImageFile(params as Record<string, unknown>),
				browseInputImage: () => files.browseInputImage(),
				browseWorkflowImage: (params) => files.browseWorkflowImage(params as Record<string, unknown>),
				saveClipboardImage: (params) => files.saveClipboardImage(params as Record<string, unknown>),
				saveImageAs: (params) => files.saveImageAs(params as Record<string, unknown>),
				batchSaveImages: (params) => files.batchSaveImages(params as Record<string, unknown>),
				importStyleImage: (params) => files.importStyleImage(params as Record<string, unknown>),
				browseMetadataFiles: () => files.browseMetadataFiles(),
				browseAndImportMetadata: () => files.browseAndImportMetadata(),
				importCivitaiMetadata: (params) => files.importCivitaiMetadata(params as Record<string, unknown>),

				// ── Style AI (uses chat service for AI completions) ──
				reviewStyles: async (params) => {
					try {
						const p = params as Record<string, any>;
						const styleIds = p.styleIds as string[] ?? [];
						if (!styleIds.length) return { status: "error", message: "No styles selected" };

						const allStyles = db.prepare("SELECT * FROM styles").all() as Row[];
						const selected = allStyles.filter((s) => styleIds.includes(s.id as string));
						if (!selected.length) return { status: "error", message: "No matching styles found" };

						const allTags = db.prepare("SELECT * FROM tags").all() as Row[];
						const tagNames = allTags.map((t) => t.name as string);

						// Build compact style data for AI review
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
				},

				applyStyleReview: (params) => {
					try {
						const p = params as Record<string, any>;
						const changes = p.changes as Array<{ id: string; name?: string; tags?: string[] }> ?? [];
						for (const change of changes) {
							if (change.name) {
								db.prepare("UPDATE styles SET name = ?, updated_at = ? WHERE id = ?").run(change.name, Math.floor(Date.now() / 1000), change.id);
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
								// Replace style tags
								db.prepare("DELETE FROM style_tags WHERE style_id = ?").run(change.id);
								const insert = db.prepare("INSERT OR IGNORE INTO style_tags (style_id, tag_id) VALUES (?, ?)");
								for (const tagId of tagIds) insert.run(change.id, tagId);
							}
						}
						return { status: "success" };
					} catch (err) { return { status: "error", message: String(err) }; }
				},

				generateStyleFromFilters: async (params) => {
					try {
						const p = params as Record<string, any>;
						const filterNames = p.filterNames as string[] ?? [];
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
				},

				createStyleFromImage: async (params) => {
					try {
						const p = params as Record<string, any>;
						const imagePath = p.imagePath as string;
						if (!imagePath) return { status: "error", message: "No image path provided" };

						const bytes = require("fs").readFileSync(imagePath);
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
						db.prepare(
							"INSERT INTO styles (id, name, description, prompt_template, category, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
						).run(styleId, styleData.name, styleData.description ?? "", styleData.prompt_template, styleData.category ?? "custom", now, now);

						const style = db.prepare("SELECT * FROM styles WHERE id = ?").get(styleId) as Row;
						return { status: "success", style };
					} catch (err) { return { status: "error", message: String(err) }; }
				},

				// ── Discovery (stub — will use multicast-dns npm) ──
				getDiscoveredServers: () => ({ status: "success", servers: [] as Res[] }),
				connectToServer: () => ({ status: "success", mode: "local" }),
				disconnectFromServer: () => ({ status: "success", mode: "local" }),
				getConnectionMode: () => ({ status: "success", mode: "local", connectedServer: null }),
			},
			messages: {},
		},
	});

	// Wire chat events → push messages to webview
	chat.onEvent((event) => {
		try {
			rpc.send.chatEvent({
				event: event.type,
				data: event.data,
			});
		} catch {
			// Window may not be ready yet
		}
	});

	// Wire sidecar events → both push messages and polling buffer
	sidecar.onEvent((event) => {
		const mapped: Res = {
			type: (event.event as string) ?? "unknown",
			data: (event.data as Res) ?? {},
		};
		inferenceEventBuffer.push(mapped);
		if (inferenceEventBuffer.length > 500) {
			inferenceEventBuffer.splice(0, inferenceEventBuffer.length - 500);
		}

		try {
			rpc.send.inferenceEvent({
				event: (event.event as string) ?? "unknown",
				jobId: (event.job_id as string) ?? "",
				data: (event.data as Res) ?? {},
			});
		} catch {
			// Window may not be ready yet
		}
	});

	return rpc;
}
