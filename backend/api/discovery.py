"""Discovery API — pywebview bridge for remote server discovery and connection."""

from __future__ import annotations

import logging
from typing import Any

from backend.services.discovery_service import DiscoveryService
from backend.services.inference_manager import InferenceServiceManager

logger = logging.getLogger(__name__)


class DiscoveryAPI:
    """pywebview js_api mixin for remote server discovery."""

    def __init__(self, manager: InferenceServiceManager, discovery: DiscoveryService) -> None:
        self._manager = manager
        self._discovery = discovery

    def get_discovered_servers(self) -> dict[str, Any]:
        """Return the list of inference servers found on the LAN."""
        try:
            servers = self._discovery.get_servers()
            return {"status": "success", "servers": servers}
        except Exception as e:
            logger.error("Failed to get discovered servers: %s", e)
            return {"status": "error", "message": str(e)}

    def connect_to_server(self, host: str, port: int, **kwargs) -> dict[str, Any]:
        """Connect to a remote inference server."""
        try:
            self._manager.connect_remote(host, port)
            return {"status": "success", "mode": "remote"}
        except Exception as e:
            logger.error("Failed to connect to %s:%d: %s", host, port, e)
            return {"status": "error", "message": str(e)}

    def disconnect_from_server(self) -> dict[str, Any]:
        """Disconnect from the remote server and revert to local mode."""
        try:
            self._manager.disconnect_remote()
            return {"status": "success", "mode": "local"}
        except Exception as e:
            logger.error("Failed to disconnect: %s", e)
            return {"status": "error", "message": str(e)}

    def get_connection_mode(self) -> dict[str, Any]:
        """Return the current connection mode and connected server info."""
        try:
            return {
                "status": "success",
                "mode": self._manager.mode,
                "connected_server": self._manager.connected_server,
            }
        except Exception as e:
            logger.error("Failed to get connection mode: %s", e)
            return {"status": "error", "message": str(e)}
