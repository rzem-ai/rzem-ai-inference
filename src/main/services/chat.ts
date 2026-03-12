/**
 * Chat service: Anthropic Claude integration with streaming and tool use.
 * Migrated from src/bun/services/chat.ts — bun:sqlite → better-sqlite3.
 */

import Anthropic from "@anthropic-ai/sdk";
import type Database from "better-sqlite3";

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
];

function buildSystemPrompt(generationContext?: Record<string, unknown>): string {
	let prompt = `You are an AI assistant integrated into an image generation app called RZEM AI Inference.
You help users create and refine their image generation prompts and settings.

You have access to tools that can directly modify the user's generation parameters:
- update_prompt: Change the prompt text
- update_generation_settings: Modify width, height, steps, cfg_scale, or seed

When the user asks you to create, modify, or improve a prompt, use the update_prompt tool.
When they ask to change generation settings, use the update_generation_settings tool.
For general questions, just respond with helpful text.`;

	if (generationContext) {
		prompt += `\n\nCurrent generation context:\n${JSON.stringify(generationContext, null, 2)}`;
	}
	return prompt;
}

export function createChatService(db: Database.Database) {
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

	async function streamResponse(
		conversationId: string,
		generationContext?: Record<string, unknown>,
	) {
		try {
			const anthropic = getClient();
			const model = getModel();
			const system = buildSystemPrompt(generationContext);
			let messages = buildMessages(conversationId);

			let maxIterations = 5;
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
					emit({
						type: "chat_tool_use",
						data: { conversationId, toolName: call.name, toolInput: call.input },
					});
					toolResults.push({
						type: "tool_result",
						tool_use_id: call.id,
						content: JSON.stringify({ status: "success", message: `Applied ${call.name}` }),
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
