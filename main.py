import logging
import os
import signal
import subprocess
import time
from pathlib import Path
from urllib.request import urlopen
from urllib.error import URLError

import webview

from backend.api.combined import CombinedAPI
from backend.bundles import BundleStore
from backend.config import AppConfig
from backend.services.app_service import AppService
from backend.services.inference_service import InferenceService

logger = logging.getLogger(__name__)

# PGID captured at start time — survives even after the shell shim exec's
_vite_pgid: int | None = None


def _start_vite(frontend_dir: Path, url: str, timeout: float = 15) -> subprocess.Popen:
    """Start the Vite dev server and wait until it's accepting connections."""
    global _vite_pgid
    vite_bin = frontend_dir / "node_modules" / ".bin" / "vite"
    proc = subprocess.Popen(
        [str(vite_bin), "--port", "1978"],
        cwd=str(frontend_dir),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        preexec_fn=os.setsid,
    )
    # Capture PGID now, while the original PID still exists.
    # After the shell shim exec's into node, proc.pid becomes stale.
    _vite_pgid = os.getpgid(proc.pid)
    logger.info("Vite dev server started (PID %d, PGID %d)", proc.pid, _vite_pgid)

    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            urlopen(url, timeout=1).close()
            logger.info("Vite dev server ready at %s", url)
            return proc
        except (URLError, OSError):
            time.sleep(0.3)

    proc.kill()
    raise RuntimeError(f"Vite dev server failed to start within {timeout}s")


def _stop_vite() -> None:
    """Kill the entire Vite process group using the PGID captured at start."""
    if _vite_pgid is None:
        return
    try:
        os.killpg(_vite_pgid, signal.SIGKILL)
    except (ProcessLookupError, OSError):
        pass


def main() -> None:
    config = AppConfig()
    service = AppService()
    inference = InferenceService(output_dir=config.output_dir)
    bundle_store = BundleStore(data_dir=config.data_dir)
    bundle_store.load()
    api = CombinedAPI(service, inference, bundle_store)

    # In dev mode, start Vite before opening the window
    if config.dev_mode:
        _start_vite(config.base_dir / "frontend", config.vite_dev_url)

    window = webview.create_window(
        config.window_title,
        url=config.entry_url,
        js_api=api,
        width=config.window_width,
        height=config.window_height,
        x=config.window_x,
        y=config.window_y,
        min_size=(config.min_width, config.min_height),
    )

    def on_closing():
        """Kill vite and force-exit. Skip graceful engine shutdown —
        tearing down CUDA from a non-main thread triggers C++ errors.
        Daemon threads and GPU resources are cleaned up by the OS."""
        _stop_vite()
        os._exit(0)

    window.events.closing += on_closing
    webview.start(debug=config.dev_mode)


if __name__ == "__main__":
    main()
