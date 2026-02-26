# Multi-Provider Chat Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Perplexity as an alternative AI chat provider alongside Claude, with a provider abstraction layer and global provider setting.

**Architecture:** Extract current Claude-specific logic from `ChatService` into a `ClaudeProvider` class. Create a `PerplexityProvider` class using the `perplexityai` SDK's `responses.create()` API. `ChatService` becomes a thin orchestrator that delegates to the active provider via a `ChatProvider` protocol. A global `AI_PROVIDER` setting switches between providers.

**Tech Stack:** Python (anthropic SDK, perplexityai SDK), Vue 3 + TypeScript + PrimeVue, SQLite settings table

---

### Task 1: Install perplexityai SDK

**Files:**
- Modify: `pyproject.toml`

**Step 1: Add the dependency**

```bash
cd /home/alex/Dev/Work/rzem-ai-inference
uv add perplexityai
```

This adds `perplexityai` to `pyproject.toml` and installs it into the venv.

**Step 2: Verify installation**

```bash
uv run python -c "from perplexity import Perplexity; print('OK')"
```

Expected: `OK`

**Step 3: Commit**

```bash
git add pyproject.toml uv.lock
git commit -m "feat: add perplexityai SDK dependency"
```

---

### Task 2: Create provider protocol and StreamEvent

**Files:**
- Create: `backend/services/providers/__init__.py`

**Step 1: Create the providers package with protocol and event types**

Create `backend/services/providers/__init__.py`:

```python
"""Chat provider abstraction — protocol and shared types."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Iterator, Protocol, runtime_checkable


@dataclass
class StreamEvent:
    """Normalized streaming event emitted by any provider."""

    type: str  # "text_delta" | "tool_call" | "done"
    text: str | None = None
    tool_id: str | None = None
    tool_name: str | None = None
    tool_input: dict[str, Any] | None = None


@dataclass
class ProviderModel:
    """A model offered by a provider."""

    id: str
    label: str


@runtime_checkable
class ChatProvider(Protocol):
    """Interface that every chat provider must implement."""

    name: str

    def configure(self, api_key: str) -> None: ...

    @property
    def is_configured(self) -> bool: ...

    def stream(
        self,
        messages: list[dict[str, Any]],
        system_prompt: str,
        tools: list[dict[str, Any]],
        model: str,
    ) -> Iterator[StreamEvent]: ...

    def get_models(self) -> list[ProviderModel]: ...

    def supports_tools(self, model: str) -> bool: ...
```

**Step 2: Verify import**

```bash
uv run python -c "from backend.services.providers import ChatProvider, StreamEvent, ProviderModel; print('OK')"
```

Expected: `OK`

**Step 3: Commit**

```bash
git add backend/services/providers/__init__.py
git commit -m "feat: add ChatProvider protocol and StreamEvent types"
```

---

### Task 3: Extract ClaudeProvider from ChatService

**Files:**
- Create: `backend/services/providers/claude.py`
- Modify: `backend/services/chat_service.py`

This is the largest task. We extract all Anthropic-specific logic from `ChatService` into `ClaudeProvider`, then update `ChatService` to use the provider interface.

**Step 1: Create ClaudeProvider**

Create `backend/services/providers/claude.py`:

```python
"""Claude chat provider — Anthropic SDK integration."""

from __future__ import annotations

import json
import logging
from typing import Any, Iterator

from backend.services.providers import ChatProvider, ProviderModel, StreamEvent

logger = logging.getLogger(__name__)


CLAUDE_MODELS = [
    ProviderModel(id="claude-haiku-4-5-20251001", label="Claude Haiku 4.6 — Fast, low cost"),
    ProviderModel(id="claude-sonnet-4-6", label="Claude Sonnet 4.6 — Balanced (default)"),
    ProviderModel(id="claude-opus-4-6", label="Claude Opus 4.6 — Most capable"),
]


class ClaudeProvider:
    """Anthropic Claude provider using messages.stream()."""

    name = "claude"

    def __init__(self) -> None:
        self._client = None

    def configure(self, api_key: str) -> None:
        import anthropic
        self._client = anthropic.Anthropic(api_key=api_key)
        logger.info("Claude provider configured")

    @property
    def is_configured(self) -> bool:
        return self._client is not None

    def get_models(self) -> list[ProviderModel]:
        return CLAUDE_MODELS

    def supports_tools(self, model: str) -> bool:
        return True  # All Claude models support tool use

    def stream(
        self,
        messages: list[dict[str, Any]],
        system_prompt: str,
        tools: list[dict[str, Any]],
        model: str,
    ) -> Iterator[StreamEvent]:
        """Stream Claude response, yielding normalized StreamEvents."""
        if not self._client:
            raise RuntimeError("Claude provider not configured")

        # Convert tool definitions to Anthropic format (they already are)
        with self._client.messages.stream(
            model=model,
            max_tokens=1024,
            system=system_prompt,
            messages=messages,
            tools=tools,
        ) as stream:
            current_tool: dict[str, Any] | None = None

            for event in stream:
                if event.type == "content_block_start":
                    if event.content_block.type == "tool_use":
                        current_tool = {
                            "id": event.content_block.id,
                            "name": event.content_block.name,
                            "input_json": "",
                        }
                elif event.type == "content_block_delta":
                    if event.delta.type == "text_delta":
                        yield StreamEvent(type="text_delta", text=event.delta.text)
                    elif event.delta.type == "input_json_delta":
                        if current_tool is not None:
                            current_tool["input_json"] += event.delta.partial_json
                elif event.type == "content_block_stop":
                    if current_tool is not None:
                        tool_input = (
                            json.loads(current_tool["input_json"])
                            if current_tool["input_json"]
                            else {}
                        )
                        yield StreamEvent(
                            type="tool_call",
                            tool_id=current_tool["id"],
                            tool_name=current_tool["name"],
                            tool_input=tool_input,
                        )
                        current_tool = None

        yield StreamEvent(type="done")

    def build_tool_result_messages(
        self,
        full_text: str,
        tool_calls: list[dict[str, Any]],
        tool_results: list[dict[str, Any]],
    ) -> tuple[dict[str, Any], dict[str, Any]]:
        """Build the assistant + user (tool_result) messages for the next turn.

        Claude requires the assistant message to contain the text + tool_use blocks,
        and the follow-up user message to contain the tool_result blocks.
        """
        assistant_content: list[dict[str, Any]] = []
        if full_text:
            assistant_content.append({"type": "text", "text": full_text})
        for tc in tool_calls:
            assistant_content.append({
                "type": "tool_use",
                "id": tc["id"],
                "name": tc["name"],
                "input": tc["input"],
            })

        user_content = [
            {
                "type": "tool_result",
                "tool_use_id": tc["id"],
                "content": json.dumps(tr["result"]),
            }
            for tc, tr in zip(tool_calls, tool_results)
        ]

        return (
            {"role": "assistant", "content": assistant_content},
            {"role": "user", "content": user_content},
        )
```

**Step 2: Refactor ChatService to use ClaudeProvider**

Rewrite `backend/services/chat_service.py`. The key changes:

1. Remove direct `anthropic` import and client creation
2. Add `_providers` dict and `active_provider` property
3. Refactor `_tool_use_loop` to consume `StreamEvent` from provider
4. Add `set_provider_api_key(provider_name, api_key)` method
5. Keep `_build_messages`, `_execute_tool`, `_push_event`, `_build_system_prompt` unchanged

```python
"""Chat service — multi-provider AI integration with streaming, tool use, and vision."""

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


def _build_system_prompt_no_tools(generation_context: dict[str, Any] | None) -> str:
    """Build a system prompt for providers/models that don't support tool use.

    Instead of tool definitions, we embed instructions telling the model to
    output JSON blocks when it wants to change settings.
    """
    base = (
        "You are an AI image generation assistant embedded in a sidebar panel. "
        "You help users craft better prompts and adjust generation settings.\n\n"
        "Guidelines:\n"
        "- Be concise — you're in a narrow sidebar, keep responses short.\n"
        "- When analyzing images, describe what you see and suggest improvements.\n"
        "- Use markdown for formatting (bold, lists, etc.) but keep it concise.\n"
        "\n"
        "When the user asks you to change the prompt or generation settings, "
        "output a JSON block on its own line like this:\n"
        '```json\n{"tool": "update_prompt", "prompt": "the new prompt text"}\n```\n'
        "or:\n"
        '```json\n{"tool": "update_generation_settings", "width": 1024, "height": 768}\n```\n'
        "Only include the fields you want to change. Always explain your reasoning.\n"
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
        self._providers: dict[str, Any] = {
            "claude": ClaudeProvider(),
        }
        self._events: deque[ChatEvent] = deque(maxlen=500)
        self._lock = threading.Lock()

    def _register_provider(self, name: str, provider: Any) -> None:
        """Register a provider instance."""
        self._providers[name] = provider

    @property
    def active_provider_name(self) -> str:
        return self._db.get_setting("AI_PROVIDER") or "claude"

    @property
    def active_provider(self):
        name = self.active_provider_name
        provider = self._providers.get(name)
        if provider is None:
            # Fall back to claude
            provider = self._providers["claude"]
        return provider

    @property
    def is_configured(self) -> bool:
        return self.active_provider.is_configured

    def set_provider_api_key(self, provider_name: str, api_key: str) -> None:
        """Configure a specific provider with an API key."""
        provider = self._providers.get(provider_name)
        if provider is None:
            raise ValueError(f"Unknown provider: {provider_name}")
        provider.configure(api_key)
        logger.info("Provider '%s' configured", provider_name)

    # Legacy method for backward compatibility during migration
    def set_api_key(self, key: str) -> None:
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
        model = self._db.get_setting("CLAUDE_MODEL")
        return model if model else DEFAULT_CLAUDE_MODEL

    def _stream_response(
        self,
        conversation_id: str,
        generation_context: dict[str, Any] | None,
    ) -> None:
        """Call the active provider with streaming and handle tool use loop."""
        provider = self.active_provider
        if not provider.is_configured:
            self._push_event("chat_error", conversation_id=conversation_id, error="API key not configured")
            return

        try:
            model = self._get_model()
            use_tools = provider.supports_tools(model)

            if use_tools:
                system_prompt = _build_system_prompt(generation_context)
            else:
                system_prompt = _build_system_prompt_no_tools(generation_context)

            messages = self._build_messages(conversation_id)
            self._tool_use_loop(conversation_id, system_prompt, messages, model, provider, use_tools)

        except Exception as e:
            logger.exception("Chat stream error for conversation %s", conversation_id)
            self._push_event("chat_error", conversation_id=conversation_id, error=str(e))

    def _tool_use_loop(
        self,
        conversation_id: str,
        system_prompt: str,
        messages: list[dict[str, Any]],
        model: str,
        provider: Any,
        use_tools: bool,
    ) -> None:
        """Stream provider response, handle tool calls, repeat until text-only."""
        while True:
            full_text = ""
            tool_calls: list[dict[str, Any]] = []

            tools = TOOLS if use_tools else []
            for event in provider.stream(messages, system_prompt, tools, model):
                if event.type == "text_delta":
                    full_text += event.text or ""
                    self._push_event(
                        "chat_chunk",
                        conversation_id=conversation_id,
                        text=event.text,
                    )
                elif event.type == "tool_call":
                    tool_calls.append({
                        "id": event.tool_id,
                        "name": event.tool_name,
                        "input": event.tool_input or {},
                    })
                elif event.type == "done":
                    pass  # End of stream

            # If there was text, persist it
            if full_text:
                self._db.insert_conversation_message(
                    id=str(uuid.uuid4()),
                    conversation_id=conversation_id,
                    role="assistant",
                    content=full_text,
                )

            # If no tool calls, we're done
            if not tool_calls:
                self._push_event("chat_complete", conversation_id=conversation_id)
                self._db.update_conversation(conversation_id)
                return

            # Process tool calls
            tool_results = []
            tool_call_records = []
            for tc in tool_calls:
                self._push_event(
                    "chat_tool_use",
                    conversation_id=conversation_id,
                    tool_name=tc["name"],
                    tool_input=tc["input"],
                )
                result = self._execute_tool(tc["name"], tc["input"])
                tool_results.append({"id": tc["id"], "result": result})
                tool_call_records.append({
                    "name": tc["name"],
                    "input": tc["input"],
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

            # Build follow-up messages for tool result loop
            # This is provider-specific (Claude uses content blocks, others use function messages)
            if hasattr(provider, "build_tool_result_messages"):
                assistant_msg, user_msg = provider.build_tool_result_messages(
                    full_text, tool_calls, tool_results,
                )
                messages.append(assistant_msg)
                messages.append(user_msg)
            else:
                # Generic fallback — just complete (no multi-turn tool loop)
                self._push_event("chat_complete", conversation_id=conversation_id)
                self._db.update_conversation(conversation_id)
                return

    def _execute_tool(self, name: str, tool_input: dict[str, Any]) -> dict[str, str]:
        """Execute a tool and return a result dict."""
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

            if msg["content"]:
                content.append({"type": "text", "text": msg["content"]})

            if content:
                api_messages.append({
                    "role": msg["role"],
                    "content": content if len(content) > 1 else msg["content"],
                })

        return api_messages
```

**Step 3: Verify the refactor**

```bash
uv run python -c "
from backend.db.database import Database
from backend.services.chat_service import ChatService
db = Database('/tmp/test_chat.db')
db.connect()
svc = ChatService(db=db)
print('active_provider:', svc.active_provider.name)
print('is_configured:', svc.is_configured)
print('OK')
"
```

Expected: `active_provider: claude`, `is_configured: False`, `OK`

**Step 4: Commit**

```bash
git add backend/services/providers/claude.py backend/services/chat_service.py
git commit -m "refactor: extract ClaudeProvider from ChatService, add provider abstraction"
```

---

### Task 4: Create PerplexityProvider

**Files:**
- Create: `backend/services/providers/perplexity.py`
- Modify: `backend/services/chat_service.py` (register perplexity provider)

**Step 1: Create PerplexityProvider**

Create `backend/services/providers/perplexity.py`:

```python
"""Perplexity chat provider — responses.create() API with frontier model access."""

from __future__ import annotations

import json
import logging
import re
from typing import Any, Iterator

from backend.services.providers import ChatProvider, ProviderModel, StreamEvent

logger = logging.getLogger(__name__)


PERPLEXITY_MODELS = [
    ProviderModel(id="perplexity/sonar", label="Sonar — Fast, web search"),
    ProviderModel(id="openai/gpt-5.2", label="GPT-5.2 — Frontier"),
    ProviderModel(id="openai/gpt-5.1", label="GPT-5.1 — Mid-tier"),
    ProviderModel(id="openai/gpt-5-mini", label="GPT-5 Mini — Budget"),
    ProviderModel(id="anthropic/claude-sonnet-4-6", label="Claude Sonnet 4.6 — Balanced (default)"),
    ProviderModel(id="anthropic/claude-haiku-4-5", label="Claude Haiku 4.5 — Fast"),
    ProviderModel(id="google/gemini-2.5-pro", label="Gemini 2.5 Pro"),
    ProviderModel(id="google/gemini-2.5-flash", label="Gemini 2.5 Flash — Budget"),
]

# Models known to support function calling through Perplexity
_TOOL_CAPABLE_MODELS = {
    "perplexity/sonar",
    "openai/gpt-5.2",
    "openai/gpt-5.1",
    "openai/gpt-5-mini",
    "anthropic/claude-sonnet-4-6",
    "anthropic/claude-haiku-4-5",
}


class PerplexityProvider:
    """Perplexity provider using the responses.create() Agent API."""

    name = "perplexity"

    def __init__(self) -> None:
        self._client = None

    def configure(self, api_key: str) -> None:
        from perplexity import Perplexity
        self._client = Perplexity(api_key=api_key)
        logger.info("Perplexity provider configured")

    @property
    def is_configured(self) -> bool:
        return self._client is not None

    def get_models(self) -> list[ProviderModel]:
        return PERPLEXITY_MODELS

    def supports_tools(self, model: str) -> bool:
        # For now, we don't use native tool calling through Perplexity's
        # responses API — we use the text-based fallback with JSON blocks.
        # The responses.create() API's tool support is for built-in tools
        # like web_search, not custom function definitions.
        return False

    def stream(
        self,
        messages: list[dict[str, Any]],
        system_prompt: str,
        tools: list[dict[str, Any]],
        model: str,
    ) -> Iterator[StreamEvent]:
        """Call Perplexity responses.create() and yield StreamEvents.

        The responses API takes a single `input` string and `instructions`
        for system prompt. We flatten conversation history into the input.
        """
        if not self._client:
            raise RuntimeError("Perplexity provider not configured")

        # Flatten conversation messages into a single input string
        input_text = self._flatten_messages(messages)

        try:
            response = self._client.responses.create(
                model=model,
                input=input_text,
                instructions=system_prompt,
            )

            output_text = response.output_text or ""

            # Parse any embedded tool JSON blocks from the response
            tool_calls = self._parse_tool_blocks(output_text)

            if tool_calls:
                # Strip tool JSON blocks from the displayed text
                clean_text = self._strip_tool_blocks(output_text)
                if clean_text.strip():
                    yield StreamEvent(type="text_delta", text=clean_text)

                for tc in tool_calls:
                    yield StreamEvent(
                        type="tool_call",
                        tool_id=f"pplx_{id(tc)}",
                        tool_name=tc["tool"],
                        tool_input={k: v for k, v in tc.items() if k != "tool"},
                    )
            else:
                # No tools — just emit the full text
                yield StreamEvent(type="text_delta", text=output_text)

        except Exception as e:
            logger.exception("Perplexity API error")
            raise

        yield StreamEvent(type="done")

    def _flatten_messages(self, messages: list[dict[str, Any]]) -> str:
        """Convert conversation message history into a single input string.

        The Perplexity responses API takes a single `input` string, not a
        messages array. We format previous messages as a transcript so the
        model has conversation context.
        """
        if not messages:
            return ""

        # If there's only one message (the latest user message), just return its text
        if len(messages) == 1:
            return self._extract_text(messages[0])

        # Build a transcript of previous messages, then append the latest
        parts: list[str] = []
        for msg in messages[:-1]:
            role = msg.get("role", "user").capitalize()
            text = self._extract_text(msg)
            if text:
                parts.append(f"{role}: {text}")

        # Latest message is the user's current input
        latest = self._extract_text(messages[-1])

        if parts:
            transcript = "\n\n".join(parts)
            return f"Previous conversation:\n{transcript}\n\n---\n\n{latest}"
        return latest

    def _extract_text(self, message: dict[str, Any]) -> str:
        """Extract text content from a message, handling both string and block formats."""
        content = message.get("content", "")
        if isinstance(content, str):
            return content
        if isinstance(content, list):
            texts = [b["text"] for b in content if isinstance(b, dict) and b.get("type") == "text"]
            return " ".join(texts)
        return str(content)

    def _parse_tool_blocks(self, text: str) -> list[dict[str, Any]]:
        """Extract tool call JSON blocks from response text.

        Looks for patterns like:
        ```json
        {"tool": "update_prompt", "prompt": "..."}
        ```
        """
        pattern = r'```json\s*\n(\{[^`]*?"tool"\s*:[^`]*?\})\s*\n```'
        matches = re.findall(pattern, text, re.DOTALL)
        results = []
        for match in matches:
            try:
                parsed = json.loads(match)
                if "tool" in parsed:
                    results.append(parsed)
            except json.JSONDecodeError:
                continue
        return results

    def _strip_tool_blocks(self, text: str) -> str:
        """Remove tool JSON blocks from response text for display."""
        pattern = r'```json\s*\n\{[^`]*?"tool"\s*:[^`]*?\}\s*\n```'
        return re.sub(pattern, "", text, flags=re.DOTALL).strip()
```

**Step 2: Register PerplexityProvider in ChatService**

In `backend/services/chat_service.py`, update the `__init__` to register the Perplexity provider. In the imports section at the top, add:

```python
from backend.services.providers.perplexity import PerplexityProvider
```

In `__init__`, change the `_providers` dict:

```python
self._providers: dict[str, Any] = {
    "claude": ClaudeProvider(),
    "perplexity": PerplexityProvider(),
}
```

**Step 3: Verify**

```bash
uv run python -c "
from backend.db.database import Database
from backend.services.chat_service import ChatService
db = Database('/tmp/test_chat2.db')
db.connect()
svc = ChatService(db=db)
print('providers:', list(svc._providers.keys()))
print('perplexity models:', [(m.id, m.label) for m in svc._providers['perplexity'].get_models()])
print('OK')
"
```

Expected: prints both providers and the Perplexity model list

**Step 4: Commit**

```bash
git add backend/services/providers/perplexity.py backend/services/chat_service.py
git commit -m "feat: add PerplexityProvider with responses.create() API"
```

---

### Task 5: Update API layer and main.py startup

**Files:**
- Modify: `backend/api/chat.py`
- Modify: `main.py`

**Step 1: Update chat_set_api_key to accept provider param**

In `backend/api/chat.py`, update the `chat_set_api_key` method:

```python
def chat_set_api_key(self, api_key: str, provider: str = "claude", **kwargs) -> dict[str, Any]:
    try:
        self._chat.set_provider_api_key(provider, api_key)
        # Store with provider-specific key
        key_name = "CLAUDE_API_KEY" if provider == "claude" else "PERPLEXITY_API_KEY"
        self._chat_db.set_setting(key_name, api_key)
        return {"status": "success"}
    except Exception as e:
        logger.error("Failed to set API key: %s", e)
        return {"status": "error", "message": str(e)}
```

Also add a method to get provider info for the frontend:

```python
def chat_get_provider_info(self, **kwargs) -> dict[str, Any]:
    """Return active provider name and its available models."""
    try:
        provider = self._chat.active_provider
        models = [{"id": m.id, "label": m.label} for m in provider.get_models()]
        return {
            "status": "success",
            "provider": provider.name,
            "models": models,
        }
    except Exception as e:
        logger.error("Failed to get provider info: %s", e)
        return {"status": "error", "message": str(e)}
```

**Step 2: Update main.py to init both providers**

In `main.py`, replace the Claude-only initialization (lines 111-114):

```python
chat_service = ChatService(db=db)
claude_key = db.get_setting("CLAUDE_API_KEY")
if claude_key:
    chat_service.set_provider_api_key("claude", claude_key)
perplexity_key = db.get_setting("PERPLEXITY_API_KEY")
if perplexity_key:
    chat_service.set_provider_api_key("perplexity", perplexity_key)
```

**Step 3: Test startup**

```bash
cd /home/alex/Dev/Work/rzem-ai-inference
uv run python -c "
from backend.db.database import Database
from backend.config import AppConfig
config = AppConfig()
db = Database(config.data_dir / 'inference.db')
db.connect()

from backend.services.chat_service import ChatService
svc = ChatService(db=db)

# Load keys like main.py does
claude_key = db.get_setting('CLAUDE_API_KEY')
if claude_key:
    svc.set_provider_api_key('claude', claude_key)
perplexity_key = db.get_setting('PERPLEXITY_API_KEY')
if perplexity_key:
    svc.set_provider_api_key('perplexity', perplexity_key)

print('Active provider:', svc.active_provider_name)
print('Claude configured:', svc._providers['claude'].is_configured)
print('Perplexity configured:', svc._providers['perplexity'].is_configured)
print('OK')
"
```

Expected: Both providers configured (since both keys exist in DB)

**Step 4: Commit**

```bash
git add backend/api/chat.py main.py
git commit -m "feat: update API layer and startup for multi-provider support"
```

---

### Task 6: Update frontend types and bridge

**Files:**
- Modify: `frontend/src/types/pywebview.d.ts`
- Modify: `frontend/src/bridge.ts`

**Step 1: Update pywebview.d.ts**

Add `provider` param to `chat_set_api_key` and add `chat_get_provider_info`:

In the `// ── Chat ──` section, change:

```typescript
chat_set_api_key(args: { api_key: string; provider?: string }): Promise<ApiResponse>;
```

Add after `chat_delete_conversation`:

```typescript
chat_get_provider_info(): Promise<ApiResponse<{
  provider?: string;
  models?: Array<{ id: string; label: string }>;
}>>;
```

**Step 2: Update bridge.ts mock**

In the `// ── Chat ──` section of `mockApi`, update `chat_set_api_key`:

```typescript
async chat_set_api_key(_args) {
  return { status: "success" };
},
```

Add `chat_get_provider_info` mock:

```typescript
async chat_get_provider_info() {
  return {
    status: "success",
    provider: "claude",
    models: [
      { id: "claude-haiku-4-5-20251001", label: "Claude Haiku 4.6 — Fast, low cost" },
      { id: "claude-sonnet-4-6", label: "Claude Sonnet 4.6 — Balanced (default)" },
      { id: "claude-opus-4-6", label: "Claude Opus 4.6 — Most capable" },
    ],
  };
},
```

**Step 3: Run type check**

```bash
cd /home/alex/Dev/Work/rzem-ai-inference/frontend && npm run type-check
```

Expected: no errors

**Step 4: Commit**

```bash
git add frontend/src/types/pywebview.d.ts frontend/src/bridge.ts
git commit -m "feat: update frontend types and bridge for multi-provider chat"
```

---

### Task 7: Update settings store

**Files:**
- Modify: `frontend/src/stores/settings.ts`

**Step 1: Add provider and perplexity model state + actions**

Add to the `state()` return object, after the `claudeModel` line:

```typescript
// AI Provider
aiProvider: 'claude' as string,

// Perplexity Model
perplexityModel: 'anthropic/claude-sonnet-4-6' as string,
```

Add these actions to the `actions` object, after the `saveClaudeModel` action:

```typescript
// ── AI Provider ──

async loadAiProvider() {
  const api = await getApiAsync();
  const res = await api.get_setting({ key: 'AI_PROVIDER' });
  if (res.status === 'success' && res.value) {
    this.aiProvider = res.value;
  }
},

async saveAiProvider(provider: string) {
  const api = await getApiAsync();
  const res = await api.set_setting({ key: 'AI_PROVIDER', value: provider });
  if (res.status === 'success') {
    this.aiProvider = provider;
  }
},

// ── Perplexity Model ──

async loadPerplexityModel() {
  const api = await getApiAsync();
  const res = await api.get_setting({ key: 'PERPLEXITY_MODEL' });
  if (res.status === 'success' && res.value) {
    this.perplexityModel = res.value;
  }
},

async savePerplexityModel(model: string) {
  const api = await getApiAsync();
  const res = await api.set_setting({ key: 'PERPLEXITY_MODEL', value: model });
  if (res.status === 'success') {
    this.perplexityModel = model;
  }
},
```

**Step 2: Type check**

```bash
cd /home/alex/Dev/Work/rzem-ai-inference/frontend && npm run type-check
```

**Step 3: Commit**

```bash
git add frontend/src/stores/settings.ts
git commit -m "feat: add AI provider and Perplexity model to settings store"
```

---

### Task 8: Update chat store

**Files:**
- Modify: `frontend/src/stores/chat.ts`

**Step 1: Update setApiKey to accept provider param**

Change the `setApiKey` action:

```typescript
async setApiKey(apiKey: string, provider: string = 'claude') {
  const api = await getApiAsync();
  const res = await api.chat_set_api_key({ api_key: apiKey, provider });
  if (res.status === 'success') {
    this.isConfigured = true;
  }
},
```

**Step 2: Type check**

```bash
cd /home/alex/Dev/Work/rzem-ai-inference/frontend && npm run type-check
```

**Step 3: Commit**

```bash
git add frontend/src/stores/chat.ts
git commit -m "feat: update chat store setApiKey with provider param"
```

---

### Task 9: Update Settings > AI page

**Files:**
- Modify: `frontend/src/pages/settings/AI.vue`

This is the main UI change — add provider selector and conditional sections.

**Step 1: Rewrite AI.vue**

Replace the full contents of `frontend/src/pages/settings/AI.vue` with:

```vue
<template>
  <div class="flex flex-col gap-6 p-4 overflow-y-auto">
    <div>
      <div class="text-xl font-semibold text-slate-900 mb-1">AI</div>
      <div class="text-base text-slate-500">
        Configure the AI provider, model, and scan button prompts used by the AI Assistant.
      </div>
    </div>

    <!-- Provider selector -->
    <Card>
      <template #title>
        <div class="flex items-center gap-2">
          <Zap :size="16" class="text-amber-500" />
          AI Provider
        </div>
      </template>
      <template #content>
        <p class="text-muted-color mb-3">Choose which AI service powers the assistant.</p>
        <SelectButton
          v-model="localProvider"
          :options="providerOptions"
          option-label="label"
          option-value="value"
          @change="onProviderChange" />
      </template>
    </Card>

    <!-- Claude config -->
    <Card v-if="localProvider === 'claude'">
      <template #title>
        <div class="flex items-center gap-2">
          <Cpu :size="16" class="text-blue-500" />
          Claude Configuration
        </div>
      </template>
      <template #content>
        <div class="flex flex-col gap-4">
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">API Key</label>
            <div class="flex gap-2">
              <InputText
                v-model="claudeApiKey"
                type="password"
                class="flex-1"
                placeholder="sk-ant-..." />
              <Button label="Save" severity="primary" @click="saveClaudeKey" :disabled="!claudeApiKey.trim()" />
            </div>
            <p class="text-xs text-slate-400 mt-1">Your Anthropic API key. Stored locally.</p>
          </div>
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">Model</label>
            <Select
              v-model="localClaudeModel"
              :options="claudeModelOptions"
              option-label="label"
              option-value="value"
              class="w-full"
              @change="saveClaudeModel" />
          </div>
        </div>
      </template>
    </Card>

    <!-- Perplexity config -->
    <Card v-if="localProvider === 'perplexity'">
      <template #title>
        <div class="flex items-center gap-2">
          <Globe :size="16" class="text-violet-500" />
          Perplexity Configuration
        </div>
      </template>
      <template #content>
        <div class="flex flex-col gap-4">
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">API Key</label>
            <div class="flex gap-2">
              <InputText
                v-model="perplexityApiKey"
                type="password"
                class="flex-1"
                placeholder="pplx-..." />
              <Button label="Save" severity="primary" @click="savePerplexityKey" :disabled="!perplexityApiKey.trim()" />
            </div>
            <p class="text-xs text-slate-400 mt-1">Your Perplexity API key. Stored locally.</p>
          </div>
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">Model</label>
            <Select
              v-model="localPerplexityModel"
              :options="perplexityModelOptions"
              option-label="label"
              option-value="value"
              class="w-full"
              @change="savePerplexityModel" />
          </div>
        </div>
      </template>
    </Card>

    <Message severity="secondary" :closable="false">
      <template #messageicon><Info :size="16" /></template>
      <div class="text-sm leading-relaxed">
        <span class="font-semibold">How prompts work:</span> Each prompt is sent alongside a reference image. The prompt should tell the AI
        what to analyze in the image, then instruct it to update your generation prompt. The AI has two tools it can call:
        <span class="font-semibold">update prompt</span> and <span class="font-semibold">update generation settings</span>
        (dimensions, steps, cfg scale, seed). Without an explicit instruction like "Then update my prompt to...",
        the AI will only describe the image without modifying anything.
      </div>
    </Message>

    <Card v-for="entry in promptEntries" :key="entry.key">
      <template #title>
        <div class="flex items-center gap-2">
          <component :is="entry.icon" :size="16" :class="entry.iconClass" />
          {{ entry.label }}
        </div>
      </template>
      <template #content>
        <div class="flex flex-col gap-4">
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">Display Text</label>
            <InputText
              v-model="local[entry.key].displayText"
              class="w-full"
              :placeholder="`Text shown in chat for ${entry.label}`"
              @change="saveDisplayText(entry.key)" />
          </div>
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">Prompt</label>
            <Textarea
              v-model="local[entry.key].prompt"
              class="w-full"
              rows="4"
              auto-resize
              :placeholder="`Full prompt sent to the AI for ${entry.label}`"
              @change="savePrompt(entry.key)" />
          </div>
        </div>
      </template>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { useChatStore } from '@/stores/chat';
import { Box, Cpu, Globe, Info, Layers, Paintbrush, Zap } from 'lucide-vue-next';

const settingsStore = useSettingsStore();
const chatStore = useChatStore();

const providerOptions = [
  { label: 'Claude', value: 'claude' },
  { label: 'Perplexity', value: 'perplexity' },
];

const claudeModelOptions = [
  { label: 'Claude Haiku 4.6 — Fast, low cost', value: 'claude-haiku-4-5-20251001' },
  { label: 'Claude Sonnet 4.6 — Balanced (default)', value: 'claude-sonnet-4-6' },
  { label: 'Claude Opus 4.6 — Most capable', value: 'claude-opus-4-6' },
];

const perplexityModelOptions = [
  { label: 'Sonar — Fast, web search', value: 'perplexity/sonar' },
  { label: 'GPT-5.2 — Frontier', value: 'openai/gpt-5.2' },
  { label: 'GPT-5.1 — Mid-tier', value: 'openai/gpt-5.1' },
  { label: 'GPT-5 Mini — Budget', value: 'openai/gpt-5-mini' },
  { label: 'Claude Sonnet 4.6 — Balanced (default)', value: 'anthropic/claude-sonnet-4-6' },
  { label: 'Claude Haiku 4.5 — Fast', value: 'anthropic/claude-haiku-4-5' },
  { label: 'Gemini 2.5 Pro', value: 'google/gemini-2.5-pro' },
  { label: 'Gemini 2.5 Flash — Budget', value: 'google/gemini-2.5-flash' },
];

const localProvider = ref('claude');
const claudeApiKey = ref('');
const perplexityApiKey = ref('');
const localClaudeModel = ref('claude-sonnet-4-6');
const localPerplexityModel = ref('anthropic/claude-sonnet-4-6');

async function onProviderChange() {
  await settingsStore.saveAiProvider(localProvider.value);
  await chatStore.checkConfigured();
}

async function saveClaudeKey() {
  if (!claudeApiKey.value.trim()) return;
  await chatStore.setApiKey(claudeApiKey.value.trim(), 'claude');
  claudeApiKey.value = '';
  await chatStore.checkConfigured();
}

async function savePerplexityKey() {
  if (!perplexityApiKey.value.trim()) return;
  await chatStore.setApiKey(perplexityApiKey.value.trim(), 'perplexity');
  perplexityApiKey.value = '';
  await chatStore.checkConfigured();
}

async function saveClaudeModel() {
  await settingsStore.saveClaudeModel(localClaudeModel.value);
}

async function savePerplexityModel() {
  await settingsStore.savePerplexityModel(localPerplexityModel.value);
}

const promptEntries = [
  { key: 'style', label: 'Style', icon: Paintbrush, iconClass: 'text-purple-500' },
  { key: 'both', label: 'Style + Subject', icon: Layers, iconClass: 'text-blue-500' },
  { key: 'subject', label: 'Subject', icon: Box, iconClass: 'text-green-500' },
];

const local = reactive({
  style: { prompt: '', displayText: '' },
  both: { prompt: '', displayText: '' },
  subject: { prompt: '', displayText: '' },
} as Record<string, { prompt: string; displayText: string }>);

function syncFromStore() {
  for (const key of ['style', 'both', 'subject']) {
    local[key].prompt = settingsStore.aiPrompts[key].prompt;
    local[key].displayText = settingsStore.aiPrompts[key].displayText;
  }
}

async function savePrompt(key: string) {
  await settingsStore.saveAiPrompt(key, local[key].prompt);
}

async function saveDisplayText(key: string) {
  await settingsStore.saveAiDisplayText(key, local[key].displayText);
}

onMounted(async () => {
  await settingsStore.loadAiProvider();
  localProvider.value = settingsStore.aiProvider;
  await settingsStore.loadClaudeModel();
  localClaudeModel.value = settingsStore.claudeModel;
  await settingsStore.loadPerplexityModel();
  localPerplexityModel.value = settingsStore.perplexityModel;
  await settingsStore.loadAiPrompts();
  syncFromStore();
});
</script>
```

**Step 2: Type check**

```bash
cd /home/alex/Dev/Work/rzem-ai-inference/frontend && npm run type-check
```

**Step 3: Commit**

```bash
git add frontend/src/pages/settings/AI.vue
git commit -m "feat: update Settings > AI page with provider selector and dual config"
```

---

### Task 10: Update ChatbotPanel for provider-aware key prompt

**Files:**
- Modify: `frontend/src/pages/create/ChatbotPanel.vue`

**Step 1: Update the not-configured state**

The "not configured" section needs to show the correct provider name and placeholder. Change lines 17-34 of `ChatbotPanel.vue`:

Replace the entire not-configured `<div>` (the `v-if="!chatStore.isConfigured"` block) with:

```vue
<div v-if="!chatStore.isConfigured" class="flex-1 flex flex-col items-center justify-center px-6 gap-3">
  <KeyRound :size="24" class="text-slate-300" />
  <p class="text-xs text-slate-500 text-center">
    Enter your {{ providerLabel }} API key to enable the AI assistant.
  </p>
  <div class="flex gap-1 w-full">
    <input
      v-model="apiKeyInput"
      type="password"
      :placeholder="providerPlaceholder"
      class="flex-1 text-xs bg-slate-100 rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-blue-300"
      @keydown.enter="onSetApiKey" />
    <button
      class="px-3 py-2 bg-blue-500 text-white text-xs rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50"
      :disabled="!apiKeyInput.trim()"
      @click="onSetApiKey">
      Save
    </button>
  </div>
</div>
```

In `<script setup>`, add these computed properties after the existing imports:

```typescript
import { computed, ref, watch, nextTick } from 'vue';
```

(Already imported — just add the computed.)

After `const apiKeyInput = ref('');` add:

```typescript
const providerLabel = computed(() => {
  return settingsStore.aiProvider === 'perplexity' ? 'Perplexity' : 'Claude';
});

const providerPlaceholder = computed(() => {
  return settingsStore.aiProvider === 'perplexity' ? 'pplx-...' : 'sk-ant-...';
});
```

Update `onSetApiKey` to pass the current provider:

```typescript
async function onSetApiKey() {
  if (!apiKeyInput.value.trim()) return;
  await chatStore.setApiKey(apiKeyInput.value.trim(), settingsStore.aiProvider);
  apiKeyInput.value = '';
}
```

**Step 2: Type check**

```bash
cd /home/alex/Dev/Work/rzem-ai-inference/frontend && npm run type-check
```

**Step 3: Commit**

```bash
git add frontend/src/pages/create/ChatbotPanel.vue
git commit -m "feat: update ChatbotPanel with provider-aware API key prompt"
```

---

### Task 11: Manual integration test

**Step 1: Run the app**

```bash
cd /home/alex/Dev/Work/rzem-ai-inference
bash scripts/dev.sh
```

In another terminal:
```bash
cd /home/alex/Dev/Work/rzem-ai-inference/frontend && npm run dev
```

**Step 2: Test Claude provider (existing behavior)**

1. Open Settings > AI
2. Verify "Claude" is selected as the provider
3. Verify Claude model dropdown shows the 3 Claude models
4. Open the chat panel and send a message — verify streaming works
5. Test a scan button (Style/Subject) — verify tool use works

**Step 3: Test Perplexity provider**

1. Go to Settings > AI
2. Switch to "Perplexity"
3. Verify Perplexity model dropdown shows the 8 models
4. The Perplexity API key should already be configured (added to DB earlier)
5. Open the chat panel and send "What are the latest trends in AI art generation?"
6. Verify a response comes back (non-streaming, delivered as single text block)
7. Test: "Set my prompt to: a majestic dragon in a crystal cave" — verify the model outputs a JSON tool block and the prompt updates

**Step 4: Test provider switching**

1. Switch back to Claude in Settings
2. Verify chat works with Claude again
3. Switch to Perplexity, verify it still works

**Step 5: Commit if any fixes needed**

```bash
git add -A
git commit -m "fix: integration test fixes for multi-provider chat"
```
