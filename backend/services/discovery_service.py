"""Zeroconf service discovery for remote inference engines on the LAN."""

from __future__ import annotations

import logging
import threading
from dataclasses import dataclass, field
from typing import Any

from backend.tracing import trace_class

logger = logging.getLogger(__name__)

SERVICE_TYPE = "_rzem-ai._tcp.local."


@dataclass
class DiscoveredServer:
    """A remote inference engine found via zeroconf."""
    name: str
    host: str
    port: int
    device: str = "unknown"
    version: str = "unknown"


@trace_class
class DiscoveryService:
    """Watches for ``_rzem-ai._tcp.local.`` services on the LAN.

    Maintains a thread-safe dict of discovered servers.  The API layer
    polls ``get_servers()`` to return the current list to the frontend.
    """

    def __init__(self) -> None:
        self._servers: dict[str, DiscoveredServer] = {}
        self._lock = threading.Lock()
        self._zeroconf: Any = None
        self._browser: Any = None

    def start(self) -> None:
        """Begin listening for service announcements."""
        try:
            from zeroconf import ServiceBrowser, Zeroconf

            self._zeroconf = Zeroconf()
            self._browser = ServiceBrowser(
                self._zeroconf, SERVICE_TYPE, handlers=[self._on_state_change]
            )
            logger.info("Discovery service started, browsing for %s", SERVICE_TYPE)
        except ImportError:
            logger.warning("zeroconf package not installed — discovery disabled")
        except Exception:
            logger.exception("Failed to start discovery service")

    def stop(self) -> None:
        """Stop the zeroconf browser and clean up."""
        if self._browser is not None:
            self._browser.cancel()
            self._browser = None
        if self._zeroconf is not None:
            self._zeroconf.close()
            self._zeroconf = None
        with self._lock:
            self._servers.clear()
        logger.info("Discovery service stopped")

    def get_servers(self) -> list[dict[str, Any]]:
        """Return currently discovered servers as serializable dicts."""
        with self._lock:
            return [
                {
                    "name": s.name,
                    "host": s.host,
                    "port": s.port,
                    "device": s.device,
                    "version": s.version,
                }
                for s in self._servers.values()
            ]

    def _on_state_change(self, zeroconf: Any, service_type: str, name: str, state_change: Any) -> None:
        """Callback fired by ServiceBrowser on add/remove."""
        from zeroconf import ServiceStateChange

        if state_change == ServiceStateChange.Added:
            info = zeroconf.get_service_info(service_type, name)
            if info is None:
                return
            host = info.parsed_addresses()[0] if info.parsed_addresses() else None
            if not host:
                return

            props = {}
            if info.properties:
                props = {
                    k.decode() if isinstance(k, bytes) else k:
                    v.decode() if isinstance(v, bytes) else v
                    for k, v in info.properties.items()
                }

            server = DiscoveredServer(
                name=name.replace(f".{SERVICE_TYPE}", ""),
                host=host,
                port=info.port,
                device=props.get("device", "unknown"),
                version=props.get("version", "unknown"),
            )
            with self._lock:
                self._servers[name] = server
            logger.info("Discovered server: %s at %s:%d", server.name, server.host, server.port)

        elif state_change == ServiceStateChange.Removed:
            with self._lock:
                removed = self._servers.pop(name, None)
            if removed:
                logger.info("Server removed: %s", removed.name)
