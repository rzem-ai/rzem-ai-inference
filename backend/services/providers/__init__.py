"""Chat provider abstraction — protocol and shared types."""

from __future__ import annotations

from dataclasses import dataclass
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

    def complete(
        self,
        messages: list[dict[str, Any]],
        system_prompt: str,
        model: str,
        max_tokens: int = 1024,
    ) -> str: ...

    def get_models(self) -> list[ProviderModel]: ...

    def supports_tools(self, model: str) -> bool: ...

    def supports_vision(self) -> bool: ...
