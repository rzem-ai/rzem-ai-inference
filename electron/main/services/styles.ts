/**
 * Styles service: CRUD operations for styles, style-lora, style-tag, and examples.
 * Migrated from src/bun/services/styles.ts — bun:sqlite → better-sqlite3.
 */

import type Database from "better-sqlite3";
import { readFileSync, readdirSync, existsSync } from "fs";
import { join, basename, extname } from "path";

type Row = Record<string, unknown>;
type SQLValue = string | number | null | bigint | boolean;

interface ApiResponse {
	status: "success" | "error";
	message?: string;
	[key: string]: unknown;
}

export function createStylesHandlers(db: Database.Database, stylesDir: string) {
	return {
		getStylesDir: (): ApiResponse => {
			return { status: "success", path: stylesDir };
		},

		listFilterThumbnails: (): ApiResponse => {
			const thumbDir = join(stylesDir, "thumbnails");
			if (!existsSync(thumbDir)) {
				return { status: "success", thumbnails: [] };
			}
			const stems = new Set<string>();
			for (const entry of readdirSync(thumbDir, { withFileTypes: true })) {
				if (!entry.isFile()) continue;
				const stem = basename(entry.name, extname(entry.name));
				if (stem.endsWith("_cover")) {
					stems.add(stem.slice(0, -6));
				} else if ([".png", ".jpg", ".jpeg", ".webp"].includes(extname(entry.name).toLowerCase())) {
					stems.add(stem);
				}
			}
			const thumbnails = [...stems].sort().map((s) => `${s}.png`);
			return { status: "success", thumbnails };
		},

		getFilterThumbnail: ({ imageName }: { imageName: string }): ApiResponse => {
			const thumbDir = join(stylesDir, "thumbnails");
			const stem = basename(imageName, extname(imageName));

			const coverPath = join(thumbDir, `${stem}_cover.webp`);
			if (existsSync(coverPath)) {
				const data = readFileSync(coverPath);
				const b64 = data.toString("base64");
				return { status: "success", dataUrl: `data:image/webp;base64,${b64}` };
			}

			const pngPath = join(thumbDir, `${stem}.png`);
			if (existsSync(pngPath)) {
				const data = readFileSync(pngPath);
				const b64 = data.toString("base64");
				return { status: "success", dataUrl: `data:image/png;base64,${b64}` };
			}

			return { status: "error", message: `Thumbnail not found: ${imageName}` };
		},

		getStyles: (params: {
			category?: string;
			tagId?: number;
			search?: string;
			favoritesOnly?: boolean;
			bundleType?: string;
			sortBy?: string;
			sortOrder?: string;
		}): ApiResponse => {
			const sortBy = params.sortBy ?? "updated_at";
			const sortOrder = params.sortOrder === "asc" ? "ASC" : "DESC";

			let where = "WHERE 1=1";
			const values: SQLValue[] = [];

			if (params.category) {
				where += " AND s.category = ?";
				values.push(params.category);
			}
			if (params.favoritesOnly) {
				where += " AND s.is_favorite = 1";
			}
			if (params.search) {
				where += " AND (s.name LIKE ? OR s.description LIKE ? OR s.prompt_template LIKE ?)";
				const term = `%${params.search}%`;
				values.push(term, term, term);
			}
			if (params.tagId) {
				where += " AND s.id IN (SELECT style_id FROM style_tags WHERE tag_id = ?)";
				values.push(params.tagId);
			}
			if (params.bundleType) {
				if (params.bundleType === "__unbound__") {
					where += " AND (s.bundle_id IS NULL OR s.bundle_id = '')";
				} else {
					where += " AND s.bundle_id IN (SELECT id FROM bundles WHERE transformer_type = ?)";
					values.push(params.bundleType);
				}
			}

			const allowedSorts = ["name", "category", "created_at", "updated_at", "usage_count"];
			const safeSort = allowedSorts.includes(sortBy) ? sortBy : "updated_at";

			const styles = db
				.prepare(`SELECT s.* FROM styles s ${where} ORDER BY s.${safeSort} ${sortOrder}`)
				.all(...values) as Row[];

			return { status: "success", styles };
		},

		getStyleBundleTypeCounts: (): ApiResponse => {
			const rows = db
				.prepare(`
					SELECT
						COALESCE(b.transformer_type, '__unbound__') as type_id,
						COUNT(*) as count
					FROM styles s
					LEFT JOIN bundles b ON s.bundle_id = b.id
					GROUP BY type_id
				`)
				.all() as Row[];
			return { status: "success", counts: rows };
		},

		getStyle: ({ styleId }: { styleId: string }): ApiResponse => {
			const style = db.prepare("SELECT * FROM styles WHERE id = ?").get(styleId) as Row | null;
			if (!style) return { status: "error", message: `Style '${styleId}' not found` };

			const loras = db
				.prepare(
					`SELECT sl.*, l.name, l.path, l.trigger_words
					 FROM style_loras sl JOIN loras l ON sl.lora_id = l.id
					 WHERE sl.style_id = ? ORDER BY sl.priority`,
				)
				.all(styleId) as Row[];

			const tags = db
				.prepare("SELECT t.* FROM tags t JOIN style_tags st ON t.id = st.tag_id WHERE st.style_id = ?")
				.all(styleId) as Row[];

			const examples = db
				.prepare("SELECT * FROM examples WHERE entity_type = 'style' AND entity_id = ? ORDER BY created_at")
				.all(styleId) as Row[];

			return { status: "success", style, loras, tags, examples };
		},

		createStyle: (params: {
			id: string;
			name: string;
			promptTemplate: string;
			description?: string;
			negativePrompt?: string;
			category?: string;
			thumbnailPath?: string;
			bundleId?: string;
		}): ApiResponse => {
			const now = Math.floor(Date.now() / 1000);
			db.prepare(`
				INSERT INTO styles (id, name, description, prompt_template, negative_prompt, category, thumbnail_path, bundle_id, created_at, updated_at)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
			`).run(
				params.id, params.name, params.description ?? null, params.promptTemplate,
				params.negativePrompt ?? null, params.category ?? null,
				params.thumbnailPath ?? null, params.bundleId ?? null, now, now,
			);
			const style = db.prepare("SELECT * FROM styles WHERE id = ?").get(params.id) as Row;
			return { status: "success", style };
		},

		updateStyle: (params: Record<string, unknown>): ApiResponse => {
			const { styleId, ...fields } = params as Record<string, any>;
			const sets: string[] = [];
			const values: SQLValue[] = [];
			const fieldMap: Record<string, string> = {
				name: "name", description: "description", promptTemplate: "prompt_template",
				negativePrompt: "negative_prompt", category: "category",
				thumbnailPath: "thumbnail_path", bundleId: "bundle_id",
			};

			for (const [camel, col] of Object.entries(fieldMap)) {
				if (camel in fields) {
					sets.push(`${col} = ?`);
					values.push(fields[camel] as SQLValue);
				}
			}

			if (sets.length === 0) return { status: "error", message: "No fields to update" };

			sets.push("updated_at = ?");
			values.push(Math.floor(Date.now() / 1000));
			values.push(styleId as string);

			db.prepare(`UPDATE styles SET ${sets.join(", ")} WHERE id = ?`).run(...values);
			const style = db.prepare("SELECT * FROM styles WHERE id = ?").get(styleId) as Row;
			return { status: "success", style };
		},

		deleteStyle: ({ styleId }: { styleId: string }): ApiResponse => {
			db.prepare("DELETE FROM styles WHERE id = ?").run(styleId);
			return { status: "success" };
		},

		toggleStyleFavorite: ({ styleId }: { styleId: string }): ApiResponse => {
			db.prepare(
				"UPDATE styles SET is_favorite = CASE WHEN is_favorite = 0 THEN 1 ELSE 0 END, updated_at = ? WHERE id = ?",
			).run(Math.floor(Date.now() / 1000), styleId);
			const style = db.prepare("SELECT * FROM styles WHERE id = ?").get(styleId) as Row;
			if (!style) return { status: "error", message: `Style '${styleId}' not found` };
			return { status: "success", style };
		},

		getStyleCategories: (): ApiResponse => {
			const rows = db
				.prepare("SELECT DISTINCT category FROM styles WHERE category IS NOT NULL ORDER BY category")
				.all() as Array<{ category: string }>;
			return { status: "success", categories: rows.map((r) => r.category) };
		},

		getStyleExamples: ({ styleId }: { styleId: string }): ApiResponse => {
			const examples = db
				.prepare("SELECT * FROM examples WHERE entity_type = 'style' AND entity_id = ? ORDER BY created_at")
				.all(styleId) as Row[];
			return { status: "success", examples };
		},

		createStyleExample: (params: {
			styleId: string;
			prompt: string;
			imagePath?: string;
			seed?: number;
			width?: number;
			height?: number;
			steps?: number;
			cfgScale?: number;
		}): ApiResponse => {
			const id = crypto.randomUUID();
			const now = Math.floor(Date.now() / 1000);
			const content = JSON.stringify({
				prompt: params.prompt, image_path: params.imagePath ?? null,
				seed: params.seed ?? null, width: params.width ?? null,
				height: params.height ?? null, steps: params.steps ?? null,
				cfg_scale: params.cfgScale ?? null,
			});

			db.prepare(
				"INSERT INTO examples (id, entity_type, entity_id, example_type, content, created_at) VALUES (?, 'style', ?, 'prompt', ?, ?)",
			).run(id, params.styleId, content, now);

			const example = db.prepare("SELECT * FROM examples WHERE id = ?").get(id) as Row;
			return { status: "success", example };
		},

		deleteStyleExample: ({ exampleId }: { exampleId: string }): ApiResponse => {
			db.prepare("DELETE FROM examples WHERE id = ?").run(exampleId);
			return { status: "success" };
		},

		getStyleLoras: ({ styleId }: { styleId: string }): ApiResponse => {
			const loras = db
				.prepare(
					`SELECT sl.*, l.name, l.path, l.trigger_words
					 FROM style_loras sl JOIN loras l ON sl.lora_id = l.id
					 WHERE sl.style_id = ? ORDER BY sl.priority`,
				)
				.all(styleId) as Row[];
			return { status: "success", loras };
		},

		setStyleLoras: ({
			styleId,
			loras,
		}: {
			styleId: string;
			loras: Array<{ loraId: string; strength: number; priority?: number }>;
		}): ApiResponse => {
			db.prepare("DELETE FROM style_loras WHERE style_id = ?").run(styleId);
			const insert = db.prepare(
				"INSERT INTO style_loras (style_id, lora_id, strength, priority) VALUES (?, ?, ?, ?)",
			);
			for (const lora of loras) {
				insert.run(styleId, lora.loraId, lora.strength, lora.priority ?? 0);
			}
			const result = db
				.prepare(
					`SELECT sl.*, l.name, l.path, l.trigger_words
					 FROM style_loras sl JOIN loras l ON sl.lora_id = l.id
					 WHERE sl.style_id = ? ORDER BY sl.priority`,
				)
				.all(styleId) as Row[];
			return { status: "success", loras: result };
		},

		getStyleTags: ({ styleId }: { styleId: string }): ApiResponse => {
			const tags = db
				.prepare("SELECT t.* FROM tags t JOIN style_tags st ON t.id = st.tag_id WHERE st.style_id = ?")
				.all(styleId) as Row[];
			return { status: "success", tags };
		},

		setStyleTags: ({ styleId, tagIds }: { styleId: string; tagIds: number[] }): ApiResponse => {
			db.prepare("DELETE FROM style_tags WHERE style_id = ?").run(styleId);
			const insert = db.prepare("INSERT OR IGNORE INTO style_tags (style_id, tag_id) VALUES (?, ?)");
			for (const tagId of tagIds) {
				insert.run(styleId, tagId);
			}
			const tags = db
				.prepare("SELECT t.* FROM tags t JOIN style_tags st ON t.id = st.tag_id WHERE st.style_id = ?")
				.all(styleId) as Row[];
			return { status: "success", tags };
		},

		getLoras: (): ApiResponse => {
			const loras = db.prepare("SELECT * FROM loras ORDER BY name").all() as Row[];
			return { status: "success", loras };
		},

		createLora: (params: {
			id: string;
			name: string;
			path: string;
			triggerWords?: string;
			baseModel?: string;
			sizeBytes?: number;
			strength?: number;
		}): ApiResponse => {
			const now = Math.floor(Date.now() / 1000);
			db.prepare(`
				INSERT INTO loras (id, name, path, trigger_words, base_model, size_bytes, strength, created_at)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?)
			`).run(
				params.id, params.name, params.path,
				params.triggerWords ?? null, params.baseModel ?? null,
				params.sizeBytes ?? null, params.strength ?? 1.0, now,
			);
			const lora = db.prepare("SELECT * FROM loras WHERE id = ?").get(params.id) as Row;
			return { status: "success", lora };
		},
	};
}
