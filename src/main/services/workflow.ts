/**
 * Workflow execution engine — parses node graphs, runs nodes sequentially.
 * Migrated from src/bun/services/workflow.ts — bun:sqlite → better-sqlite3.
 */

import type Database from "better-sqlite3";
import type { SidecarManager } from "../sidecar";
import type { FalService } from "./fal";
import { readFileSync, existsSync } from "fs";
import { join } from "path";

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

type Row = Record<string, unknown>;

interface WorkflowEvent {
	type: string;
	[key: string]: unknown;
}

interface NodeData {
	id: string;
	type: string;
	data: Record<string, unknown>;
}

interface Edge {
	source: string;
	target: string;
	sourceHandle?: string;
	targetHandle?: string;
}

type ProgressCallback = (progress: number, message: string) => void;
type CancelCheck = () => boolean;

interface NodeExecutor {
	execute(
		data: Record<string, unknown>,
		inputs: Record<string, unknown>,
		progressCallback: ProgressCallback,
		cancelCheck: CancelCheck,
	): Promise<Record<string, unknown>>;
}

class ImageInputExecutor implements NodeExecutor {
	async execute(data: Record<string, unknown>, _inputs: Record<string, unknown>, progressCallback: ProgressCallback): Promise<Record<string, unknown>> {
		const imagePath = (data.imagePath as string) || "";
		if (!imagePath) throw new Error("No image path specified in ImageInput node");
		if (!existsSync(imagePath)) throw new Error(`Image file not found: ${imagePath}`);
		progressCallback(1.0, "Image loaded");
		return { image: imagePath };
	}
}

class VisionQAExecutor implements NodeExecutor {
	constructor(private chatService: { completeWithVision(messages: unknown[], maxTokens?: number): Promise<string> }) {}

	async execute(data: Record<string, unknown>, inputs: Record<string, unknown>, progressCallback: ProgressCallback): Promise<Record<string, unknown>> {
		let question = (data.question as string) || "";
		if (!question) throw new Error("No question specified in VisionQA node");

		const imagePath = (inputs.image as string) || "";
		if (!imagePath) throw new Error("No image input connected to VisionQA node");

		for (const [key, value] of Object.entries(inputs)) {
			if (typeof value === "string" && key !== "image") {
				question = question.replaceAll(`{${key}}`, value);
			}
		}

		progressCallback(0.2, "Sending to vision model");

		const imageData = readFileSync(imagePath);
		const base64 = imageData.toString("base64");
		const ext = imagePath.split(".").pop()?.toLowerCase() || "png";
		const mediaType = ext === "jpg" ? "image/jpeg" : `image/${ext}`;

		const messages = [{
			role: "user",
			content: [
				{ type: "image", source: { type: "base64", media_type: mediaType, data: base64 } },
				{ type: "text", text: question },
			],
		}];

		const maxTokens = (data.maxTokens as number) || 1024;
		const answer = await this.chatService.completeWithVision(messages, maxTokens);

		progressCallback(1.0, "Vision analysis complete");
		return { text: answer };
	}
}

class ImageGenExecutor implements NodeExecutor {
	constructor(
		private sidecar: SidecarManager,
		private db: Database.Database,
		private outputDir: string,
		private falService: FalService,
	) {}

	async execute(data: Record<string, unknown>, inputs: Record<string, unknown>, progressCallback: ProgressCallback, cancelCheck: CancelCheck): Promise<Record<string, unknown>> {
		let prompt = (data.prompt as string) || "";

		if (inputs.text) {
			if (prompt) {
				prompt = prompt.replaceAll("{input}", inputs.text as string);
			} else {
				prompt = inputs.text as string;
			}
		}
		if (!prompt) throw new Error("No prompt specified for ImageGen node");

		const bundleId = data.bundleId as string;
		if (!bundleId) throw new Error("No model bundle selected for Image Gen node. Please select a bundle in the node properties.");

		const bundle = this.db.prepare("SELECT * FROM bundles WHERE id = ?").get(bundleId) as Row | null;
		if (!bundle) throw new Error(`Bundle not found: ${bundleId}`);

		const transformerType = (bundle.transformer_type as string) || (data.transformerType as string) || "flux_dev";

		// ── FAL cloud route ──
		if (transformerType === "fal_cloud") {
			return this.executeCloud(prompt, bundle, data, progressCallback, cancelCheck);
		}

		// ── Local sidecar route ──
		return this.executeLocal(prompt, bundle, bundleId, transformerType, data, progressCallback, cancelCheck);
	}

	private async executeCloud(
		prompt: string,
		bundle: Row,
		data: Record<string, unknown>,
		progressCallback: ProgressCallback,
		cancelCheck: CancelCheck,
	): Promise<Record<string, unknown>> {
		const falKey = (this.db.prepare("SELECT value FROM settings WHERE key = 'FAL_KEY'").get() as { value: string } | null)?.value;
		if (!falKey) throw new Error("FAL API key not configured. Set it in Settings.");

		const falEndpoint = (bundle.fal_endpoint as string) || "";
		if (!falEndpoint) throw new Error("No FAL endpoint configured for this bundle");

		let falAspectratio = bundle.fal_aspectratio;
		if (typeof falAspectratio === "string") {
			try { falAspectratio = JSON.parse(falAspectratio); } catch { falAspectratio = undefined; }
		}

		progressCallback(0.1, "Submitting to FAL cloud");

		const jobId = crypto.randomUUID();

		// Wait for the FAL job to complete via a Promise
		const result = await new Promise<Record<string, unknown>>((resolve, reject) => {
			const unsubscribe = this.falService.onEvent((event) => {
				const eventJobId = event.job_id as string;
				if (eventJobId !== jobId) return;

				const eventType = event.event as string;
				if (eventType === "job_completed") {
					unsubscribe();
					const eventData = event.data as Record<string, unknown>;
					resolve({ image: eventData.image_path as string });
				} else if (eventType === "job_failed") {
					unsubscribe();
					const eventData = event.data as Record<string, unknown>;
					reject(new Error(`FAL generation failed: ${eventData.error ?? "Unknown error"}`));
				} else if (eventType === "job_progress") {
					progressCallback(0.5, "Generating on FAL cloud...");
				}
			});

			this.falService.submitJob(jobId, falKey, {
				prompt,
				width: (data.width as number) ?? 1024,
				height: (data.height as number) ?? 1024,
				steps: (data.steps as number) ?? (bundle.steps as number) ?? undefined,
				cfg_scale: (data.cfg as number) ?? (bundle.cfg_scale as number) ?? undefined,
				seed: (data.seed as number) ?? -1,
				fal_endpoint: falEndpoint,
				fal_aspectratio: falAspectratio as string[] | undefined,
			});
		});

		progressCallback(1.0, "Cloud generation complete");
		return result;
	}

	private async executeLocal(
		prompt: string,
		bundle: Row,
		bundleId: string,
		transformerType: string,
		data: Record<string, unknown>,
		progressCallback: ProgressCallback,
		cancelCheck: CancelCheck,
	): Promise<Record<string, unknown>> {
		const jobParams: Record<string, unknown> = {
			prompt,
			transformer_model: bundle.transformer_model || data.transformerModel || "",
			transformer_type: transformerType,
			vae_model: bundle.vae_model || data.vaeModel || "",
			steps: data.steps ?? bundle.steps ?? 20,
			cfg_scale: data.cfg ?? bundle.cfg_scale ?? 1.0,
			width: data.width ?? 1024,
			height: data.height ?? 1024,
			seed: data.seed ?? -1,
			sampler: data.sampler ?? bundle.sampler ?? "euler",
			scheduler: data.scheduler ?? bundle.scheduler ?? "normal",
		};

		progressCallback(0.1, "Submitting generation job");

		const submitRes = await this.sidecar.fetchJson<{ job_id: string }>("/jobs", {
			method: "POST",
			body: JSON.stringify(keysToSnake({ ...jobParams, bundle_id: bundleId })),
		});
		const jobId = submitRes.job_id;

		progressCallback(0.2, `Job submitted: ${jobId.slice(0, 8)}`);

		while (true) {
			if (cancelCheck()) {
				try { await this.sidecar.fetch(`/jobs/${jobId}`, { method: "DELETE" }); } catch { /* ignore */ }
				throw new Error("Workflow cancelled");
			}

			const status = await this.sidecar.fetchJson<{
				status: string;
				progress?: { step: number; total_steps: number };
				result?: { seed: number; image_url: string };
				error?: { error: string } | unknown;
			}>(`/jobs/${jobId}`);

			if (status.status === "completed") {
				let imagePath = join(this.outputDir, `${jobId}.png`);
				if (!existsSync(imagePath)) {
					const { readdirSync } = require("fs") as typeof import("fs");
					const match = readdirSync(this.outputDir).find((f: string) => f.endsWith(`_${jobId}.png`));
					if (match) {
						imagePath = join(this.outputDir, match);
					} else {
						throw new Error(`Generation completed but image file not found at: ${imagePath}`);
					}
				}
				progressCallback(1.0, "Generation complete");
				return { image: imagePath };
			} else if (status.status === "failed") {
				const errObj = status.error as { error?: string } | string | null;
				const errMsg = typeof errObj === "string" ? errObj
					: errObj && typeof errObj === "object" && errObj.error ? errObj.error
					: errObj ? JSON.stringify(errObj) : "Unknown error";
				throw new Error(`Image generation failed: ${errMsg}`);
			} else if (status.status === "running" && status.progress) {
				const { step, total_steps } = status.progress;
				const pct = 0.2 + 0.7 * (step / Math.max(total_steps, 1));
				progressCallback(pct, `Generating step ${step}/${total_steps}`);
			}

			await new Promise((r) => setTimeout(r, 300));
		}
	}
}

class TextExecutor implements NodeExecutor {
	async execute(data: Record<string, unknown>, inputs: Record<string, unknown>, progressCallback: ProgressCallback): Promise<Record<string, unknown>> {
		let text = (data.content as string) || (data.text as string) || "";
		for (const [key, value] of Object.entries(inputs)) {
			if (typeof value === "string") {
				text = text.replaceAll(`{${key}}`, value);
			}
		}
		if (typeof inputs.text === "string") {
			text = text.replaceAll("{input}", inputs.text);
		}
		progressCallback(1.0, "Text ready");
		return { text };
	}
}

class OutputExecutor implements NodeExecutor {
	async execute(_data: Record<string, unknown>, inputs: Record<string, unknown>, progressCallback: ProgressCallback): Promise<Record<string, unknown>> {
		progressCallback(1.0, "Output collected");
		return { ...inputs };
	}
}

function topologicalSort(nodes: NodeData[], edges: Edge[]): NodeData[] {
	const nodeMap = new Map(nodes.map((n) => [n.id, n]));
	const inDegree = new Map(nodes.map((n) => [n.id, 0]));
	const adjacency = new Map<string, string[]>(nodes.map((n) => [n.id, []]));

	for (const edge of edges) {
		if (nodeMap.has(edge.source) && nodeMap.has(edge.target)) {
			adjacency.get(edge.source)!.push(edge.target);
			inDegree.set(edge.target, (inDegree.get(edge.target) || 0) + 1);
		}
	}

	const queue: string[] = [];
	for (const [nid, deg] of inDegree) {
		if (deg === 0) queue.push(nid);
	}

	const sortedIds: string[] = [];
	while (queue.length > 0) {
		const nid = queue.shift()!;
		sortedIds.push(nid);
		for (const neighbor of adjacency.get(nid) || []) {
			const newDeg = (inDegree.get(neighbor) || 1) - 1;
			inDegree.set(neighbor, newDeg);
			if (newDeg === 0) queue.push(neighbor);
		}
	}

	if (sortedIds.length !== nodes.length) {
		throw new Error("Workflow graph contains a cycle");
	}

	return sortedIds.map((id) => nodeMap.get(id)!);
}

function buildInputMap(
	nodeId: string,
	edges: Edge[],
	outputs: Map<string, Record<string, unknown>>,
): Record<string, unknown> {
	const inputs: Record<string, unknown> = {};
	for (const edge of edges) {
		if (edge.target === nodeId) {
			const srcOutputs = outputs.get(edge.source) || {};
			const srcHandle = edge.sourceHandle || "";
			const tgtHandle = edge.targetHandle || "";
			if (srcHandle && srcHandle in srcOutputs) {
				const key = tgtHandle || srcHandle;
				inputs[key] = srcOutputs[srcHandle];
			} else if (Object.keys(srcOutputs).length > 0) {
				const firstKey = Object.keys(srcOutputs)[0]!;
				const key = tgtHandle || firstKey;
				inputs[key] = srcOutputs[firstKey];
			}
		}
	}
	return inputs;
}

export interface WorkflowServiceDeps {
	db: Database.Database;
	sidecar: SidecarManager;
	outputDir: string;
	chatService: {
		completeWithVision(messages: unknown[], maxTokens?: number): Promise<string>;
	};
	falService: FalService;
}

export function createWorkflowService(deps: WorkflowServiceDeps) {
	const { db, sidecar, outputDir, chatService, falService } = deps;

	const events: WorkflowEvent[] = [];
	const activeRuns = new Map<string, { cancelled: boolean }>();

	const executors: Record<string, NodeExecutor> = {
		image_input: new ImageInputExecutor(),
		vision_qa: new VisionQAExecutor(chatService),
		image_gen: new ImageGenExecutor(sidecar, db, outputDir, falService),
		text: new TextExecutor(),
		output: new OutputExecutor(),
	};

	function pushEvent(type: string, data: Record<string, unknown>) {
		events.push({ type, ...data });
		if (events.length > 500) events.splice(0, events.length - 500);
	}

	function pollEvents(): WorkflowEvent[] {
		return events.splice(0, events.length);
	}

	async function executeGraph(runId: string, graphJson: string, run: { cancelled: boolean }) {
		try {
			const graph = JSON.parse(graphJson);
			const nodes: NodeData[] = graph.nodes || [];
			const edges: Edge[] = graph.edges || [];

			if (nodes.length === 0) throw new Error("Workflow has no nodes");

			const sortedNodes = topologicalSort(nodes, edges);

			pushEvent("workflow_started", { run_id: runId, total_nodes: sortedNodes.length });

			const nodeOutputs = new Map<string, Record<string, unknown>>();

			for (let i = 0; i < sortedNodes.length; i++) {
				if (run.cancelled) {
					pushEvent("workflow_failed", { run_id: runId, error: "Workflow cancelled" });
					return;
				}

				const node = sortedNodes[i]!;
				const nodeType = node.type || "";
				const nodeData = node.data || {};
				const nodeLabel = (nodeData.label as string) || nodeType;

				const executor = executors[nodeType];
				if (!executor) throw new Error(`Unknown node type: ${nodeType}`);

				pushEvent("node_started", {
					run_id: runId, node_id: node.id, node_type: nodeType,
					node_label: nodeLabel, node_index: i, total_nodes: sortedNodes.length,
				});

				const inputs = buildInputMap(node.id, edges, nodeOutputs);
				const progressCallback: ProgressCallback = (progress, message) => {
					pushEvent("node_progress", { run_id: runId, node_id: node.id, progress, message });
				};
				const cancelCheck: CancelCheck = () => run.cancelled;

				try {
					const outputs = await executor.execute(nodeData, inputs, progressCallback, cancelCheck);
					nodeOutputs.set(node.id, outputs);
					pushEvent("node_completed", { run_id: runId, node_id: node.id, node_type: nodeType, outputs });
				} catch (err) {
					const error = String(err instanceof Error ? err.message : err);
					console.error(`Workflow ${runId}: node ${node.id} (${nodeType}) failed:`, error);
					pushEvent("node_failed", { run_id: runId, node_id: node.id, node_type: nodeType, error });
					pushEvent("workflow_failed", { run_id: runId, error: `Node '${nodeLabel}' failed: ${error}` });
					return;
				}
			}

			const finalOutputs: Record<string, Record<string, unknown>> = {};
			for (const node of sortedNodes) {
				if (node.type === "output") {
					finalOutputs[node.id] = nodeOutputs.get(node.id) || {};
				}
			}

			pushEvent("workflow_completed", { run_id: runId, outputs: finalOutputs });
		} catch (err) {
			const error = String(err instanceof Error ? err.message : err);
			console.error(`Workflow ${runId} failed:`, error);
			pushEvent("workflow_failed", { run_id: runId, error });
		} finally {
			activeRuns.delete(runId);
		}
	}

	return {
		runWorkflow(graphJson: string): string {
			const runId = crypto.randomUUID();
			const run = { cancelled: false };
			activeRuns.set(runId, run);
			executeGraph(runId, graphJson, run);
			return runId;
		},

		cancelWorkflow(runId: string): boolean {
			const run = activeRuns.get(runId);
			if (!run) return false;
			run.cancelled = true;
			return true;
		},

		pollEvents,
	};
}
