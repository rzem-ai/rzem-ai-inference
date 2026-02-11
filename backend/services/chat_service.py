"""Chat service — Claude API integration with streaming, tool use, and vision."""

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

logger = logging.getLogger(__name__)

CLAUDE_MODEL = "claude-sonnet-4-20250514"

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
    """Build a system prompt that includes current generation settings."""
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
    """Claude API chat with streaming, tool use, and event buffering."""

    def __init__(self, db: Database) -> None:
        self._db = db
        self._client = None  # anthropic.Anthropic instance
        self._events: deque[ChatEvent] = deque(maxlen=500)
        self._lock = threading.Lock()

    @property
    def is_configured(self) -> bool:
        return self._client is not None

    def set_api_key(self, key: str) -> None:
        """Initialize or replace the Anthropic client."""
        import anthropic
        self._client = anthropic.Anthropic(api_key=key)
        logger.info("Claude API client initialized")

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
    ) -> None:
        """Persist user message and spawn streaming response thread."""
        # Persist user message
        self._db.insert_conversation_message(
            id=str(uuid.uuid4()),
            conversation_id=conversation_id,
            role="user",
            content=user_content,
            image_paths=json.dumps(image_paths) if image_paths else None,
        )
        # Touch conversation updated_at
        self._db.update_conversation(conversation_id)

        # Stream response in background
        thread = threading.Thread(
            target=self._stream_response,
            args=(conversation_id, generation_context),
            daemon=True,
        )
        thread.start()

    def _push_event(self, event_type: str, **data: Any) -> None:
        with self._lock:
            self._events.append(ChatEvent(type=event_type, data=data))

    def _stream_response(
        self,
        conversation_id: str,
        generation_context: dict[str, Any] | None,
    ) -> None:
        """Call Claude API with streaming and handle tool use loop."""
        if not self._client:
            self._push_event("chat_error", conversation_id=conversation_id, error="API key not configured")
            return

        try:
            system_prompt = _build_system_prompt(generation_context)
            messages = self._build_messages(conversation_id)

            self._tool_use_loop(conversation_id, system_prompt, messages)

        except Exception as e:
            logger.exception("Chat stream error for conversation %s", conversation_id)
            self._push_event("chat_error", conversation_id=conversation_id, error=str(e))

    def _tool_use_loop(
        self,
        conversation_id: str,
        system_prompt: str,
        messages: list[dict[str, Any]],
    ) -> None:
        """Stream Claude response, handle tool calls, repeat until text-only."""
        while True:
            full_text = ""
            tool_uses: list[dict[str, Any]] = []

            with self._client.messages.stream(
                model=CLAUDE_MODEL,
                max_tokens=1024,
                system=system_prompt,
                messages=messages,
                tools=TOOLS,
            ) as stream:
                for event in stream:
                    if event.type == "content_block_start":
                        if event.content_block.type == "tool_use":
                            tool_uses.append({
                                "id": event.content_block.id,
                                "name": event.content_block.name,
                                "input_json": "",
                            })
                    elif event.type == "content_block_delta":
                        if event.delta.type == "text_delta":
                            full_text += event.delta.text
                            self._push_event(
                                "chat_chunk",
                                conversation_id=conversation_id,
                                text=event.delta.text,
                            )
                        elif event.delta.type == "input_json_delta":
                            if tool_uses:
                                tool_uses[-1]["input_json"] += event.delta.partial_json

            # If there was text, persist it
            if full_text:
                self._db.insert_conversation_message(
                    id=str(uuid.uuid4()),
                    conversation_id=conversation_id,
                    role="assistant",
                    content=full_text,
                )

            # If no tool calls, we're done
            if not tool_uses:
                self._push_event("chat_complete", conversation_id=conversation_id)
                self._db.update_conversation(conversation_id)
                return

            # Process tool calls
            tool_results = []
            tool_call_records = []
            for tool in tool_uses:
                tool_input = json.loads(tool["input_json"]) if tool["input_json"] else {}
                tool_name = tool["name"]

                self._push_event(
                    "chat_tool_use",
                    conversation_id=conversation_id,
                    tool_name=tool_name,
                    tool_input=tool_input,
                )

                result = self._execute_tool(tool_name, tool_input)
                tool_results.append({
                    "type": "tool_result",
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

            # Build the assistant content blocks for the next turn
            assistant_content: list[dict[str, Any]] = []
            if full_text:
                assistant_content.append({"type": "text", "text": full_text})
            for tool in tool_uses:
                tool_input = json.loads(tool["input_json"]) if tool["input_json"] else {}
                assistant_content.append({
                    "type": "tool_use",
                    "id": tool["id"],
                    "name": tool["name"],
                    "input": tool_input,
                })

            messages.append({"role": "assistant", "content": assistant_content})
            messages.append({"role": "user", "content": tool_results})

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
        """Load conversation history and format for the Anthropic API."""
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
