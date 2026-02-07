"""
Event system for frontend communication.

Since pywebview doesn't have a built-in event system like Tauri,
we implement a simple polling-based event system.
"""

import asyncio
from typing import Dict, Any, List
from collections import deque
from loguru import logger


class EventQueue:
    """
    Simple event queue for frontend polling.

    Events are stored in a FIFO queue and can be retrieved by the frontend.
    """

    def __init__(self, max_size: int = 1000):
        self.events: deque = deque(maxlen=max_size)
        self.lock = asyncio.Lock()

    async def push_event(self, event_name: str, payload: Dict[str, Any]) -> None:
        """
        Push an event to the queue.

        Args:
            event_name: Event name (e.g., "job-progress")
            payload: Event payload
        """
        async with self.lock:
            self.events.append({
                "event": event_name,
                "payload": payload,
            })

        logger.debug(f"Event pushed: {event_name}")

    async def pop_events(self, max_events: int = 50) -> List[Dict[str, Any]]:
        """
        Pop events from the queue (up to max_events).

        Args:
            max_events: Maximum number of events to return

        Returns:
            List of events
        """
        async with self.lock:
            events = []
            for _ in range(min(len(self.events), max_events)):
                events.append(self.events.popleft())
            return events

    async def clear(self) -> None:
        """Clear all events from the queue"""
        async with self.lock:
            self.events.clear()


# Global event queue instance
_event_queue = EventQueue()


async def push_event(event_name: str, payload: Dict[str, Any]) -> None:
    """Push an event to the global queue"""
    await _event_queue.push_event(event_name, payload)


async def pop_events(max_events: int = 50) -> List[Dict[str, Any]]:
    """Pop events from the global queue"""
    return await _event_queue.pop_events(max_events)


async def clear_events() -> None:
    """Clear all events from the global queue"""
    await _event_queue.clear()
