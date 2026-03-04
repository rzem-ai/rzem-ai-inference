"""Claude chat provider — Anthropic Messages API with native tool use."""

from __future__ import annotations

import json
import logging
from typing import Any, Iterator

from backend.services.providers import ProviderModel, StreamEvent
from backend.tracing import trace_class

logger = logging.getLogger(__name__)

CLAUDE_MODELS = [
    ProviderModel(id="claude-haiku-4-5-20251001", label="Claude Haiku 4.5"),
    ProviderModel(id="claude-sonnet-4-6", label="Claude Sonnet 4.6"),
    ProviderModel(id="claude-opus-4-6", label="Claude Opus 4.6"),
]


@trace_class
class ClaudeProvider:
    """Anthropic Messages API provider with streaming and native tool use."""

    name: str = "claude"

    def __init__(self) -> None:
        self._client = None  # anthropic.Anthropic, lazy-imported

    def configure(self, api_key: str) -> None:
        import anthropic

        from backend.ssl_utils import is_ssl_verification_disabled

        kwargs: dict[str, Any] = {"api_key": api_key}
        if is_ssl_verification_disabled():
            import httpx

            kwargs["http_client"] = httpx.Client(verify=False)
            logger.info("Claude provider using SSL-disabled httpx client")

        self._client = anthropic.Anthropic(**kwargs)
        logger.info("Claude provider configured")

    @property
    def is_configured(self) -> bool:
        return self._client is not None

    def get_models(self) -> list[ProviderModel]:
        return list(CLAUDE_MODELS)

    def supports_tools(self, model: str) -> bool:
        return True

    def supports_vision(self) -> bool:
        return True

    def complete(
        self,
        messages: list[dict[str, Any]],
        system_prompt: str,
        model: str,
        max_tokens: int = 1024,
    ) -> str:
        """One-shot completion (non-streaming). Returns the full text response."""
        if not self._client:
            raise RuntimeError("Claude provider not configured")

        response = self._client.messages.create(
            model=model,
            max_tokens=max_tokens,
            system=system_prompt,
            messages=messages,
        )
        return response.content[0].text

    def stream(
        self,
        messages: list[dict[str, Any]],
        system_prompt: str,
        tools: list[dict[str, Any]],
        model: str,
    ) -> Iterator[StreamEvent]:
        """Stream a Claude response, yielding normalized StreamEvents.

        Converts Anthropic-native events:
        - content_block_start (tool_use) -> accumulate tool
        - content_block_delta (text_delta) -> yield text_delta StreamEvent
        - content_block_delta (input_json_delta) -> accumulate partial JSON
        - content_block_stop for tool -> yield tool_call StreamEvent
        - End of stream -> yield done StreamEvent
        """
        if not self._client:
            raise RuntimeError("Claude provider not configured")

        tool_uses: list[dict[str, Any]] = []

        with self._client.messages.stream(
            model=model,
            max_tokens=1024,
            system=system_prompt,
            messages=messages,
            tools=tools,
        ) as api_stream:
            for event in api_stream:
                if event.type == "content_block_start":
                    if event.content_block.type == "tool_use":
                        tool_uses.append({
                            "id": event.content_block.id,
                            "name": event.content_block.name,
                            "input_json": "",
                        })
                elif event.type == "content_block_delta":
                    if event.delta.type == "text_delta":
                        yield StreamEvent(type="text_delta", text=event.delta.text)
                    elif event.delta.type == "input_json_delta":
                        if tool_uses:
                            tool_uses[-1]["input_json"] += event.delta.partial_json
                elif event.type == "content_block_stop":
                    # If the last accumulated block was a tool, yield it
                    if tool_uses and tool_uses[-1].get("input_json") is not None:
                        tool = tool_uses[-1]
                        tool_input = (
                            json.loads(tool["input_json"])
                            if tool["input_json"]
                            else {}
                        )
                        yield StreamEvent(
                            type="tool_call",
                            tool_id=tool["id"],
                            tool_name=tool["name"],
                            tool_input=tool_input,
                        )

        yield StreamEvent(type="done")

    def build_tool_result_messages(
        self,
        full_text: str,
        tool_uses: list[dict[str, Any]],
        tool_results: list[dict[str, Any]],
    ) -> list[dict[str, Any]]:
        """Build Claude-specific assistant + user messages for the tool result loop.

        Claude requires content blocks with tool_use and tool_result types
        to continue the conversation after tool calls.

        Args:
            full_text: Any text the assistant produced before/alongside tool calls.
            tool_uses: List of dicts with id, name, tool_input for each tool call.
            tool_results: List of dicts with tool_use_id and content for each result.

        Returns:
            Two messages: [assistant_msg, user_msg] to append to conversation.
        """
        assistant_content: list[dict[str, Any]] = []
        if full_text:
            assistant_content.append({"type": "text", "text": full_text})
        for tool in tool_uses:
            assistant_content.append({
                "type": "tool_use",
                "id": tool["id"],
                "name": tool["name"],
                "input": tool["tool_input"],
            })

        user_content = [
            {
                "type": "tool_result",
                "tool_use_id": r["tool_use_id"],
                "content": r["content"],
            }
            for r in tool_results
        ]

        return [
            {"role": "assistant", "content": assistant_content},
            {"role": "user", "content": user_content},
        ]
