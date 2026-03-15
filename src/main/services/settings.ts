/**
 * Settings service: engine status, VRAM, HF cache, data paths, disk usage.
 * Migrated from src/bun/services/settings.ts — electrobun → electron.
 */

import type Database from "better-sqlite3";
import type { SidecarManager } from "../sidecar";
import { statSync, readdirSync } from "fs";
import { join } from "path";
import { app } from "electron";

type Row = Record<string, unknown>;

interface ApiResponse {
	status: "success" | "error";
	message?: string;
	[key: string]: unknown;
}

function dirSizeBytes(dirPath: string): number {
	let total = 0;
	try {
		for (const entry of readdirSync(dirPath, { withFileTypes: true })) {
			const fullPath = join(dirPath, entry.name);
			if (entry.isFile()) {
				total += statSync(fullPath).size;
			} else if (entry.isDirectory()) {
				total += dirSizeBytes(fullPath);
			}
		}
	} catch {
		// Permission denied or not found
	}
	return total;
}

export function createSettingsHandlers(
	db: Database.Database,
	sidecar: SidecarManager,
	outputDir: string,
) {
	const dataDir = app.getPath("userData");

	return {
		getEngineStatus: async (): Promise<ApiResponse> => {
			if (!sidecar.ready) {
				return { status: "success", ready: false };
			}
			try {
				const health = await sidecar.fetchJson<{
					status: string;
					device: string;
					jobs_queued: number;
					jobs_running: number;
					jobs_completed: number;
				}>("/health");
				return {
					status: "success",
					ready: true,
					completedCount: health.jobs_completed,
				};
			} catch {
				return { status: "success", ready: false };
			}
		},

		getCudaVersion: async (): Promise<ApiResponse> => {
			if (!sidecar.ready) {
				return { status: "success", cudaVersion: null };
			}
			try {
				const data = await sidecar.fetchJson<{ cuda_version: string | null }>("/cuda-version");
				return { status: "success", cudaVersion: data.cuda_version };
			} catch {
				return { status: "success", cudaVersion: null };
			}
		},

		resetEngine: async (): Promise<ApiResponse> => {
			try {
				await sidecar.restart();
				return { status: "success" };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		getCacheInfo: async (): Promise<ApiResponse> => {
			if (!sidecar.ready) {
				return { status: "success", models: [], totalSize: 0 };
			}
			try {
				const models = await sidecar.fetchJson<
					Array<{
						repo_id: string;
						size_on_disk: number;
						nb_files: number;
						last_modified: number;
					}>
				>("/models");
				const totalSize = models.reduce((sum, m) => sum + m.size_on_disk, 0);
				return { status: "success", models, totalSize };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		deleteCachedModel: async ({ repoId }: { repoId: string }): Promise<ApiResponse> => {
			if (!sidecar.ready) {
				return { status: "error", message: "Engine not ready" };
			}
			try {
				await sidecar.fetch(`/models/${encodeURIComponent(repoId)}`, { method: "DELETE" });
				return { status: "success" };
			} catch (err) {
				return { status: "error", message: String(err) };
			}
		},

		getDataPaths: (): ApiResponse => {
			const hfCacheDir =
				process.env.HF_HOME ??
				join(process.env.HOME ?? "/tmp", ".cache", "huggingface", "hub");
			return { status: "success", dataDir, outputDir, hfCacheDir };
		},

		getDiskUsage: (): ApiResponse => {
			const outputSize = dirSizeBytes(outputDir);
			const hfCacheDir =
				process.env.HF_HOME ??
				join(process.env.HOME ?? "/tmp", ".cache", "huggingface", "hub");
			const hfCacheSize = dirSizeBytes(hfCacheDir);
			return { status: "success", outputSize, hfCacheSize };
		},

		getSslVerificationDisabled: (): ApiResponse => {
			const row = db
				.prepare("SELECT value FROM settings WHERE key = 'DISABLE_SSL_VERIFICATION'")
				.get() as { value: string } | null;
			return { status: "success", disabled: row?.value === "1" };
		},

		setSslVerificationDisabled: ({ disabled }: { disabled: boolean }): ApiResponse => {
			db.prepare(
				"INSERT INTO settings (key, value) VALUES ('DISABLE_SSL_VERIFICATION', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
			).run(disabled ? "1" : "0");
			return { status: "success", disabled };
		},

		completeSetup: ({
			generationMode,
			apiKeys,
		}: {
			generationMode: string;
			apiKeys?: Record<string, string>;
		}): ApiResponse => {
			db.prepare(
				"INSERT INTO settings (key, value) VALUES ('SETUP_COMPLETED', '1') ON CONFLICT(key) DO UPDATE SET value = excluded.value",
			).run();
			db.prepare(
				"INSERT INTO settings (key, value) VALUES ('GENERATION_MODE', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
			).run(generationMode);

			if (apiKeys) {
				const upsert = db.prepare(
					"INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
				);
				for (const [key, value] of Object.entries(apiKeys)) {
					if (value) upsert.run(key, value);
				}
			}
			return { status: "success" };
		},
	};
}
