"""Simple HTTP file server for serving local images to the frontend.

Browsers block file:// URLs from pages loaded via http:// (Vite dev server
and pywebview internal server). This module starts a lightweight HTTP server
on 127.0.0.1 that serves image files from allowed directories.
"""

import mimetypes
import os
import threading
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs, unquote

from loguru import logger


class _FileHandler(BaseHTTPRequestHandler):
    """HTTP handler that serves files from allowed directories."""

    allowed_dirs: list[str] = []

    def do_GET(self):
        parsed = urlparse(self.path)

        if parsed.path != "/file":
            self.send_error(404)
            return

        params = parse_qs(parsed.query)
        raw_path = params.get("path", [None])[0]

        if not raw_path:
            self.send_error(400, "Missing 'path' parameter")
            return

        file_path = os.path.realpath(unquote(raw_path))

        # Security: only serve files within allowed directories
        if not any(
            file_path.startswith(os.path.realpath(d)) for d in self.allowed_dirs
        ):
            self.send_error(403, "Path not in allowed directories")
            return

        if not os.path.isfile(file_path):
            self.send_error(404, "File not found")
            return

        content_type, _ = mimetypes.guess_type(file_path)
        content_type = content_type or "application/octet-stream"

        try:
            with open(file_path, "rb") as f:
                data = f.read()

            self.send_response(200)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(data)))
            self.send_header("Access-Control-Allow-Origin", "*")
            self.send_header("Cache-Control", "public, max-age=86400")
            self.end_headers()
            self.wfile.write(data)
        except Exception as e:
            self.send_error(500, str(e))

    def log_message(self, format, *args):
        """Suppress default HTTP logging (loguru handles ours)."""
        pass


def start_file_server(allowed_dirs: list[str], port: int = 0) -> tuple[HTTPServer, int]:
    """Start a file server on 127.0.0.1.

    Args:
        allowed_dirs: Directories from which files may be served.
        port: Port to bind (0 = OS picks a random free port).

    Returns:
        (server, actual_port)
    """
    handler_cls = type("Handler", (_FileHandler,), {"allowed_dirs": allowed_dirs})
    server = HTTPServer(("127.0.0.1", port), handler_cls)
    actual_port = server.server_address[1]

    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()

    logger.info(f"File server started on http://127.0.0.1:{actual_port}")
    return server, actual_port
