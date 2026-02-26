"""API layer — thin facades exposed to the frontend via pywebview.

The ``ApiMeta`` metaclass auto-wraps every public method so that
pywebview's single-dict calling convention is transparently unpacked
into proper keyword arguments.
"""

from __future__ import annotations

import functools
import inspect
import logging
import re
from typing import Any, Callable

logger = logging.getLogger(__name__)

# camelCase → snake_case
_CAMEL_RE = re.compile(r"(?<=[a-z0-9])([A-Z])")

# High-frequency methods that would flood DEBUG logs
_TRACE_SKIP = frozenset({"poll_events", "health_check"})


def _camel_to_snake(name: str) -> str:
    return _CAMEL_RE.sub(r"_\1", name).lower()


def _unpack_js_args(func: Callable) -> Callable:
    """Decorator that converts pywebview's single-dict call into **kwargs.

    pywebview passes ``method({"camelKey": val, ...})`` — one positional
    dict.  This decorator converts camelCase keys to snake_case and
    forwards them as keyword arguments.
    """
    sig = inspect.signature(func)
    param_names = [
        p.name for p in sig.parameters.values()
        if p.name != "self"
    ]

    _skip_trace = func.__name__ in _TRACE_SKIP

    @functools.wraps(func)
    def wrapper(self: Any, *args: Any, **kwargs: Any) -> Any:
        if not _skip_trace:
            # Trace entry — truncate arg representations to avoid huge payloads
            _tr_args = args
            if _tr_args and isinstance(_tr_args[0], dict):
                _tr_repr = repr(_tr_args[0])
                if len(_tr_repr) > 120:
                    _tr_repr = _tr_repr[:117] + "..."
                logger.debug("→ %s(%s)", func.__qualname__, _tr_repr)
            else:
                _tr_parts = [repr(a)[:120] for a in _tr_args] + [f"{k}={repr(v)[:120]}" for k, v in kwargs.items()]
                logger.debug("→ %s(%s)", func.__qualname__, ", ".join(_tr_parts))

        try:
            # Only unpack when pywebview sends a single positional dict
            if len(args) == 1 and isinstance(args[0], dict) and not kwargs:
                raw: dict[str, Any] = args[0]
                converted = {_camel_to_snake(k): v for k, v in raw.items()}

                # If the method actually expects a single dict param, pass as-is
                if len(param_names) == 1 and param_names[0] not in converted:
                    result = func(self, raw)
                else:
                    result = func(self, **converted)
            else:
                result = func(self, *args, **kwargs)

            if not _skip_trace:
                logger.debug("← %s ⟶ %s", func.__qualname__, type(result).__name__)
            return result
        except Exception as exc:
            logger.debug("✗ %s raised %s: %s", func.__qualname__, type(exc).__name__, exc)
            raise

    return wrapper


class ApiMeta(type):
    """Metaclass that applies ``_unpack_js_args`` to every public method."""

    def __new__(mcs, name: str, bases: tuple, namespace: dict) -> type:
        for attr_name, attr_value in namespace.items():
            if (
                callable(attr_value)
                and not attr_name.startswith("_")
                and isinstance(attr_value, (staticmethod, classmethod)) is False
            ):
                namespace[attr_name] = _unpack_js_args(attr_value)

        cls = super().__new__(mcs, name, bases, namespace)

        # Also wrap inherited methods that weren't overridden
        for base in bases:
            for attr_name in dir(base):
                if attr_name.startswith("_") or attr_name in namespace:
                    continue
                attr_value = getattr(base, attr_name, None)
                if callable(attr_value) and not isinstance(attr_value, type):
                    setattr(cls, attr_name, _unpack_js_args(attr_value))

        return cls
