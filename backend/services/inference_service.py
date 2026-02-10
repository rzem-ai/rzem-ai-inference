"""Thin wrapper around InferenceEngine that collects events for frontend polling."""

from __future__ import annotations

import logging
import threading
from collections import deque
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from rzem_ai_inference_engine import (
    CompletedEvent,
    EventType,
    FailedEvent,
    InferenceEngine,
    JobParams,
    ProgressEvent,
)
from rzem_ai_inference_engine.types import PreviewConfig

logger = logging.getLogger(__name__)


@dataclass
class FrontendEvent:
    """Serializable event for the frontend."""
    type: str
    data: dict[str, Any]


class InferenceService:
    """Manages the InferenceEngine lifecycle and buffers events for frontend polling.

    Events from the engine's background thread are collected into a thread-safe
    deque. The frontend polls ``drain_events()`` to retrieve them.
    """

    def __init__(self, output_dir: Path) -> None:
        self._engine: InferenceEngine | None = None
        self._events: deque[FrontendEvent] = deque(maxlen=500)
        self._lock = threading.Lock()
        self._output_dir = output_dir
        self._output_dir.mkdir(parents=True, exist_ok=True)

    @property
    def ready(self) -> bool:
        return self._engine is not None

    def start(self, device: str = "auto", vram_limit_gb: float | None = None) -> None:
        """Initialize the inference engine and subscribe to all events."""
        if self._engine is not None:
            return

        self._engine = InferenceEngine(
            device=device,
            vram_limit_gb=vram_limit_gb,
            preview_config=PreviewConfig(enabled=True, interval=5, max_size=256),
        )

        for event_type in EventType:
            self._engine.on(event_type, self._make_handler(event_type))

    def shutdown(self) -> None:
        if self._engine is not None:
            self._engine.shutdown()
            self._engine = None

    def submit(self, params: JobParams) -> str:
        """Submit a generation job. Returns the job ID."""
        if not self._engine:
            raise RuntimeError("Engine not started")
        return self._engine.submit(params)

    def cancel(self, job_id: str) -> None:
        if not self._engine:
            raise RuntimeError("Engine not started")
        self._engine.cancel(job_id)

    def drain_events(self) -> list[dict[str, Any]]:
        """Return and clear all buffered events. Thread-safe."""
        with self._lock:
            events = list(self._events)
            self._events.clear()
        return [{"type": e.type, "data": e.data} for e in events]

    # ── Internal ──────────────────────────────────────────────────────

    def _make_handler(self, event_type: EventType):
        """Create a handler that serializes events into the buffer."""
        def handler(event_data):
            try:
                fe = self._serialize(event_type, event_data)
                with self._lock:
                    self._events.append(fe)
            except Exception:
                logger.exception("Failed to serialize %s event", event_type.value)
        return handler

    def _save_image(self, image, job_id: str, suffix: str) -> str:
        """Save a PIL image to disk and return the file path."""
        filename = f"{job_id}_{suffix}.png"
        path = self._output_dir / filename
        image.save(str(path), format="PNG")
        logger.info("Saved %s to %s", suffix, path)
        return str(path)

    def _serialize(self, event_type: EventType, event_data: Any) -> FrontendEvent:
        """Convert engine event dataclasses to JSON-safe dicts."""
        data: dict[str, Any] = {}
        if event_data is not None:
            raw = asdict(event_data) if hasattr(event_data, "__dataclass_fields__") else {}
            if event_type == EventType.JOB_PROGRESS:
                has_preview = raw.get("preview_image") is not None
                logger.info("job_progress step=%s preview_image=%s", raw.get("step"), has_preview)
            for key, val in raw.items():
                if key == "image" and val is not None:
                    # Save completed image to disk, store path
                    data["image_path"] = self._save_image(
                        event_data.image, raw.get("job_id", "unknown"), "output"
                    )
                elif key == "preview_image" and val is not None:
                    # Save preview image to disk, store path
                    data["preview_path"] = self._save_image(
                        event_data.preview_image, raw.get("job_id", "unknown"),
                        f"preview_{raw.get('step', 0)}"
                    )
                elif key in ("image", "preview_image"):
                    # Field is None — skip it
                    pass
                else:
                    data[key] = val
        return FrontendEvent(type=event_type.value, data=data)
