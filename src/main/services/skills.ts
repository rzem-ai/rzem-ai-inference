/**
 * Skills service: loads markdown skill files from resources/skills/.
 *
 * Each skill is a .md file with YAML-ish frontmatter:
 *   ---
 *   name: flux-prompting
 *   description: How to write prompts for FLUX.1/2 models
 *   when_to_use: When the user is on a FLUX bundle and asks about prompt wording
 *   model_families: [flux1_dev, flux1_kontext, flux2_dev]
 *   tags: [prompting]
 *   ---
 *   <markdown body...>
 *
 * The chat agent receives the skill index in its system prompt and fetches
 * full content on demand via the read_skill tool.
 */

import { app } from "electron";
import { join } from "path";
import { readdirSync, readFileSync } from "fs";

export interface SkillMetadata {
	name: string;
	description: string;
	when_to_use: string;
	model_families?: string[];
	tags?: string[];
}

interface ParsedSkill {
	metadata: SkillMetadata;
	raw: string;
}

const FRONTMATTER_RE = /^---\r?\n([\s\S]*?)\r?\n---\r?\n([\s\S]*)$/;

function parseFrontmatter(raw: string, filename: string): ParsedSkill {
	const match = raw.match(FRONTMATTER_RE);
	if (!match) {
		throw new Error(`[skills] ${filename}: missing or malformed frontmatter`);
	}
	const fmBlock = match[1] ?? "";
	const meta: Record<string, unknown> = {};

	for (const line of fmBlock.split(/\r?\n/)) {
		const m = line.match(/^(\w+):\s*(.*)$/);
		if (!m) continue;
		const key = m[1]!;
		const value = (m[2] ?? "").trim();
		if (value.startsWith("[") && value.endsWith("]")) {
			meta[key] = value
				.slice(1, -1)
				.split(",")
				.map((s) => s.trim().replace(/^["']|["']$/g, ""))
				.filter(Boolean);
		} else {
			meta[key] = value.replace(/^["']|["']$/g, "");
		}
	}

	if (typeof meta.name !== "string" || !meta.name) {
		throw new Error(`[skills] ${filename}: required field 'name' missing`);
	}
	if (typeof meta.description !== "string" || !meta.description) {
		throw new Error(`[skills] ${filename}: required field 'description' missing`);
	}
	if (typeof meta.when_to_use !== "string" || !meta.when_to_use) {
		throw new Error(`[skills] ${filename}: required field 'when_to_use' missing`);
	}

	const stem = filename.replace(/\.md$/, "");
	if (meta.name !== stem) {
		console.warn(`[skills] ${filename}: 'name' field "${meta.name}" does not match filename stem "${stem}" — using filename`);
		meta.name = stem;
	}

	return {
		metadata: meta as unknown as SkillMetadata,
		raw,
	};
}

export function createSkillsService() {
	let cache: Map<string, ParsedSkill> | null = null;
	const skillsDir = join(app.getAppPath(), "resources", "skills");
	const reloadEveryRequest = process.env.RZEM_SKILLS_RELOAD === "1";

	function load(): Map<string, ParsedSkill> {
		const result = new Map<string, ParsedSkill>();
		let files: string[];
		try {
			files = readdirSync(skillsDir).filter((f) => f.endsWith(".md")).sort();
		} catch (err) {
			console.error(`[skills] failed to read ${skillsDir}:`, err);
			return result;
		}

		for (const filename of files) {
			try {
				const raw = readFileSync(join(skillsDir, filename), "utf-8");
				const parsed = parseFrontmatter(raw, filename);
				result.set(parsed.metadata.name, parsed);
			} catch (err) {
				console.error(err instanceof Error ? err.message : err);
			}
		}

		console.log(`[skills] loaded ${result.size} skills from ${skillsDir}`);
		return result;
	}

	function ensureLoaded(): Map<string, ParsedSkill> {
		if (!cache || reloadEveryRequest) {
			cache = load();
		}
		return cache;
	}

	return {
		listSkills(filter?: { modelFamily?: string }): SkillMetadata[] {
			const all = Array.from(ensureLoaded().values()).map((s) => s.metadata);
			const family = filter?.modelFamily;
			if (!family) return all;
			return all.filter((m) => !m.model_families || m.model_families.includes(family));
		},

		readSkill(name: string): string | null {
			return ensureLoaded().get(name)?.raw ?? null;
		},

		reload() {
			cache = null;
		},
	};
}

export type SkillsService = ReturnType<typeof createSkillsService>;
