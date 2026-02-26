"""Lightweight tracing for method entry/exit logging.

Usage — class decorator (wraps all public methods automatically):

    from backend.tracing import trace_class

    @trace_class
    class MyService:
        def do_work(self, item_id: str) -> dict: ...

Usage — individual method decorator:

    from backend.tracing import trace_method

    @trace_method
    def standalone_function(path: str) -> None: ...

All trace output goes to ``logging.DEBUG`` so it is invisible in
production unless the log level is explicitly lowered.
"""

from __future__ import annotations

import functools
import logging
from typing import Any, Callable

_TRUNCATE_LEN = 120

# High-frequency methods that would flood DEBUG logs
_TRACE_SKIP = frozenset({"drain_events"})


def _repr_arg(value: Any) -> str:
    """Return a compact string representation, truncating large values."""
    r = repr(value)
    if len(r) > _TRUNCATE_LEN:
        return r[: _TRUNCATE_LEN - 3] + "..."
    return r


def _format_args(args: tuple, kwargs: dict) -> str:
    parts = [_repr_arg(a) for a in args]
    parts += [f"{k}={_repr_arg(v)}" for k, v in kwargs.items()]
    return ", ".join(parts)


def trace_method(func: Callable) -> Callable:
    """Decorator that logs method entry and exit at DEBUG level."""
    if func.__name__ in _TRACE_SKIP:
        return func

    log = logging.getLogger(func.__module__)
    qualname = func.__qualname__

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        # Skip 'self' / 'cls' in displayed args
        display_args = args[1:] if args else args
        log.debug("→ %s(%s)", qualname, _format_args(display_args, kwargs))
        try:
            result = func(*args, **kwargs)
            log.debug("← %s ⟶ %s", qualname, type(result).__name__)
            return result
        except Exception as exc:
            log.debug("✗ %s raised %s: %s", qualname, type(exc).__name__, exc)
            raise

    return wrapper


def trace_class(cls: type) -> type:
    """Class decorator — applies ``trace_method`` to every public method."""
    for name in list(vars(cls)):
        if name.startswith("_"):
            continue
        attr = vars(cls)[name]
        if callable(attr):
            setattr(cls, name, trace_method(attr))
    return cls
