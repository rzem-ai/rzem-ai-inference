"""
Base API class with shared infrastructure.

Provides the async bridge, health check, database init, and event polling
that all other API mixins depend on.

Uses a metaclass to automatically handle pywebview's argument passing:
pywebview passes JS objects as a single positional dict argument, but our
Python methods expect individual keyword arguments. The metaclass wraps
all public methods to auto-unpack dicts and convert camelCase → snake_case.
"""

import asyncio
import re
from functools import wraps
from typing import Optional, List, Dict, Any
from loguru import logger

from app_state import AppState
import events


def _camel_to_snake(name: str) -> str:
    """Convert camelCase or PascalCase to snake_case."""
    s1 = re.sub(r'(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub(r'([a-z0-9])([A-Z])', r'\1_\2', s1).lower()


def _unpack_js_args(func):
    """Wrap a method to handle pywebview's JS→Python argument passing.

    When the frontend calls invoke('method', {key1: val1, key2: val2}),
    pywebview passes the entire JS object as a single positional argument.
    This wrapper detects that pattern and unpacks the dict into keyword
    arguments with camelCase → snake_case key conversion.

    If unpacking fails (method expects a single dict param), falls back
    to passing the dict as-is.
    """
    @wraps(func)
    def wrapper(self, *args, **kwargs):
        if len(args) == 1 and isinstance(args[0], dict) and not kwargs:
            js_dict = args[0]
            snake_kwargs = {_camel_to_snake(k): v for k, v in js_dict.items()}
            try:
                return func(self, **snake_kwargs)
            except TypeError:
                # Method signature doesn't match kwargs (e.g. expects a single
                # dict param like `folder: Dict`). Pass the original dict as-is.
                return func(self, js_dict)
        return func(self, *args, **kwargs)
    return wrapper


class _ApiMeta(type):
    """Metaclass that wraps all public methods with JS dict unpacking."""

    def __new__(mcs, name, bases, namespace):
        for key in list(namespace):
            if key.startswith('_'):
                continue
            value = namespace[key]
            if isinstance(value, (classmethod, staticmethod, property)):
                continue
            if callable(value):
                namespace[key] = _unpack_js_args(value)
        return super().__new__(mcs, name, bases, namespace)


class ApiBase(metaclass=_ApiMeta):
    """
    Base class for the API. All mixins inherit from this
    and use self._app_state / self._run_async.

    The _ApiMeta metaclass automatically wraps all public methods
    to handle pywebview's JS→Python argument passing.
    """

    def __init__(self, app_state: AppState):
        self._app_state = app_state
        self._loop: Optional[asyncio.AbstractEventLoop] = None

    def set_event_loop(self, loop: asyncio.AbstractEventLoop) -> None:
        """Set the asyncio event loop for async operations"""
        self._loop = loop

    def _run_async(self, coro):
        """Helper to run async coroutines from sync context"""
        if not self._loop:
            raise RuntimeError("Event loop not set")
        return asyncio.run_coroutine_threadsafe(coro, self._loop).result()

    # ==================== Health ====================

    def health_check(self) -> str:
        """Health check endpoint"""
        return "OK"

    # ==================== Database ====================

    def init_database(self, db_path: str) -> Dict[str, str]:
        """Initialize the database."""
        try:
            self._app_state.db_path = db_path
            self._run_async(self._app_state.initialize())
            return {"status": "success", "message": "Database initialized"}
        except Exception as e:
            logger.error(f"Failed to initialize database: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Events ====================

    def poll_events(self, max_events: int = 50) -> List[Dict[str, Any]]:
        """
        Poll for events from the backend.

        Frontend should call this periodically to get updates.
        """
        try:
            events_list = self._run_async(events.pop_events(max_events))
            return events_list
        except Exception as e:
            logger.error(f"Failed to poll events: {e}")
            return []
