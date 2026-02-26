# Multi-Provider Chat Design

## Goal

Add Perplexity as an alternative AI chat provider alongside Claude, with a provider abstraction layer that makes ChatService provider-agnostic. Global provider setting in Settings > AI.

## Architecture

Provider abstraction protocol normalizes streaming events between Anthropic and Perplexity SDKs. ChatService becomes a thin orchestrator that delegates streaming to the active provider and handles persistence, event buffering, and tool execution uniformly.

```
ChatService (orchestrator)
  ├── ClaudeProvider        → anthropic SDK, messages.stream()
  ├── PerplexityProvider    → perplexity SDK, responses.create()
  │
  ├── _push_event()         → thread-safe event buffer (unchanged)
  ├── _build_messages()     → conversation history from DB (unchanged)
  ├── _execute_tool()       → tool result logic (unchanged)
  └── _tool_use_loop()      → refactored to use provider.stream()
```

## Provider Protocol

```python
@dataclass
class StreamEvent:
    type: str           # "text_delta" | "tool_call" | "done"
    text: str | None = None
    tool_id: str | None = None
    tool_name: str | None = None
    tool_input: dict | None = None

class ChatProvider(Protocol):
    name: str
    def configure(self, api_key: str) -> None: ...
    def is_configured(self) -> bool: ...
    def stream(self, messages, system_prompt, tools, model) -> Iterator[StreamEvent]: ...
    def get_models(self) -> list[dict]: ...
    def supports_tools(self, model: str) -> bool: ...
```

## ClaudeProvider

Extracted from current `chat_service.py`. Uses `anthropic.Anthropic` with `messages.stream()`. Converts Anthropic's `content_block_start`, `content_block_delta`, `text_delta`, `input_json_delta` into `StreamEvent` objects. Full tool use on all models.

Models:
- `claude-haiku-4-5-20251001` — Fast, low cost
- `claude-sonnet-4-6` — Balanced (default)
- `claude-opus-4-6` — Most capable

## PerplexityProvider

Uses `perplexity.Perplexity` with `responses.create()`. System prompt via `instructions` parameter. Conversation history flattened into `input`.

Tool use strategy:
- Include `update_prompt` and `update_generation_settings` as function definitions where model supports it
- Models without function calling get tool descriptions in `instructions` with structured JSON format
- `supports_tools(model)` returns True for models known to handle function calling

Non-streaming fallback: if Responses API doesn't support streaming, send full response as single `text_delta` + `done`.

Models:
- `perplexity/sonar` — Fast, web search
- `openai/gpt-5.2` — Frontier
- `openai/gpt-5.1` — Mid-tier
- `openai/gpt-5-mini` — Budget
- `anthropic/claude-sonnet-4-6` — Via Perplexity
- `anthropic/claude-haiku-4-5` — Via Perplexity
- `google/gemini-2.5-pro` — Via Perplexity
- `google/gemini-2.5-flash` — Budget

Default model: `anthropic/claude-sonnet-4-6`

## Settings

Database keys:
- `AI_PROVIDER` — `"claude"` or `"perplexity"` (default: `"claude"`)
- `CLAUDE_API_KEY` — existing
- `CLAUDE_MODEL` — existing (default: `claude-sonnet-4-6`)
- `PERPLEXITY_API_KEY` — already in DB
- `PERPLEXITY_MODEL` — default: `anthropic/claude-sonnet-4-6`

## Settings UI

1. Provider selector at top (Claude / Perplexity segmented toggle)
2. Conditional provider section: API key + model dropdown per provider
3. AI Prompts section unchanged (provider-agnostic)

## ChatbotPanel

- API key prompt reflects active provider name
- `chat_is_configured()` checks active provider

## API Layer

- `chat_set_api_key(api_key, provider)` — explicit provider param
- `chat_is_configured()` — checks active provider

## Initialization (main.py)

Load both API keys on startup, configure both providers if keys exist. Active provider determined by `AI_PROVIDER` setting.

## File Layout

New:
- `backend/services/providers/__init__.py` — protocol + StreamEvent
- `backend/services/providers/claude.py` — ClaudeProvider
- `backend/services/providers/perplexity.py` — PerplexityProvider

Refactored:
- `backend/services/chat_service.py` — thin orchestrator

Modified:
- `backend/api/chat.py` — provider param on set_api_key
- `main.py` — init both providers
- `frontend/src/pages/settings/AI.vue` — provider selector, dual config
- `frontend/src/stores/settings.ts` — new actions
- `frontend/src/stores/chat.ts` — minor
- `frontend/src/pages/create/ChatbotPanel.vue` — provider-aware key prompt
- `frontend/src/types/inference.ts` — types

Dependencies:
- Add `perplexityai` to pyproject.toml

No database migration needed — settings are key-value pairs.
