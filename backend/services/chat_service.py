"""Chat service — multi-provider chat with streaming, tool use, and vision."""

from __future__ import annotations

import base64
import json
import logging
import mimetypes
import threading
import uuid
from collections import deque
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from backend.db.database import Database
from backend.services.providers import StreamEvent
from backend.services.providers.claude import ClaudeProvider
from backend.services.providers.perplexity import PerplexityProvider

logger = logging.getLogger(__name__)

DEFAULT_CLAUDE_MODEL = "claude-sonnet-4-6"
DEFAULT_PERPLEXITY_MODEL = "anthropic/claude-sonnet-4-6"

TOOLS = [
    {
        "name": "update_prompt",
        "description": "Update the user's generation prompt text.",
        "input_schema": {
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "The new prompt text to set.",
                },
            },
            "required": ["prompt"],
        },
    },
    {
        "name": "update_generation_settings",
        "description": "Modify one or more generation parameters. Only include the fields you want to change.",
        "input_schema": {
            "type": "object",
            "properties": {
                "width": {"type": "integer", "description": "Image width in pixels (e.g. 512, 768, 1024)."},
                "height": {"type": "integer", "description": "Image height in pixels (e.g. 512, 768, 1024)."},
                "steps": {"type": "integer", "description": "Number of inference steps (e.g. 15-50)."},
                "cfg_scale": {"type": "number", "description": "Classifier-free guidance scale (e.g. 1.0-20.0)."},
                "seed": {"type": "integer", "description": "Random seed (-1 for random)."},
            },
        },
    },
]


def _build_system_prompt(generation_context: dict[str, Any] | None) -> str:
    """Build a system prompt that includes current generation settings (for tool-capable providers)."""
    base = (
        "You are an AI image generation assistant embedded in a sidebar panel. "
        "You help users craft better prompts and adjust generation settings.\n\n"
        "Guidelines:\n"
        "- Be concise — you're in a narrow sidebar, keep responses short.\n"
        "- When the user asks you to change the prompt, use the update_prompt tool.\n"
        "- When they ask to change dimensions, steps, seed, or cfg_scale, use the update_generation_settings tool.\n"
        "- Always explain your reasoning briefly when making changes.\n"
        "- When analyzing images, describe what you see and suggest improvements.\n"
        "- Use markdown for formatting (bold, lists, etc.) but keep it concise.\n"
    )

    if generation_context:
        ctx_lines = ["\nCurrent generation settings:"]
        for key, val in generation_context.items():
            if val is not None:
                ctx_lines.append(f"- {key}: {val}")
        base += "\n".join(ctx_lines)

    return base


def _build_system_prompt_no_tools(generation_context: dict[str, Any] | None) -> str:
    """Build a system prompt for providers without native tool support.

    Embeds tool instructions as text, telling the model to output JSON blocks
    when it wants to change settings.
    """
    base = (
        "You are an AI image generation assistant embedded in a sidebar panel. "
        "You help users craft better prompts and adjust generation settings.\n\n"
        "Guidelines:\n"
        "- Be concise — you're in a narrow sidebar, keep responses short.\n"
        "- Always explain your reasoning briefly when making changes.\n"
        "- When analyzing images, describe what you see and suggest improvements.\n"
        "- Use markdown for formatting (bold, lists, etc.) but keep it concise.\n\n"
        "## Changing Settings\n\n"
        "You can change the user's generation settings by outputting a JSON block. "
        "When the user asks you to change the prompt or generation settings, include "
        "a fenced JSON block in your response.\n\n"
        "To update the prompt, output:\n"
        "```json\n"
        '{"tool": "update_prompt", "prompt": "the new prompt text here"}\n'
        "```\n\n"
        "To update generation settings (width, height, steps, cfg_scale, seed), output:\n"
        "```json\n"
        '{"tool": "update_generation_settings", "width": 1024, "height": 768}\n'
        "```\n"
        "Only include the fields you want to change. Available fields: "
        "width (int), height (int), steps (int), cfg_scale (float), seed (int, -1 for random).\n\n"
        "You may include multiple JSON blocks in a single response if needed. "
        "Always add a brief explanation alongside the JSON block.\n"
    )

    if generation_context:
        ctx_lines = ["\nCurrent generation settings:"]
        for key, val in generation_context.items():
            if val is not None:
                ctx_lines.append(f"- {key}: {val}")
        base += "\n".join(ctx_lines)

    return base


def _load_image_base64(file_path: str) -> tuple[str, str] | None:
    """Load an image file as base64 data with its media type."""
    path = Path(file_path)
    if not path.is_file():
        return None
    mime = mimetypes.guess_type(str(path))[0] or "image/png"
    data = base64.standard_b64encode(path.read_bytes()).decode("ascii")
    return mime, data


@dataclass
class ChatEvent:
    """Serializable chat event for the frontend."""
    type: str
    data: dict[str, Any]


class ChatService:
    """Multi-provider chat with streaming, tool use, and event buffering."""

    def __init__(self, db: Database) -> None:
        self._db = db
        self._events: deque[ChatEvent] = deque(maxlen=500)
        self._lock = threading.Lock()

        # Provider registry
        self._providers: dict[str, ClaudeProvider | PerplexityProvider] = {
            "claude": ClaudeProvider(),
            "perplexity": PerplexityProvider(),
        }

    @property
    def active_provider(self) -> ClaudeProvider | PerplexityProvider:
        """Return the currently active provider based on AI_PROVIDER setting."""
        name = self.active_provider_name
        return self._providers[name]

    @property
    def active_provider_name(self) -> str:
        """Read AI_PROVIDER setting from DB, default to 'claude'."""
        name = self._db.get_setting("AI_PROVIDER")
        if name and name in self._providers:
            return name
        return "claude"

    @property
    def is_configured(self) -> bool:
        return self.active_provider.is_configured

    def set_provider_api_key(self, provider_name: str, api_key: str) -> None:
        """Configure an API key for a specific provider."""
        provider = self._providers.get(provider_name)
        if not provider:
            raise ValueError(f"Unknown provider: {provider_name}")
        provider.configure(api_key)
        logger.info("API key configured for provider: %s", provider_name)

    def set_api_key(self, key: str) -> None:
        """Legacy method — delegates to set_provider_api_key('claude', ...)."""
        self.set_provider_api_key("claude", key)

    def drain_events(self) -> list[dict[str, Any]]:
        """Return and clear all buffered chat events."""
        with self._lock:
            events = list(self._events)
            self._events.clear()
        return [asdict(e) for e in events]

    def send_message(
        self,
        conversation_id: str,
        user_content: str,
        image_paths: list[str] | None = None,
        generation_context: dict[str, Any] | None = None,
        display_text: str | None = None,
    ) -> None:
        """Persist user message and spawn streaming response thread."""
        self._db.insert_conversation_message(
            id=str(uuid.uuid4()),
            conversation_id=conversation_id,
            role="user",
            content=user_content,
            display_text=display_text,
            image_paths=json.dumps(image_paths) if image_paths else None,
        )
        self._db.update_conversation(conversation_id)

        thread = threading.Thread(
            target=self._stream_response,
            args=(conversation_id, generation_context),
            daemon=True,
        )
        thread.start()

    def _push_event(self, event_type: str, **data: Any) -> None:
        with self._lock:
            self._events.append(ChatEvent(type=event_type, data=data))

    def _get_model(self) -> str:
        """Read the configured model for the active provider."""
        provider_name = self.active_provider_name
        if provider_name == "perplexity":
            model = self._db.get_setting("PERPLEXITY_MODEL")
            return model if model else DEFAULT_PERPLEXITY_MODEL
        else:
            model = self._db.get_setting("CLAUDE_MODEL")
            return model if model else DEFAULT_CLAUDE_MODEL

    def _stream_response(
        self,
        conversation_id: str,
        generation_context: dict[str, Any] | None,
    ) -> None:
        """Call the active provider's API with streaming and handle tool use loop."""
        provider = self.active_provider
        if not provider.is_configured:
            self._push_event(
                "chat_error",
                conversation_id=conversation_id,
                error="API key not configured",
            )
            return

        try:
            model = self._get_model()
            use_tools = provider.supports_tools(model)

            if use_tools:
                system_prompt = _build_system_prompt(generation_context)
            else:
                system_prompt = _build_system_prompt_no_tools(generation_context)

            messages = self._build_messages(conversation_id)
            self._tool_use_loop(conversation_id, system_prompt, messages, model, provider)

        except Exception as e:
            logger.exception("Chat stream error for conversation %s", conversation_id)
            self._push_event("chat_error", conversation_id=conversation_id, error=str(e))

    def _tool_use_loop(
        self,
        conversation_id: str,
        system_prompt: str,
        messages: list[dict[str, Any]],
        model: str,
        provider: ClaudeProvider | PerplexityProvider,
    ) -> None:
        """Stream provider response, handle tool calls, repeat until text-only."""
        has_build_tool_messages = hasattr(provider, "build_tool_result_messages")

        while True:
            full_text = ""
            tool_calls: list[dict[str, Any]] = []

            for event in provider.stream(messages, system_prompt, TOOLS, model):
                if event.type == "text_delta":
                    full_text += event.text or ""
                    self._push_event(
                        "chat_chunk",
                        conversation_id=conversation_id,
                        text=event.text or "",
                    )
                elif event.type == "tool_call":
                    tool_calls.append({
                        "id": event.tool_id,
                        "name": event.tool_name,
                        "tool_input": event.tool_input or {},
                    })
                # "done" is just a sentinel, no action needed

            # Persist assistant text if any
            if full_text:
                self._db.insert_conversation_message(
                    id=str(uuid.uuid4()),
                    conversation_id=conversation_id,
                    role="assistant",
                    content=full_text,
                )

            # No tool calls -> we're done
            if not tool_calls:
                self._push_event("chat_complete", conversation_id=conversation_id)
                self._db.update_conversation(conversation_id)
                return

            # Process tool calls
            tool_results = []
            tool_call_records = []
            for tool in tool_calls:
                tool_name = tool["name"]
                tool_input = tool["tool_input"]

                self._push_event(
                    "chat_tool_use",
                    conversation_id=conversation_id,
                    tool_name=tool_name,
                    tool_input=tool_input,
                )

                result = self._execute_tool(tool_name, tool_input)
                tool_results.append({
                    "tool_use_id": tool["id"],
                    "content": json.dumps(result),
                })
                tool_call_records.append({
                    "name": tool_name,
                    "input": tool_input,
                    "result": result,
                })

            # Persist assistant message with tool calls
            self._db.insert_conversation_message(
                id=str(uuid.uuid4()),
                conversation_id=conversation_id,
                role="assistant",
                content=full_text or "",
                tool_calls=json.dumps(tool_call_records),
            )

            # If provider supports tool result messages, build them and loop
            if has_build_tool_messages:
                new_messages = provider.build_tool_result_messages(
                    full_text, tool_calls, tool_results
                )
                messages.extend(new_messages)
                # Continue the loop for the next round
            else:
                # Provider doesn't support tool result loop — complete after first tool call
                self._push_event("chat_complete", conversation_id=conversation_id)
                self._db.update_conversation(conversation_id)
                return

    def _execute_tool(self, name: str, tool_input: dict[str, Any]) -> dict[str, str]:
        """Execute a tool and return a result dict. Actual param changes are
        applied on the frontend via chat_tool_use events."""
        if name == "update_prompt":
            return {"status": "success", "message": f"Prompt updated to: {tool_input.get('prompt', '')}"}
        elif name == "update_generation_settings":
            changed = ", ".join(f"{k}={v}" for k, v in tool_input.items())
            return {"status": "success", "message": f"Settings updated: {changed}"}
        return {"status": "error", "message": f"Unknown tool: {name}"}

    def _build_messages(self, conversation_id: str) -> list[dict[str, Any]]:
        """Load conversation history and format for the API."""
        db_messages = self._db.get_conversation_messages(conversation_id)
        api_messages: list[dict[str, Any]] = []

        for msg in db_messages:
            content: list[dict[str, Any]] = []

            # Add images if present
            if msg.get("image_paths"):
                try:
                    paths = json.loads(msg["image_paths"])
                    for path in paths:
                        img_data = _load_image_base64(path)
                        if img_data:
                            mime, data = img_data
                            content.append({
                                "type": "image",
                                "source": {
                                    "type": "base64",
                                    "media_type": mime,
                                    "data": data,
                                },
                            })
                except (json.JSONDecodeError, TypeError):
                    pass

            # Add text content
            if msg["content"]:
                content.append({"type": "text", "text": msg["content"]})

            if content:
                api_messages.append({
                    "role": msg["role"],
                    "content": content if len(content) > 1 else msg["content"],
                })

        return api_messages
