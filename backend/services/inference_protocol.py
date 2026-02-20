"""Shared types and protocol for inference service implementations."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Protocol, runtime_checkable


@dataclass
class FrontendEvent:
    """Serializable event for the frontend."""
    type: str
    data: dict[str, Any]


@runtime_checkable
class InferenceServiceProtocol(Protocol):
    """Interface that both local and remote inference services implement.

    The manager delegates all calls to whichever service is active.
    Both implementations buffer events into deque[FrontendEvent] so the
    frontend's polling loop works identically in local and remote modes.
    """

    @property
    def ready(self) -> bool: ...

    def start(self, device: str = "auto", vram_limit_gb: float | None = None) -> None: ...

    def shutdown(self) -> None: ...

    def submit(self, params: Any, bundle_id: str | None = None) -> str: ...

    def cancel(self, job_id: str) -> None: ...

    def drain_events(self) -> list[dict[str, Any]]: ...

    def get_engine_status(self) -> dict[str, Any]: ...

    def get_gpu_info(self) -> dict[str, Any]: ...

    def set_output_dir(self, output_dir: Path) -> None: ...
