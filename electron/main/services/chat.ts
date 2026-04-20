/**
 * Chat service: Anthropic Claude integration with streaming and tool use.
 * Migrated from src/bun/services/chat.ts — bun:sqlite → better-sqlite3.
 */

import Anthropic from "@anthropic-ai/sdk";
import type Database from "better-sqlite3";
import type { SkillsService, SkillMetadata } from "./skills";

type Row = Record<string, unknown>;

interface ChatEvent {
	type: string;
	data: Record<string, unknown>;
}

interface ToolDefinition {
	name: string;
	description: string;
	input_schema: {
		type: "object";
		properties: Record<string, unknown>;
		required?: string[];
	};
}

interface ApiResponse {
	status: "success" | "error";
	message?: string;
	[key: string]: unknown;
}

const TOOLS: ToolDefinition[] = [
	{
		name: "update_prompt",
		description: "Update the user's image generation prompt text",
		input_schema: {
			type: "object",
			properties: {
				prompt: { type: "string", description: "The new prompt text for image generation" },
			},
			required: ["prompt"],
		},
	},
	{
		name: "update_generation_settings",
		description: "Modify image generation parameters",
		input_schema: {
			type: "object",
			properties: {
				width: { type: "number", description: "Image width in pixels" },
				height: { type: "number", description: "Image height in pixels" },
				steps: { type: "number", description: "Number of inference steps" },
				cfg_scale: { type: "number", description: "Classifier-free guidance scale" },
				seed: { type: "number", description: "Random seed (-1 for random)" },
			},
		},
	},
	{
		name: "read_skill",
		description: "Fetch the full markdown content of a skill from the available skills index. Call this before answering questions on topics covered by a skill — your inline primer is intentionally short.",
		input_schema: {
			type: "object",
			properties: {
				name: { type: "string", description: "The skill name (e.g. 'flux-prompting')" },
			},
			required: ["name"],
		},
	},
];

const FAMILY_PRIMERS: Record<string, string> = {
	flux1_dev:
		"FLUX.1 prefers descriptive natural-language sentences over comma-separated tags. Negative prompts have minimal effect — describe what you want, not what to avoid. CFG 3.0–4.5 is the sweet spot, 28–40 steps.",
	flux1_kontext:
		"FLUX.1 Kontext is an image-to-image edit model. Provide an input image and describe the desired edit in natural-language sentences (not tags). Same prompting style as FLUX.1 Dev.",
	flux2_dev:
		"FLUX.2 uses a Qwen3 encoder, so it handles complex multi-subject scenes and multilingual prompts well. Use descriptive sentences. CFG 1.0 is recommended (higher values over-saturate). 28 steps default.",
	z_image:
		"Z-Image is bilingual (English/Chinese) with an internal Qwen3 encoder. Use descriptive sentences, not tags. Turbo runs 9 steps for fast iteration; Standard runs 28 for quality. CFG 1.0.",
	qwen_image:
		"Qwen-Image is unusually strong at rendering legible text inside images and at complex multi-subject compositions. Quote any text that should appear (e.g. a sign reading \"OPEN\"). Descriptive sentences, CFG 1.0.",
	fal_cloud:
		"This is a FAL.ai cloud bundle — generation runs on remote servers. Steps and CFG values may be ignored by the endpoint. Aspect ratio is set via preset strings, not pixel dimensions. Prompt style follows the underlying model architecture.",
};

const DEFAULT_PRIMER =
	"Write descriptive, specific prompts. Mention subject, setting, lighting, mood, and camera/style cues explicitly. The clearer the scene description, the better the result.";

function formatGenerationContext(ctx?: Record<string, unknown>): string {
	if (!ctx) return "(no generation context provided)";
	const lines: string[] = [];
	const bundleLabel = ctx.bundle_label ?? ctx.model;
	const tt = ctx.transformer_type;
	if (bundleLabel || tt) {
		lines.push(`Model: ${bundleLabel ?? "(unknown)"}${tt ? ` (${tt})` : ""}`);
	}
	if (ctx.tier) lines.push(`Tier: ${ctx.tier}`);
	if (ctx.prompt) lines.push(`Prompt: ${ctx.prompt}`);
	if (ctx.width && ctx.height) lines.push(`Resolution: ${ctx.width}x${ctx.height}`);
	const params: string[] = [];
	if (ctx.steps !== undefined) params.push(`steps=${ctx.steps}`);
	if (ctx.cfg_scale !== undefined) params.push(`cfg=${ctx.cfg_scale}`);
	if (ctx.sampler) params.push(`sampler=${ctx.sampler}`);
	if (ctx.scheduler) params.push(`scheduler=${ctx.scheduler}`);
	if (params.length) lines.push(params.join("  "));
	if (ctx.seed !== undefined) lines.push(`Seed: ${ctx.seed}`);
	const loras = ctx.loras as Array<{ name?: string; strength?: number }> | undefined;
	if (loras?.length) {
		const formatted = loras.map((l) => `${l.name ?? "?"}@${l.strength ?? 1}`).join(", ");
		lines.push(`LoRAs: ${formatted}`);
	} else {
		lines.push("LoRAs: none");
	}
	if (ctx.negative_prompt) lines.push(`Negative prompt: ${ctx.negative_prompt}`);
	return lines.join("\n");
}

function formatSkillIndex(skills: SkillMetadata[]): string {
	if (!skills.length) return "(no skills available)";
	return skills.map((s) => `- ${s.name}: ${s.description}`).join("\n");
}

function buildSystemPrompt(
	generationContext: Record<string, unknown> | undefined,
	skillIndex: SkillMetadata[],
	familyPrimer: string,
	familyLabel: string,
): string {
	return `You are an expert image generation assistant inside RZEM AI Inference, a desktop app for local and cloud diffusion models. Your job is to help the user write better prompts and pick the right generation parameters for the model they're currently using.

# Current generation context
${formatGenerationContext(generationContext)}

# Quick prompting primer for ${familyLabel}
${familyPrimer}

# Available skills (call read_skill to expand)
${formatSkillIndex(skillIndex)}

# Tools
- update_prompt: rewrite the user's image prompt
- update_generation_settings: adjust width / height / steps / cfg_scale / seed
- read_skill: fetch the full content of one of the skills above. Call this before giving advice on topics covered by a skill — your inline primer above is intentionally short.

# Behavior
- When asked to improve a prompt for a model with a relevant skill, call read_skill first, then update_prompt.
- If the user is on a Flux/Qwen/Z-Image bundle and pastes SDXL-style "(masterpiece:1.3), best quality, ..." tag prompting, gently redirect — that style hurts these models' output.
- Be concise. Don't lecture if the user just wants a fix.`;
}

export function createChatService(db: Database.Database, skills: SkillsService) {
	let client: Anthropic | null = null;
	let apiKey: string | null = null;
	const eventBuffer: ChatEvent[] = [];

	function pushEvent(event: ChatEvent) {
		eventBuffer.push(event);
		if (eventBuffer.length > 500) {
			eventBuffer.splice(0, eventBuffer.length - 500);
		}
	}

	let onEventCallback: ((event: ChatEvent) => void) | null = null;

	function emit(event: ChatEvent) {
		pushEvent(event);
		onEventCallback?.(event);
	}

	function getClient(): Anthropic {
		if (!client || !apiKey) {
			const row = db.prepare("SELECT value FROM settings WHERE key = 'CLAUDE_API_KEY'").get() as { value: string } | null;
			if (row?.value) {
				apiKey = row.value;
				client = new Anthropic({ apiKey });
			} else {
				throw new Error("Claude API key not configured");
			}
		}
		return client;
	}

	function getModel(): string {
		const row = db.prepare("SELECT value FROM settings WHERE key = 'CHAT_MODEL'").get() as { value: string } | null;
		return row?.value ?? "claude-sonnet-4-6";
	}

	function buildMessages(conversationId: string): Anthropic.MessageParam[] {
		const rows = db.prepare(
			"SELECT * FROM conversation_messages WHERE conversation_id = ? ORDER BY created_at",
		).all(conversationId) as Row[];

		const messages: Anthropic.MessageParam[] = [];

		for (const row of rows) {
			const role = row.role as "user" | "assistant";
			const content: Anthropic.ContentBlockParam[] = [];

			if (row.image_paths) {
				try {
					const { readFileSync } = require("fs");
					const paths = JSON.parse(row.image_paths as string) as string[];
					for (const imgPath of paths) {
						try {
							const bytes = readFileSync(imgPath);
							const b64 = Buffer.from(bytes).toString("base64");
							const ext = imgPath.split(".").pop()?.toLowerCase() ?? "png";
							const mediaType = ext === "jpg" || ext === "jpeg" ? "image/jpeg" : ext === "webp" ? "image/webp" : "image/png";
							content.push({
								type: "image",
								source: { type: "base64", media_type: mediaType as "image/jpeg" | "image/png" | "image/gif" | "image/webp", data: b64 },
							});
						} catch {
							// Skip images that can't be loaded
						}
					}
				} catch {
					// Invalid JSON
				}
			}

			const text = (row.content as string) ?? "";
			if (text) {
				content.push({ type: "text", text });
			}

			if (content.length > 0) {
				messages.push({ role, content });
			}
		}

		return messages;
	}

	type ToolDispatchResult = {
		resultContent: string;
		emitFrontendEvent?: boolean;
		isError?: boolean;
	};

	async function dispatchTool(
		name: string,
		input: Record<string, unknown>,
	): Promise<ToolDispatchResult> {
		switch (name) {
			case "read_skill": {
				const skillName = input.name as string | undefined;
				if (!skillName) {
					return { resultContent: JSON.stringify({ error: "missing 'name' argument" }), isError: true };
				}
				const content = skills.readSkill(skillName);
				if (!content) {
					return { resultContent: JSON.stringify({ error: `unknown skill: ${skillName}` }), isError: true };
				}
				return { resultContent: content };
			}
			case "update_prompt":
				return {
					resultContent: JSON.stringify({ status: "applied", prompt: input.prompt }),
					emitFrontendEvent: true,
				};
			case "update_generation_settings":
				return {
					resultContent: JSON.stringify({ status: "applied", ...input }),
					emitFrontendEvent: true,
				};
			default:
				return { resultContent: JSON.stringify({ error: `unknown tool: ${name}` }), isError: true };
		}
	}

	async function streamResponse(
		conversationId: string,
		generationContext?: Record<string, unknown>,
	) {
		try {
			const anthropic = getClient();
			const model = getModel();
			const transformerType = generationContext?.transformer_type as string | undefined;
			const skillIndex = skills.listSkills({ modelFamily: transformerType });
			const familyPrimer = (transformerType && FAMILY_PRIMERS[transformerType]) || DEFAULT_PRIMER;
			const familyLabel = (generationContext?.bundle_label as string | undefined) ?? transformerType ?? "this model";
			const system = buildSystemPrompt(generationContext, skillIndex, familyPrimer, familyLabel);
			let messages = buildMessages(conversationId);

			let maxIterations = 8;
			while (maxIterations-- > 0) {
				let fullText = "";
				const toolCalls: Array<{ id: string; name: string; input: Record<string, unknown> }> = [];

				const stream = anthropic.messages.stream({
					model,
					max_tokens: 4096,
					system,
					messages,
					tools: TOOLS as Anthropic.Tool[],
				});

				for await (const event of stream) {
					if (event.type === "content_block_delta") {
						if (event.delta.type === "text_delta") {
							fullText += event.delta.text;
							emit({
								type: "chat_chunk",
								data: { conversationId, text: event.delta.text },
							});
						}
					}
				}

				const finalMessage = await stream.finalMessage();
				for (const block of finalMessage.content) {
					if (block.type === "tool_use") {
						toolCalls.push({
							id: block.id,
							name: block.name,
							input: block.input as Record<string, unknown>,
						});
					}
				}

				if (fullText || toolCalls.length > 0) {
					const msgId = crypto.randomUUID();
					const now = Math.floor(Date.now() / 1000);
					db.prepare(
						"INSERT INTO conversation_messages (id, conversation_id, role, content, tool_calls, created_at) VALUES (?, ?, 'assistant', ?, ?, ?)",
					).run(msgId, conversationId, fullText, toolCalls.length > 0 ? JSON.stringify(toolCalls) : null, now);
				}

				if (toolCalls.length === 0) {
					emit({ type: "chat_complete", data: { conversationId } });
					break;
				}

				const toolResults: Anthropic.ToolResultBlockParam[] = [];
				for (const call of toolCalls) {
					const { resultContent, emitFrontendEvent, isError } = await dispatchTool(call.name, call.input);
					if (emitFrontendEvent) {
						emit({
							type: "chat_tool_use",
							data: { conversationId, toolName: call.name, toolInput: call.input },
						});
					}
					toolResults.push({
						type: "tool_result",
						tool_use_id: call.id,
						content: resultContent,
						...(isError ? { is_error: true } : {}),
					});
				}

				const assistantContent: Anthropic.ContentBlockParam[] = [];
				if (fullText) {
					assistantContent.push({ type: "text", text: fullText });
				}
				for (const call of toolCalls) {
					assistantContent.push({
						type: "tool_use",
						id: call.id,
						name: call.name,
						input: call.input,
					});
				}
				messages = [
					...messages,
					{ role: "assistant", content: assistantContent },
					{ role: "user", content: toolResults },
				];
			}

			db.prepare("UPDATE conversations SET updated_at = ? WHERE id = ?").run(
				Math.floor(Date.now() / 1000),
				conversationId,
			);
		} catch (err) {
			console.error("Chat stream error:", err);
			emit({
				type: "chat_error",
				data: { conversationId, error: String(err) },
			});
		}
	}

	return {
		async complete(messages: Anthropic.MessageParam[], maxTokens = 2048): Promise<string> {
			const anthropic = getClient();
			const model = getModel();
			const response = await anthropic.messages.create({ model, max_tokens: maxTokens, messages });
			const textBlock = response.content.find((b) => b.type === "text");
			return textBlock?.text ?? "";
		},

		async completeWithVision(messages: Anthropic.MessageParam[], maxTokens = 2048): Promise<string> {
			return this.complete(messages, maxTokens);
		},

		onEvent(callback: (event: ChatEvent) => void) {
			onEventCallback = callback;
		},

		drainEvents(): ChatEvent[] {
			return eventBuffer.splice(0, eventBuffer.length);
		},

		isConfigured(): ApiResponse {
			const key = db.prepare("SELECT value FROM settings WHERE key = 'CLAUDE_API_KEY'").get() as { value: string } | null;
			return { status: "success", configured: !!key?.value };
		},

		setApiKey({ apiKey: key, provider }: { apiKey: string; provider?: string }): ApiResponse {
			const keyName = provider === "perplexity" ? "PERPLEXITY_API_KEY" : "CLAUDE_API_KEY";
			db.prepare(
				"INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
			).run(keyName, key);
			if (keyName === "CLAUDE_API_KEY") {
				apiKey = key;
				client = new Anthropic({ apiKey: key });
			}
			return { status: "success" };
		},

		createConversation({ title }: { title?: string }): ApiResponse {
			const id = crypto.randomUUID();
			const now = Math.floor(Date.now() / 1000);
			db.prepare("INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)").run(id, title ?? "New Chat", now, now);
			return { status: "success", conversation: db.prepare("SELECT * FROM conversations WHERE id = ?").get(id) as Row };
		},

		getConversations(): ApiResponse {
			return {
				status: "success",
				conversations: db.prepare("SELECT * FROM conversations ORDER BY updated_at DESC").all() as Row[],
			};
		},

		getMessages({ conversationId }: { conversationId: string }): ApiResponse {
			return {
				status: "success",
				messages: db.prepare("SELECT * FROM conversation_messages WHERE conversation_id = ? ORDER BY created_at").all(conversationId) as Row[],
			};
		},

		deleteConversation({ conversationId }: { conversationId: string }): ApiResponse {
			db.prepare("DELETE FROM conversations WHERE id = ?").run(conversationId);
			return { status: "success" };
		},

		getProviderInfo(): ApiResponse {
			return {
				status: "success",
				provider: "claude",
				models: [
					{ id: "claude-haiku-4-5-20251001", label: "Claude Haiku 4.5 — Fast, low cost" },
					{ id: "claude-sonnet-4-6", label: "Claude Sonnet 4.6 — Balanced (default)" },
					{ id: "claude-opus-4-6", label: "Claude Opus 4.6 — Most capable" },
				],
			};
		},

		async sendMessage(params: Record<string, unknown>): Promise<ApiResponse> {
			const conversationId = params.conversationId as string;
			const content = params.content as string;
			const imagePaths = (params.imagePaths as string[] | undefined) ?? null;
			const generationContext = params.generationContext as Record<string, unknown> | undefined;
			const displayText = params.displayText as string | undefined;

			if (!conversationId || !content) {
				return { status: "error", message: "conversationId and content are required" };
			}

			const msgId = crypto.randomUUID();
			const now = Math.floor(Date.now() / 1000);
			db.prepare(
				"INSERT INTO conversation_messages (id, conversation_id, role, content, display_text, image_paths, created_at) VALUES (?, ?, 'user', ?, ?, ?, ?)",
			).run(msgId, conversationId, content, displayText ?? null, imagePaths ? JSON.stringify(imagePaths) : null, now);

			db.prepare("UPDATE conversations SET updated_at = ? WHERE id = ?").run(now, conversationId);

			streamResponse(conversationId, generationContext);

			return { status: "success" };
		},
	};
}
