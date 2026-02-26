import logging
import platform
import threading

from backend.tracing import trace_class

logger = logging.getLogger(__name__)


@trace_class
class AppService:
    def __init__(self) -> None:
        self._counter = 0
        self._lock = threading.Lock()

    def get_system_info(self) -> dict:
        return {
            "platform": platform.system(),
            "platform_version": platform.version(),
            "python_version": platform.python_version(),
            "machine": platform.machine(),
            "processor": platform.processor(),
        }

    def greet(self, name: str) -> str:
        return f"Hello, {name}! Welcome from the Python backend."

    def increment_counter(self) -> int:
        with self._lock:
            self._counter += 1
            return self._counter

    def get_counter(self) -> int:
        with self._lock:
            return self._counter
