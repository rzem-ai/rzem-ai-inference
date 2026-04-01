/**
 * FAL cloud image generation service.
 * Calls the FAL API directly via @fal-ai/client, bypassing the local inference engine.
 */

import { fal } from "@fal-ai/client";
import { createWriteStream, existsSync } from "fs";
import { join } from "path";
import { pipeline } from "stream/promises";
import axios from "axios";

// ── Aspect ratio mapping ─────────────────────────────────────────

interface AspectDimensions {
	width: number;
	height: number;
}

const ASPECT_DIMENSIONS: Record<string, AspectDimensions> = {
	square_hd: { width: 1024, height: 1024 },
	square: { width: 512, height: 512 },
	portrait_4_3: { width: 768, height: 1024 },
	portrait_16_9: { width: 576, height: 1024 },
	landscape_4_3: { width: 1024, height: 768 },
	landscape_16_9: { width: 1024, height: 576 },
};

/**
 * Find the closest FAL aspect ratio string for given pixel dimensions.
 * Falls back to explicit {width}x{height} if no close match.
 */
function closestAspectRatio(
	width: number,
	height: number,
	allowed: string[],
): string | { width: number; height: number } {
	const targetRatio = width / height;
	let best: string | null = null;
	let bestDiff = Infinity;

	for (const name of allowed) {
		const dims = ASPECT_DIMENSIONS[name];
		if (!dims) {
			// Handle raw ratio strings like "16:9"
			const parts = name.split(":");
			if (parts.length === 2) {
				const a = parseFloat(parts[0]!);
				const b = parseFloat(parts[1]!);
				const r = a / b;
				const diff = Math.abs(r - targetRatio);
				if (diff < bestDiff) {
					bestDiff = diff;
					best = name;
				}
			}
			continue;
		}
		const diff = Math.abs(dims.width / dims.height - targetRatio);
		if (diff < bestDiff) {
			bestDiff = diff;
			best = name;
		}
	}

	if (best && bestDiff < 0.1) return best;
	return { width, height };
}

// ── Types ────────────────────────────────────────────────────────

interface FalJobParams {
	prompt: string;
	width: number;
	height: number;
	steps?: number;
	cfg_scale?: number;
	seed?: number;
	negative_prompt?: string;
	fal_endpoint: string;
	fal_aspectratio?: string[];
	num_images?: number;
}

interface FalResultImage {
	url: string;
	width: number;
	height: number;
	content_type?: string;
}

interface FalResult {
	images: FalResultImage[];
	seed?: number;
	prompt?: string;
}

type EventCallback = (event: Record<string, unknown>) => void;

// ── Service ──────────────────────────────────────────────────────

export interface FalServiceConfig {
	outputDir: string;
}

export function createFalService(config: FalServiceConfig) {
	const eventListeners: EventCallback[] = [];

	function onEvent(listener: EventCallback): () => void {
		eventListeners.push(listener);
		return () => {
			const idx = eventListeners.indexOf(listener);
			if (idx >= 0) eventListeners.splice(idx, 1);
		};
	}

	function emit(event: Record<string, unknown>) {
		for (const listener of eventListeners) {
			listener(event);
		}
	}

	async function submitJob(
		jobId: string,
		apiKey: string,
		params: FalJobParams,
	): Promise<void> {
		const startTime = performance.now();

		try {
			// Configure FAL client with the API key
			fal.config({ credentials: apiKey });

			emit({
				event: "job_started",
				job_id: jobId,
				data: { job_id: jobId, width: params.width, height: params.height },
			});

			// Build the image size / aspect ratio input
			const allowed = params.fal_aspectratio ?? [];
			let imageSize: string | { width: number; height: number };
			if (allowed.length > 0) {
				imageSize = closestAspectRatio(params.width, params.height, allowed);
			} else {
				imageSize = { width: params.width, height: params.height };
			}

			// Build FAL input — use aspect_ratio for ratio strings (e.g. "16:9"),
			// image_size for named presets (e.g. "landscape_16_9") or explicit dimensions
			const input: Record<string, unknown> = {
				prompt: params.prompt,
				num_images: params.num_images ?? 1,
			};
			if (typeof imageSize === "string" && imageSize.includes(":")) {
				input.aspect_ratio = imageSize;
			} else {
				input.image_size = imageSize;
			}

			if (params.steps && params.steps > 0) {
				input.num_inference_steps = params.steps;
			}
			if (params.cfg_scale && params.cfg_scale > 0) {
				input.guidance_scale = params.cfg_scale;
			}
			if (params.seed !== undefined && params.seed >= 0) {
				input.seed = params.seed;
			}
			if (params.negative_prompt) {
				input.negative_prompt = params.negative_prompt;
			}

			emit({
				event: "job_progress",
				job_id: jobId,
				data: { job_id: jobId, step: 0, total_steps: 1, width: params.width, height: params.height },
			});

			// Call the FAL API
			const result = await fal.run(params.fal_endpoint, { input }) as { data: FalResult };
			const data = result.data;

			if (!data.images?.length) {
				throw new Error("FAL API returned no images");
			}

			const firstImage = data.images[0]!;
			const imageUrl = firstImage.url;

			// Download the image to outputDir
			const imagePath = join(config.outputDir, `${jobId}.png`);
			const response = await axios.get(imageUrl, { responseType: "stream" });
			const writer = createWriteStream(imagePath);
			await pipeline(response.data, writer);

			// Check if we need to convert to PNG (if the result is not PNG)
			const contentType = firstImage.content_type ?? response.headers["content-type"] ?? "";
			if (contentType && !contentType.includes("png")) {
				try {
					const sharp = (await import("sharp")).default;
					const pngPath = imagePath + ".tmp.png";
					await sharp(imagePath).png().toFile(pngPath);
					const { renameSync } = await import("fs");
					renameSync(pngPath, imagePath);
				} catch {
					// If sharp conversion fails, keep the original file
				}
			}

			const genTimeMs = Math.round(performance.now() - startTime);

			emit({
				event: "job_completed",
				job_id: jobId,
				data: {
					job_id: jobId,
					seed: data.seed ?? -1,
					image_path: imagePath,
					width: firstImage.width ?? params.width,
					height: firstImage.height ?? params.height,
					generation_time_ms: genTimeMs,
				},
			});
		} catch (err) {
			const message = err instanceof Error ? err.message : String(err);
			console.error(`[fal] Job ${jobId} failed:`, message);

			emit({
				event: "job_failed",
				job_id: jobId,
				data: { job_id: jobId, error: message },
			});
		}
	}

	return {
		submitJob,
		onEvent,
	};
}

export type FalService = ReturnType<typeof createFalService>;
