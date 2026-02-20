"""Thin wrapper around InferenceEngine that collects events for frontend polling."""

from __future__ import annotations

import json
import logging
import os
import threading
import time
import uuid
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
from rzem_ai_inference_engine.models.memory import detect_device, get_total_vram
from rzem_ai_inference_engine.types import PreviewConfig

from backend.db.database import Database
from backend.services.inference_protocol import FrontendEvent

logger = logging.getLogger(__name__)


class LocalInferenceService:
    """Manages the InferenceEngine lifecycle and buffers events for frontend polling.

    Events from the engine's background thread are collected into a thread-safe
    deque. The frontend polls ``drain_events()`` to retrieve them.
    """

    def __init__(self, output_dir: Path, db: Database) -> None:
        self._engine: InferenceEngine | None = None
        self._events: deque[FrontendEvent] = deque(maxlen=500)
        self._lock = threading.Lock()
        self.set_output_dir(output_dir)
        self._db = db
        self._job_params: dict[str, JobParams] = {}  # job_id → params used
        self._job_start_times: dict[str, float] = {}  # job_id → monotonic start
        self._job_bundle_ids: dict[str, str] = {}  # job_id → bundle_id
        self._job_style_ids: dict[str, str] = {}  # job_id → style_id
        self._job_raw_prompts: dict[str, str] = {}  # job_id → raw user prompt
        self._start_time: float | None = None  # monotonic time engine started
        self._completed_count: int = 0  # total jobs completed this session

    def set_output_dir(self, output_dir: Path) -> None:
        """Update the output directory, creating it if needed."""
        self._output_dir = output_dir
        self._output_dir.mkdir(parents=True, exist_ok=True)

    @property
    def ready(self) -> bool:
        return self._engine is not None

    def get_gpu_info(self) -> dict[str, Any]:
        """Detect GPU device and return its capabilities."""
        try:
            device = detect_device()
            total_bytes = get_total_vram(device)
            total_gb = round(total_bytes / (1024 ** 3), 1)

            device_name = None
            if device.type == "cuda":
                import torch
                device_name = torch.cuda.get_device_name(device)

            return {
                "device_type": device.type,
                "device_name": device_name,
                "total_vram_gb": total_gb,
            }
        except Exception:
            logger.exception("Failed to detect GPU info")
            return {
                "device_type": "cpu",
                "device_name": None,
                "total_vram_gb": 0,
            }

    def _load_preview_config(self) -> PreviewConfig:
        """Read preview settings from the database, falling back to defaults."""
        interval = 5
        max_size = 256
        raw = self._db.get_setting("PREVIEW_INTERVAL")
        if raw is not None:
            try:
                interval = max(1, int(raw))
            except (ValueError, TypeError):
                pass
        raw = self._db.get_setting("PREVIEW_MAX_SIZE")
        if raw is not None:
            try:
                max_size = max(64, int(raw))
            except (ValueError, TypeError):
                pass
        return PreviewConfig(enabled=True, interval=interval, max_size=max_size)

    def start(self, device: str = "auto", vram_limit_gb: float | None = None) -> None:
        """Initialize the inference engine and subscribe to all events."""
        if self._engine is not None:
            return

        self._engine = InferenceEngine(
            device=device,
            vram_limit_gb=vram_limit_gb,
            preview_config=self._load_preview_config(),
        )
        self._start_time = time.monotonic()

        for event_type in EventType:
            self._engine.on(event_type, self._make_handler(event_type))

    def shutdown(self) -> None:
        if self._engine is not None:
            self._engine.shutdown()
            self._engine = None
            self._start_time = None

    def get_engine_status(self) -> dict[str, Any]:
        """Return engine readiness, uptime, and completed job count."""
        uptime_seconds = None
        if self._start_time is not None:
            uptime_seconds = int(time.monotonic() - self._start_time)
        return {
            "ready": self._engine is not None,
            "uptime_seconds": uptime_seconds,
            "completed_count": self._completed_count,
        }

    def submit(
        self,
        params: JobParams,
        bundle_id: str | None = None,
        style_id: str | None = None,
        raw_prompt: str | None = None,
    ) -> str:
        """Submit a generation job. Returns the job ID."""
        if not self._engine:
            raise RuntimeError("Engine not started")
        job_id = self._engine.submit(params)
        self._job_params[job_id] = params
        self._job_start_times[job_id] = time.monotonic()
        if bundle_id:
            self._job_bundle_ids[job_id] = bundle_id
        if style_id:
            self._job_style_ids[job_id] = style_id
        if raw_prompt is not None:
            self._job_raw_prompts[job_id] = raw_prompt
        return job_id

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

        # Enrich progress/completed events with job dimensions
        if event_type in (EventType.JOB_PROGRESS, EventType.JOB_COMPLETED):
            job_id = data.get("job_id")
            params = self._job_params.get(job_id) if job_id else None
            if params:
                data["width"] = params.width
                data["height"] = params.height

        # Persist completed images to the database and clean up previews
        if event_type == EventType.JOB_COMPLETED and data.get("image_path"):
            self._completed_count += 1
            self._persist_image(data)
            self._cleanup_previews(data.get("job_id", "unknown"))

        return FrontendEvent(type=event_type.value, data=data)

    def _persist_image(self, event_data: dict[str, Any]) -> None:
        """Insert a completed image record into the database."""
        job_id = event_data.get("job_id", "unknown")
        params = self._job_params.pop(job_id, None)
        start_time = self._job_start_times.pop(job_id, None)

        generation_time_ms = None
        if start_time is not None:
            generation_time_ms = int((time.monotonic() - start_time) * 1000)

        file_path = event_data["image_path"]
        file_size = None
        if os.path.isfile(file_path):
            file_size = os.path.getsize(file_path)

        # Build model_config JSON snapshot from the job params
        model_config = None
        loras_json = None
        bundle_id = self._job_bundle_ids.pop(job_id, None)
        style_id = self._job_style_ids.pop(job_id, None)
        raw_prompt = self._job_raw_prompts.pop(job_id, None)
        if params:
            config_dict = {
                "transformer_model": params.transformer_model,
                "transformer_type": params.transformer_type.value if hasattr(params.transformer_type, "value") else str(params.transformer_type),
                "vae_model": params.vae_model,
                "clip_tokenizer": params.clip_tokenizer,
                "clip_encoder": params.clip_encoder,
                "t5_tokenizer": params.t5_tokenizer,
                "t5_encoder": params.t5_encoder,
                "qwen3_tokenizer": params.qwen3_tokenizer,
                "qwen3_encoder": params.qwen3_encoder,
                "sampler": params.sampler,
                "scheduler": params.scheduler,
            }
            if params.fal_endpoint:
                config_dict["source"] = "cloud"
                config_dict["fal_endpoint"] = params.fal_endpoint
            if style_id:
                config_dict["style_id"] = style_id
            model_config = json.dumps(config_dict)
            if params.loras:
                loras_json = json.dumps([
                    {"model_file": l.model_file, "strength": l.strength}
                    for l in params.loras
                ])

        try:
            self._db.insert_image(
                id=str(uuid.uuid4()),
                file_path=file_path,
                prompt=params.prompt if params else "",
                raw_prompt=raw_prompt,
                width=params.width if params else 0,
                height=params.height if params else 0,
                steps=params.steps if params else 0,
                cfg_scale=params.cfg_scale if params else 0.0,
                seed=event_data.get("seed", -1),
                file_size=file_size,
                bundle_id=bundle_id,
                model_config=model_config,
                loras=loras_json,
                generation_time_ms=generation_time_ms,
            )
            logger.info("Persisted image for job %s to database", job_id)
        except Exception:
            logger.exception("Failed to persist image for job %s", job_id)

    def _cleanup_previews(self, job_id: str) -> None:
        """Delete preview images for a completed job."""
        try:
            for preview in self._output_dir.glob(f"{job_id}_preview_*.png"):
                preview.unlink()
                logger.debug("Cleaned up preview: %s", preview.name)
        except Exception:
            logger.exception("Failed to clean up previews for job %s", job_id)
