import { BrowserView, type RPCSchema } from "electrobun/bun";
import type Database from "bun:sqlite";
import type { SidecarManager } from "./sidecar";

// ── Response types ───────────────────────────────────────────────

type Row = Record<string, unknown>;
type SQLValue = string | number | null | bigint | boolean;

interface ApiResponse {
	status: "success" | "error";
	message?: string;
}

// ── RPC Schema ───────────────────────────────────────────────────
// This defines the typed contract between the Bun main process and
// the Vue webview, replacing the old pywebview js_api bridge.

export type AppRPCSchema = {
	bun: RPCSchema<{
		requests: {
			// ── System ──
			healthCheck: {
				params: {};
				response: { status: string };
			};

			// ── Settings (key-value) ──
			getSetting: {
				params: { key: string };
				response: ApiResponse & { value?: string | null };
			};
			setSetting: {
				params: { key: string; value: string };
				response: ApiResponse;
			};

			// ── Bundle Types ──
			getBundleTypes: {
				params: {};
				response: ApiResponse & { bundleTypes?: Array<Record<string, unknown>> };
			};

			// ── Bundles ──
			getBundles: {
				params: { includeHidden?: boolean };
				response: ApiResponse & { bundles?: Array<Record<string, unknown>> };
			};
			getBundle: {
				params: { bundleId: string };
				response: ApiResponse & { bundle?: Record<string, unknown> };
			};
			getBundlesForType: {
				params: { transformerType: string };
				response: ApiResponse & { bundles?: Array<Record<string, unknown>> };
			};
			createBundle: {
				params: Record<string, unknown>;
				response: ApiResponse & { bundle?: Record<string, unknown> };
			};
			updateBundle: {
				params: Record<string, unknown>;
				response: ApiResponse & { bundle?: Record<string, unknown> };
			};
			deleteBundle: {
				params: { bundleId: string };
				response: ApiResponse;
			};
			toggleBundleFavorite: {
				params: { bundleId: string };
				response: ApiResponse & { bundle?: Record<string, unknown> };
			};
			toggleBundleHidden: {
				params: { bundleId: string };
				response: ApiResponse & { bundle?: Record<string, unknown> };
			};

			// ── Gallery ──
			getGalleryImages: {
				params: {
					limit?: number;
					offset?: number;
					folderId?: string;
					tagId?: number;
					search?: string;
					favoritesOnly?: boolean;
					sortBy?: string;
					sortOrder?: string;
				};
				response: ApiResponse & {
					images?: Array<Record<string, unknown>>;
					total?: number;
				};
			};
			getImage: {
				params: { imageId: string };
				response: ApiResponse & { image?: Record<string, unknown> };
			};
			toggleFavorite: {
				params: { imageId: string };
				response: ApiResponse & { image?: Record<string, unknown> };
			};
			deleteImage: {
				params: { imageId: string };
				response: ApiResponse;
			};

			// ── Folders ──
			getFolders: {
				params: {};
				response: ApiResponse & { folders?: Array<Record<string, unknown>> };
			};
			createFolder: {
				params: {
					id: string;
					name: string;
					parentId?: string;
					color?: string;
					icon?: string;
					sortOrder?: number;
				};
				response: ApiResponse & { folder?: Record<string, unknown> };
			};
			updateFolder: {
				params: Record<string, unknown>;
				response: ApiResponse & { folder?: Record<string, unknown> };
			};
			deleteFolder: {
				params: { folderId: string };
				response: ApiResponse;
			};
			addImageToFolder: {
				params: { imageId: string; folderId: string };
				response: ApiResponse;
			};
			removeImageFromFolder: {
				params: { imageId: string; folderId: string };
				response: ApiResponse;
			};

			// ── Tags ──
			getTags: {
				params: {};
				response: ApiResponse & { tags?: Array<Record<string, unknown>> };
			};
			createTag: {
				params: { name: string; color?: string; category?: string };
				response: ApiResponse & { tag?: Record<string, unknown> };
			};
			updateTag: {
				params: { tagId: number; name?: string; color?: string; category?: string };
				response: ApiResponse & { tag?: Record<string, unknown> };
			};
			deleteTag: {
				params: { tagId: number };
				response: ApiResponse;
			};
			addTagToImage: {
				params: { imageId: string; tagId: number };
				response: ApiResponse;
			};
			removeTagFromImage: {
				params: { imageId: string; tagId: number };
				response: ApiResponse;
			};
			getImageTags: {
				params: { imageId: string };
				response: ApiResponse & { tags?: Array<Record<string, unknown>> };
			};

			// ── Inference (proxied to sidecar) ──
			getGpuInfo: {
				params: {};
				response: ApiResponse & {
					deviceType?: string;
					deviceName?: string | null;
					totalVramGb?: number;
				};
			};
			startEngine: {
				params: { device?: string; vramLimitGb?: number };
				response: ApiResponse;
			};
			stopEngine: {
				params: {};
				response: ApiResponse;
			};
			engineReady: {
				params: {};
				response: ApiResponse & { ready: boolean };
			};
			submitJob: {
				params: Record<string, unknown>;
				response: ApiResponse & { jobId?: string };
			};
			cancelJob: {
				params: { jobId: string };
				response: ApiResponse;
			};

			// ── VRAM ──
			getVramUsage: {
				params: {};
				response: ApiResponse & {
					available?: boolean;
					allocated?: number;
					reserved?: number;
					free?: number;
					total?: number;
				};
			};
			clearVramCache: {
				params: {};
				response: ApiResponse;
			};
		};
		messages: {};
	}>;
	webview: RPCSchema<{
		requests: {};
		messages: {
			// Push events from bun → webview (replaces polling)
			inferenceEvent: {
				event: string;
				jobId: string;
				data: Record<string, unknown>;
			};
			sidecarStatus: {
				ready: boolean;
				pid: number | null;
			};
		};
	}>;
};

/**
 * Define the Electrobun RPC with handlers wired to the database and sidecar.
 */
export function defineAppRPC(db: Database, sidecar: SidecarManager) {
	const rpc = BrowserView.defineRPC<AppRPCSchema>({
		maxRequestTime: 30_000,
		handlers: {
			requests: {
				// ── System ──
				healthCheck: () => ({ status: "ok" }),

				// ── Settings ──
				getSetting: ({ key }) => {
					const row = db
						.prepare("SELECT value FROM settings WHERE key = ?")
						.get(key) as { value: string } | null;
					return { status: "success", value: row?.value ?? null };
				},
				setSetting: ({ key, value }) => {
					db.prepare(
						"INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
					).run(key, value);
					return { status: "success" };
				},

				// ── Bundle Types ──
				getBundleTypes: () => {
					const rows = db
						.prepare("SELECT * FROM bundle_types ORDER BY sort_order")
						.all() as Row[];
					return { status: "success", bundleTypes: rows };
				},

				// ── Bundles ──
				getBundles: ({ includeHidden }) => {
					const query = includeHidden
						? "SELECT * FROM bundles ORDER BY transformer_type, tier"
						: "SELECT * FROM bundles WHERE hidden = 0 ORDER BY transformer_type, tier";
					const rows = db.prepare(query).all() as Row[];
					return { status: "success", bundles: rows };
				},
				getBundle: ({ bundleId }) => {
					const row = db
						.prepare("SELECT * FROM bundles WHERE id = ?")
						.get(bundleId);
					if (!row) return { status: "error", message: "Bundle not found" };
					return { status: "success", bundle: row as Record<string, unknown> };
				},
				getBundlesForType: ({ transformerType }) => {
					const rows = db
						.prepare(
							"SELECT * FROM bundles WHERE transformer_type = ? AND hidden = 0 ORDER BY tier",
						)
						.all(transformerType) as Row[];
					return { status: "success", bundles: rows };
				},
				createBundle: (params) => {
					const {
						id, label, description, transformerType, tier,
						transformerModel, vaeModel, ...rest
					} = params as Record<string, any>;
					db.prepare(`
						INSERT INTO bundles (id, label, description, transformer_type, tier, transformer_model, vae_model,
							clip_tokenizer, clip_encoder, t5_tokenizer, t5_encoder, t5_encoder_config,
							qwen3_tokenizer, qwen3_encoder, steps, cfg_scale, sampler, scheduler,
							vram_estimate_gb, is_default, source, fal_endpoint, fal_aspectratio)
						VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
					`).run(
						id, label, description ?? "", transformerType, tier ?? "balanced",
						transformerModel, vaeModel,
						rest.clipTokenizer ?? null, rest.clipEncoder ?? null,
						rest.t5Tokenizer ?? null, rest.t5Encoder ?? null, rest.t5EncoderConfig ?? null,
						rest.qwen3Tokenizer ?? null, rest.qwen3Encoder ?? null,
						rest.steps ?? 20, rest.cfgScale ?? 1.0, rest.sampler ?? "euler",
						rest.scheduler ?? "normal", rest.vramEstimateGb ?? 0.0,
						rest.isDefault ?? 1, rest.source ?? "local",
						rest.falEndpoint ?? null, rest.falAspectratio ?? null,
					);
					const bundle = db.prepare("SELECT * FROM bundles WHERE id = ?").get(id);
					return { status: "success", bundle: bundle as Record<string, unknown> };
				},
				updateBundle: (params) => {
					const { bundleId, ...fields } = params as Record<string, any>;
					const sets: string[] = [];
					const values: SQLValue[] = [];
					const fieldMap: Record<string, string> = {
						label: "label", description: "description", tier: "tier",
						transformerModel: "transformer_model", vaeModel: "vae_model",
						steps: "steps", cfgScale: "cfg_scale", sampler: "sampler",
						scheduler: "scheduler", vramEstimateGb: "vram_estimate_gb",
						clipTokenizer: "clip_tokenizer", clipEncoder: "clip_encoder",
						t5Tokenizer: "t5_tokenizer", t5Encoder: "t5_encoder",
						t5EncoderConfig: "t5_encoder_config",
						qwen3Tokenizer: "qwen3_tokenizer", qwen3Encoder: "qwen3_encoder",
						source: "source", falEndpoint: "fal_endpoint",
						falAspectratio: "fal_aspectratio",
					};
					for (const [camel, col] of Object.entries(fieldMap)) {
						if (camel in fields) {
							sets.push(`${col} = ?`);
							values.push(fields[camel] as SQLValue);
						}
					}
					if (sets.length === 0) {
						return { status: "error", message: "No fields to update" };
					}
					values.push(bundleId as string);
					db.prepare(`UPDATE bundles SET ${sets.join(", ")} WHERE id = ?`).run(
						...values,
					);
					const bundle = db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId) as Row;
					return { status: "success", bundle };
				},
				deleteBundle: ({ bundleId }) => {
					db.prepare("DELETE FROM bundles WHERE id = ?").run(bundleId);
					return { status: "success" };
				},
				toggleBundleFavorite: ({ bundleId }) => {
					db.prepare(
						"UPDATE bundles SET favorite = CASE WHEN favorite = 0 THEN 1 ELSE 0 END WHERE id = ?",
					).run(bundleId);
					const bundle = db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId);
					return { status: "success", bundle: bundle as Record<string, unknown> };
				},
				toggleBundleHidden: ({ bundleId }) => {
					db.prepare(
						"UPDATE bundles SET hidden = CASE WHEN hidden = 0 THEN 1 ELSE 0 END WHERE id = ?",
					).run(bundleId);
					const bundle = db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId);
					return { status: "success", bundle: bundle as Record<string, unknown> };
				},

				// ── Gallery ──
				getGalleryImages: (params) => {
					const limit = params.limit ?? 50;
					const offset = params.offset ?? 0;
					const sortBy = params.sortBy ?? "created_at";
					const sortOrder = params.sortOrder === "asc" ? "ASC" : "DESC";

					let where = "WHERE 1=1";
					const bindValues: SQLValue[] = [];

					if (params.favoritesOnly) {
						where += " AND i.favorite = 1";
					}
					if (params.search) {
						where += " AND i.prompt LIKE ?";
						bindValues.push(`%${params.search}%`);
					}
					if (params.folderId) {
						where += " AND i.id IN (SELECT image_id FROM image_folders WHERE folder_id = ?)";
						bindValues.push(params.folderId);
					}
					if (params.tagId) {
						where += " AND i.id IN (SELECT image_id FROM image_tags WHERE tag_id = ?)";
						bindValues.push(params.tagId);
					}

					const countRow = db
						.prepare(`SELECT COUNT(*) as total FROM images i ${where}`)
						.get(...bindValues) as { total: number };

					const allowedSorts = ["created_at", "seed", "steps", "width", "height"];
					const safeSort = allowedSorts.includes(sortBy) ? sortBy : "created_at";

					const images = db
						.prepare(
							`SELECT * FROM images i ${where} ORDER BY ${safeSort} ${sortOrder} LIMIT ? OFFSET ?`,
						)
						.all(...bindValues, limit, offset) as Row[];

					return { status: "success", images, total: countRow.total };
				},
				getImage: ({ imageId }) => {
					const image = db
						.prepare("SELECT * FROM images WHERE id = ?")
						.get(imageId);
					if (!image) return { status: "error", message: "Image not found" };
					return { status: "success", image: image as Record<string, unknown> };
				},
				toggleFavorite: ({ imageId }) => {
					db.prepare(
						"UPDATE images SET favorite = CASE WHEN favorite = 0 THEN 1 ELSE 0 END WHERE id = ?",
					).run(imageId);
					const image = db.prepare("SELECT * FROM images WHERE id = ?").get(imageId);
					return { status: "success", image: image as Record<string, unknown> };
				},
				deleteImage: ({ imageId }) => {
					db.prepare("DELETE FROM images WHERE id = ?").run(imageId);
					return { status: "success" };
				},

				// ── Folders ──
				getFolders: () => {
					const folders = db
						.prepare("SELECT * FROM folders ORDER BY sort_order, name")
						.all() as Row[];
					return { status: "success", folders };
				},
				createFolder: (params) => {
					const now = Math.floor(Date.now() / 1000);
					db.prepare(
						"INSERT INTO folders (id, name, parent_id, color, icon, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
					).run(
						params.id, params.name, params.parentId ?? null,
						params.color ?? null, params.icon ?? null,
						params.sortOrder ?? 0, now, now,
					);
					const folder = db.prepare("SELECT * FROM folders WHERE id = ?").get(params.id);
					return { status: "success", folder: folder as Record<string, unknown> };
				},
				updateFolder: (params) => {
					const { folderId, ...fields } = params as Record<string, any>;
					const sets: string[] = [];
					const values: SQLValue[] = [];
					const fieldMap: Record<string, string> = {
						name: "name", parentId: "parent_id", color: "color",
						icon: "icon", sortOrder: "sort_order",
					};
					for (const [camel, col] of Object.entries(fieldMap)) {
						if (camel in fields) {
							sets.push(`${col} = ?`);
							values.push(fields[camel] as SQLValue);
						}
					}
					if (sets.length > 0) {
						sets.push("updated_at = ?");
						values.push(Math.floor(Date.now() / 1000));
						values.push(folderId as string);
						db.prepare(`UPDATE folders SET ${sets.join(", ")} WHERE id = ?`).run(
							...values,
						);
					}
					const folder = db.prepare("SELECT * FROM folders WHERE id = ?").get(folderId) as Row;
					return { status: "success", folder };
				},
				deleteFolder: ({ folderId }) => {
					db.prepare("DELETE FROM folders WHERE id = ?").run(folderId);
					return { status: "success" };
				},
				addImageToFolder: ({ imageId, folderId }) => {
					const now = Math.floor(Date.now() / 1000);
					db.prepare(
						"INSERT OR IGNORE INTO image_folders (image_id, folder_id, added_at) VALUES (?, ?, ?)",
					).run(imageId, folderId, now);
					return { status: "success" };
				},
				removeImageFromFolder: ({ imageId, folderId }) => {
					db.prepare(
						"DELETE FROM image_folders WHERE image_id = ? AND folder_id = ?",
					).run(imageId, folderId);
					return { status: "success" };
				},

				// ── Tags ──
				getTags: () => {
					const tags = db.prepare("SELECT * FROM tags ORDER BY name").all() as Row[];
					return { status: "success", tags };
				},
				createTag: ({ name, color, category }) => {
					db.prepare(
						"INSERT INTO tags (name, color, category) VALUES (?, ?, ?)",
					).run(name, color ?? null, category ?? null);
					const tag = db
						.prepare("SELECT * FROM tags WHERE name = ?")
						.get(name);
					return { status: "success", tag: tag as Record<string, unknown> };
				},
				updateTag: ({ tagId, name, color, category }) => {
					const sets: string[] = [];
					const values: SQLValue[] = [];
					if (name !== undefined) { sets.push("name = ?"); values.push(name); }
					if (color !== undefined) { sets.push("color = ?"); values.push(color ?? null); }
					if (category !== undefined) { sets.push("category = ?"); values.push(category ?? null); }
					if (sets.length > 0) {
						values.push(tagId);
						db.prepare(`UPDATE tags SET ${sets.join(", ")} WHERE id = ?`).run(...values);
					}
					const tag = db.prepare("SELECT * FROM tags WHERE id = ?").get(tagId) as Row;
					return { status: "success", tag };
				},
				deleteTag: ({ tagId }) => {
					db.prepare("DELETE FROM tags WHERE id = ?").run(tagId);
					return { status: "success" };
				},
				addTagToImage: ({ imageId, tagId }) => {
					db.prepare(
						"INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?, ?)",
					).run(imageId, tagId);
					return { status: "success" };
				},
				removeTagFromImage: ({ imageId, tagId }) => {
					db.prepare(
						"DELETE FROM image_tags WHERE image_id = ? AND tag_id = ?",
					).run(imageId, tagId);
					return { status: "success" };
				},
				getImageTags: ({ imageId }) => {
					const tags = db
						.prepare(
							"SELECT t.* FROM tags t JOIN image_tags it ON t.id = it.tag_id WHERE it.image_id = ?",
						)
						.all(imageId) as Row[];
					return { status: "success", tags };
				},

				// ── Inference (proxied to sidecar) ──
				getGpuInfo: async () => {
					if (!sidecar.ready) {
						return { status: "error", message: "Engine not ready" };
					}
					try {
						const info = await sidecar.fetchJson<{
							device_type: string;
							device_name: string | null;
							total_vram_gb: number | null;
						}>("/gpu-info");
						return {
							status: "success",
							deviceType: info.device_type,
							deviceName: info.device_name,
							totalVramGb: info.total_vram_gb ?? undefined,
						};
					} catch (err) {
						return { status: "error", message: String(err) };
					}
				},
				startEngine: async () => {
					try {
						await sidecar.start();
						return { status: "success" };
					} catch (err) {
						return { status: "error", message: String(err) };
					}
				},
				stopEngine: () => {
					sidecar.stop();
					return { status: "success" };
				},
				engineReady: () => {
					return { status: "success", ready: sidecar.ready };
				},
				submitJob: async (params) => {
					if (!sidecar.ready) {
						return { status: "error", message: "Engine not ready" };
					}
					try {
						const result = await sidecar.fetchJson<{
							job_id: string;
						}>("/jobs", {
							method: "POST",
							body: JSON.stringify(params),
						});
						return { status: "success", jobId: result.job_id };
					} catch (err) {
						return { status: "error", message: String(err) };
					}
				},
				cancelJob: async ({ jobId }) => {
					if (!sidecar.ready) {
						return { status: "error", message: "Engine not ready" };
					}
					try {
						await sidecar.fetch(`/jobs/${jobId}`, { method: "DELETE" });
						return { status: "success" };
					} catch (err) {
						return { status: "error", message: String(err) };
					}
				},

				// ── VRAM ──
				getVramUsage: async () => {
					if (!sidecar.ready) {
						return { status: "success", available: false };
					}
					try {
						const data = await sidecar.fetchJson<Record<string, unknown>>("/vram");
						return { status: "success", available: true, ...data };
					} catch {
						return { status: "success", available: false };
					}
				},
				clearVramCache: async () => {
					if (!sidecar.ready) {
						return { status: "error", message: "Engine not ready" };
					}
					try {
						await sidecar.fetch("/vram/clear", { method: "POST" });
						return { status: "success" };
					} catch (err) {
						return { status: "error", message: String(err) };
					}
				},
			},
			messages: {},
		},
	});

	return rpc;
}
