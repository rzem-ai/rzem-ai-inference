"""Remote inference service — HTTP REST + WebSocket client for a remote engine server."""

from __future__ import annotations

import base64
import json
import logging
import os
import threading
import time
import uuid
from collections import deque
from pathlib import Path
from typing import Any

import requests
import websocket

from backend.db.database import Database
from backend.services.inference_protocol import FrontendEvent

logger = logging.getLogger(__name__)


class RemoteInferenceService:
    """Connects to a remote inference engine server over HTTP + WebSocket.

    Implements the same interface as LocalInferenceService so the manager
    can delegate transparently.  Events from the server's WebSocket are
    translated into FrontendEvent objects in the same deque format.
    """

    def __init__(self, host: str, port: int, output_dir: Path, db: Database) -> None:
        self._host = host
        self._port = port
        self._base_url = f"http://{host}:{port}"
        self._ws_url = f"ws://{host}:{port}/ws"
        self.set_output_dir(output_dir)
        self._db = db

        self._events: deque[FrontendEvent] = deque(maxlen=500)
        self._lock = threading.Lock()
        self._ready = False
        self._ws: websocket.WebSocketApp | None = None
        self._ws_thread: threading.Thread | None = None
        self._start_time: float | None = None
        self._completed_count: int = 0

        # Track job params for DB persistence (same as local service)
        self._job_params: dict[str, Any] = {}
        self._job_start_times: dict[str, float] = {}
        self._job_bundle_ids: dict[str, str] = {}
        self._job_style_ids: dict[str, str] = {}
        self._job_raw_prompts: dict[str, str] = {}

    def set_output_dir(self, output_dir: Path) -> None:
        """Update the output directory, creating it if needed."""
        self._output_dir = output_dir
        self._output_dir.mkdir(parents=True, exist_ok=True)

    @property
    def ready(self) -> bool:
        return self._ready

    def start(self, device: str = "auto", vram_limit_gb: float | None = None) -> None:
        """Connect the WebSocket to the remote server."""
        if self._ready:
            return
        self._start_ws()
        self._start_time = time.monotonic()
        self._ready = True

    def shutdown(self) -> None:
        """Disconnect from the remote server."""
        self._ready = False
        self._start_time = None
        if self._ws is not None:
            self._ws.close()
            self._ws = None
        if self._ws_thread is not None:
            self._ws_thread.join(timeout=3)
            self._ws_thread = None

    def submit(
        self,
        params: Any,
        bundle_id: str | None = None,
        style_id: str | None = None,
        raw_prompt: str | None = None,
    ) -> str:
        """Submit a generation job to the remote server via POST /jobs."""
        payload = params.model_dump() if hasattr(params, "model_dump") else dict(params)

        # Convert enum values to strings for JSON serialization
        for key, val in payload.items():
            if hasattr(val, "value"):
                payload[key] = val.value

        resp = requests.post(f"{self._base_url}/jobs", json=payload, timeout=30)
        resp.raise_for_status()
        data = resp.json()
        job_id = data["job_id"]

        # Track for DB persistence on completion
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
        """Cancel a job on the remote server via DELETE /jobs/{job_id}."""
        resp = requests.delete(f"{self._base_url}/jobs/{job_id}", timeout=10)
        resp.raise_for_status()

    def drain_events(self) -> list[dict[str, Any]]:
        """Return and clear all buffered events. Thread-safe."""
        with self._lock:
            events = list(self._events)
            self._events.clear()
        return [{"type": e.type, "data": e.data} for e in events]

    def get_engine_status(self) -> dict[str, Any]:
        """Query the remote server's health endpoint."""
        try:
            resp = requests.get(f"{self._base_url}/health", timeout=5)
            resp.raise_for_status()
            health = resp.json()
            uptime = None
            if self._start_time is not None:
                uptime = int(time.monotonic() - self._start_time)
            return {
                "ready": True,
                "uptime_seconds": uptime,
                "completed_count": self._completed_count,
                "remote": True,
                "device": health.get("device", "unknown"),
                "jobs_queued": health.get("jobs_queued", 0),
                "jobs_running": health.get("jobs_running", 0),
            }
        except Exception:
            return {"ready": False, "uptime_seconds": None, "completed_count": 0}

    def get_gpu_info(self) -> dict[str, Any]:
        """Get device info from the remote server's health endpoint."""
        try:
            resp = requests.get(f"{self._base_url}/health", timeout=5)
            resp.raise_for_status()
            health = resp.json()
            return {
                "device_type": health.get("device", "unknown"),
                "device_name": f"Remote ({self._host}:{self._port})",
                "total_vram_gb": 0,
            }
        except Exception:
            return {"device_type": "unknown", "device_name": None, "total_vram_gb": 0}

    # ── WebSocket ──────────────────────────────────────────────────

    def _start_ws(self) -> None:
        """Start the WebSocket client in a daemon thread."""
        self._ws = websocket.WebSocketApp(
            self._ws_url,
            on_message=self._on_message,
            on_error=self._on_error,
            on_close=self._on_close,
            on_open=self._on_open,
        )
        self._ws_thread = threading.Thread(
            target=self._ws.run_forever,
            kwargs={"reconnect": 5},
            daemon=True,
            name="remote-ws",
        )
        self._ws_thread.start()

    def _on_open(self, ws: Any) -> None:
        logger.info("WebSocket connected to %s", self._ws_url)
        self._emit(FrontendEvent(type="server_connected", data={
            "host": self._host,
            "port": self._port,
        }))

    def _on_close(self, ws: Any, close_status_code: Any, close_msg: Any) -> None:
        logger.info("WebSocket disconnected from %s (code=%s)", self._ws_url, close_status_code)

    def _on_error(self, ws: Any, error: Any) -> None:
        logger.error("WebSocket error: %s", error)

    def _on_message(self, ws: Any, message: str) -> None:
        """Handle a WebSocket message from the server."""
        try:
            msg = json.loads(message)
            event_type = msg.get("event", "")
            job_id = msg.get("job_id", "")
            data = msg.get("data", {})
            data["job_id"] = job_id

            if event_type == "job_progress":
                self._handle_progress(data)
            elif event_type == "job_completed":
                self._handle_completed(data)
            elif event_type == "job_failed":
                self._emit(FrontendEvent(type="job_failed", data=data))
            elif event_type == "job_cancelled":
                self._emit(FrontendEvent(type="job_cancelled", data=data))
            elif event_type == "job_queued":
                self._emit(FrontendEvent(type="job_queued", data=data))
            elif event_type == "job_started":
                self._emit(FrontendEvent(type="job_started", data=data))
            else:
                self._emit(FrontendEvent(type=event_type, data=data))
        except Exception:
            logger.exception("Failed to process WebSocket message")

    # ── Event handlers ─────────────────────────────────────────────

    def _handle_progress(self, data: dict[str, Any]) -> None:
        """Translate a progress event — decode preview if present."""
        preview_data_uri = data.pop("preview", None)
        if preview_data_uri and isinstance(preview_data_uri, str):
            preview_path = self._save_data_uri(
                preview_data_uri,
                data.get("job_id", "unknown"),
                f"preview_{data.get('step', 0)}",
            )
            if preview_path:
                data["preview_path"] = preview_path
        self._emit(FrontendEvent(type="job_progress", data=data))

    def _handle_completed(self, data: dict[str, Any]) -> None:
        """Download the final image and persist to gallery DB."""
        job_id = data.get("job_id", "unknown")
        image_url = data.pop("image_url", None)

        if image_url:
            image_path = self._download_image(job_id, image_url)
            if image_path:
                data["image_path"] = image_path
                self._completed_count += 1
                self._persist_image(data)
                self._cleanup_previews(job_id)

        self._emit(FrontendEvent(type="job_completed", data=data))

    def _download_image(self, job_id: str, image_url: str) -> str | None:
        """Download the completed image from the server."""
        try:
            url = f"{self._base_url}{image_url}"
            resp = requests.get(url, timeout=60)
            resp.raise_for_status()

            filename = f"{job_id}_output.png"
            path = self._output_dir / filename
            path.write_bytes(resp.content)
            logger.info("Downloaded image for job %s to %s", job_id, path)
            return str(path)
        except Exception:
            logger.exception("Failed to download image for job %s", job_id)
            return None

    def _save_data_uri(self, data_uri: str, job_id: str, suffix: str) -> str | None:
        """Decode a base64 data URI and save to disk."""
        try:
            # Format: "data:image/jpeg;base64,/9j/4AAQ..."
            header, b64data = data_uri.split(",", 1)
            ext = "jpg" if "jpeg" in header else "png"
            image_bytes = base64.b64decode(b64data)

            filename = f"{job_id}_{suffix}.{ext}"
            path = self._output_dir / filename
            path.write_bytes(image_bytes)
            return str(path)
        except Exception:
            logger.exception("Failed to save data URI for %s", job_id)
            return None

    def _persist_image(self, event_data: dict[str, Any]) -> None:
        """Insert a completed image record into the database."""
        job_id = event_data.get("job_id", "unknown")
        params = self._job_params.pop(job_id, None)
        start_time = self._job_start_times.pop(job_id, None)

        generation_time_ms = None
        if start_time is not None:
            generation_time_ms = int((time.monotonic() - start_time) * 1000)

        file_path = event_data.get("image_path", "")
        file_size = None
        if file_path and os.path.isfile(file_path):
            file_size = os.path.getsize(file_path)

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
                "clip_tokenizer": getattr(params, "clip_tokenizer", None),
                "clip_encoder": getattr(params, "clip_encoder", None),
                "t5_tokenizer": getattr(params, "t5_tokenizer", None),
                "t5_encoder": getattr(params, "t5_encoder", None),
                "qwen3_tokenizer": getattr(params, "qwen3_tokenizer", None),
                "qwen3_encoder": getattr(params, "qwen3_encoder", None),
                "sampler": params.sampler,
                "scheduler": params.scheduler,
                "remote_server": f"{self._host}:{self._port}",
            }
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
            logger.info("Persisted remote image for job %s to database", job_id)
        except Exception:
            logger.exception("Failed to persist remote image for job %s", job_id)

    def _cleanup_previews(self, job_id: str) -> None:
        """Delete preview images for a completed job."""
        try:
            for preview in self._output_dir.glob(f"{job_id}_preview_*.*"):
                preview.unlink()
                logger.debug("Cleaned up preview: %s", preview.name)
        except Exception:
            logger.exception("Failed to clean up previews for job %s", job_id)

    def _emit(self, event: FrontendEvent) -> None:
        """Append an event to the thread-safe buffer."""
        with self._lock:
            self._events.append(event)
