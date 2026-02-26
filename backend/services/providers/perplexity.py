"""Perplexity chat provider — responses.create() API with frontier model access."""

from __future__ import annotations

import json
import logging
import re
from typing import Any, Iterator

from backend.services.providers import ProviderModel, StreamEvent

logger = logging.getLogger(__name__)

PERPLEXITY_MODELS = [
    ProviderModel(id="perplexity/sonar", label="Perplexity Sonar"),
    ProviderModel(id="openai/gpt-5.2", label="GPT 5.2"),
    ProviderModel(id="openai/gpt-5.1", label="GPT 5.1"),
    ProviderModel(id="openai/gpt-5-mini", label="GPT 5 Mini"),
    ProviderModel(id="anthropic/claude-sonnet-4-6", label="Claude Sonnet 4.6"),
    ProviderModel(id="anthropic/claude-haiku-4-5", label="Claude Haiku 4.5"),
    ProviderModel(id="google/gemini-2.5-pro", label="Gemini 2.5 Pro"),
    ProviderModel(id="google/gemini-2.5-flash", label="Gemini 2.5 Flash"),
]

# Regex to find ```json blocks containing a "tool" key
_TOOL_BLOCK_RE = re.compile(
    r"```json\s*\n(\{[^`]*?\"tool\"\s*:[^`]*?\})\s*\n```",
    re.DOTALL,
)


class PerplexityProvider:
    """Perplexity responses API provider — no native tool use, uses text-based JSON blocks."""

    name: str = "perplexity"

    def __init__(self) -> None:
        self._client = None  # perplexity.Perplexity, lazy-imported

    def configure(self, api_key: str) -> None:
        import perplexity

        self._client = perplexity.Perplexity(api_key=api_key)
        logger.info("Perplexity provider configured")

    @property
    def is_configured(self) -> bool:
        return self._client is not None

    def get_models(self) -> list[ProviderModel]:
        return list(PERPLEXITY_MODELS)

    def supports_tools(self, model: str) -> bool:
        # Perplexity responses API only supports built-in tools (web_search),
        # not custom function calling. We use text-based JSON blocks instead.
        return False

    def stream(
        self,
        messages: list[dict[str, Any]],
        system_prompt: str,
        tools: list[dict[str, Any]],
        model: str,
    ) -> Iterator[StreamEvent]:
        """Call the Perplexity responses API and yield normalized StreamEvents.

        Since Perplexity doesn't support native function calling for custom tools,
        the system prompt instructs the model to output JSON blocks. We parse those
        and emit tool_call events alongside the cleaned text.
        """
        if not self._client:
            raise RuntimeError("Perplexity provider not configured")

        input_text = self._flatten_messages(messages)

        response = self._client.responses.create(
            model=model,
            input=input_text,
            instructions=system_prompt,
        )

        output_text = response.output_text or ""

        # Parse any embedded tool JSON blocks
        tool_blocks = self._parse_tool_blocks(output_text)

        if tool_blocks:
            # Strip tool blocks from displayed text
            clean_text = _TOOL_BLOCK_RE.sub("", output_text).strip()
            if clean_text:
                yield StreamEvent(type="text_delta", text=clean_text)
            for block in tool_blocks:
                tool_name = block.pop("tool", None)
                if tool_name:
                    yield StreamEvent(
                        type="tool_call",
                        tool_id=None,
                        tool_name=tool_name,
                        tool_input=block,
                    )
        else:
            if output_text:
                yield StreamEvent(type="text_delta", text=output_text)

        yield StreamEvent(type="done")

    def _flatten_messages(self, messages: list[dict[str, Any]]) -> str:
        """Flatten conversation messages into a single input string.

        Handles both simple string content and Claude-style content block lists.
        """
        if not messages:
            return ""

        if len(messages) == 1:
            return self._extract_text(messages[0])

        # Multiple messages: format previous as context, latest as the input
        parts = ["Previous conversation:"]
        for msg in messages[:-1]:
            role = msg.get("role", "user").capitalize()
            text = self._extract_text(msg)
            if text:
                parts.append(f"{role}: {text}")

        parts.append("---\n")
        parts.append(self._extract_text(messages[-1]))

        return "\n".join(parts)

    @staticmethod
    def _extract_text(message: dict[str, Any]) -> str:
        """Extract plain text from a message, handling both string and content-block formats."""
        content = message.get("content", "")
        if isinstance(content, str):
            return content
        if isinstance(content, list):
            text_parts = []
            for block in content:
                if isinstance(block, dict):
                    if block.get("type") == "text":
                        text_parts.append(block.get("text", ""))
                    elif block.get("type") == "tool_result":
                        text_parts.append(block.get("content", ""))
                elif isinstance(block, str):
                    text_parts.append(block)
            return " ".join(text_parts)
        return str(content)

    @staticmethod
    def _parse_tool_blocks(text: str) -> list[dict[str, Any]]:
        """Parse ```json blocks containing a 'tool' key from the model output."""
        results: list[dict[str, Any]] = []
        for match in _TOOL_BLOCK_RE.finditer(text):
            try:
                parsed = json.loads(match.group(1))
                if isinstance(parsed, dict) and "tool" in parsed:
                    results.append(parsed)
            except (json.JSONDecodeError, TypeError):
                continue
        return results
