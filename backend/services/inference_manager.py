"""Orchestrator that holds the active inference service and switches between local/remote."""

from __future__ import annotations

import logging
from pathlib import Path
from typing import Any

from backend.db.database import Database
from backend.services.inference_protocol import InferenceServiceProtocol
from backend.services.remote_inference_service import RemoteInferenceService

logger = logging.getLogger(__name__)


class InferenceServiceManager:
    """Holds a reference to the active inference service (local or remote).

    The API layer delegates to ``manager.active`` for all operations.
    For local-only operations (VRAM, CUDA), use ``manager.local`` directly.
    """

    def __init__(
        self,
        local: InferenceServiceProtocol,
        output_dir: Path,
        db: Database,
    ) -> None:
        self._local = local
        self._remote: RemoteInferenceService | None = None
        self._active: InferenceServiceProtocol = local
        self._mode: str = "local"
        self._output_dir = output_dir
        self._db = db

    @property
    def active(self) -> InferenceServiceProtocol:
        return self._active

    @property
    def local(self) -> InferenceServiceProtocol:
        return self._local

    @property
    def mode(self) -> str:
        return self._mode

    @property
    def connected_server(self) -> dict[str, Any] | None:
        if self._remote is not None:
            return {"host": self._remote._host, "port": self._remote._port}
        return None

    def connect_remote(self, host: str, port: int) -> None:
        """Create a remote service, connect, and switch to it."""
        if self._remote is not None:
            self.disconnect_remote()

        self._remote = RemoteInferenceService(
            host=host,
            port=port,
            output_dir=self._output_dir,
            db=self._db,
        )
        self._remote.start()
        self._active = self._remote
        self._mode = "remote"
        logger.info("Switched to remote mode: %s:%d", host, port)

    def set_output_dir(self, output_dir: Path) -> None:
        """Update the output directory for all services."""
        self._output_dir = output_dir
        self._local.set_output_dir(output_dir)
        if self._remote is not None:
            self._remote.set_output_dir(output_dir)

    def disconnect_remote(self) -> None:
        """Shut down the remote service and revert to local."""
        if self._remote is not None:
            self._remote.shutdown()
            self._remote = None
        self._active = self._local
        self._mode = "local"
        logger.info("Switched back to local mode")

    # ── Delegated properties/methods ───────────────────────────────

    @property
    def ready(self) -> bool:
        return self._active.ready

    def drain_events(self) -> list[dict[str, Any]]:
        """Drain events from the active service.

        When in remote mode, also drain any local events (e.g. model loading
        that happened locally before switching).
        """
        events = self._active.drain_events()
        if self._mode == "remote" and self._active is not self._local:
            events.extend(self._local.drain_events())
        return events
