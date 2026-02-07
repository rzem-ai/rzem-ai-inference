"""Application state management"""

from typing import Optional
from pathlib import Path
from loguru import logger

from job_queue import QueueManager, QueueProcessor
from db import InferenceDb
from shared import RuntimeConfig
import events


class AppState:
    """Central application state"""

    def __init__(self, runtime_config: RuntimeConfig, db_path: Optional[str] = None):
        self.runtime_config = runtime_config

        # Database
        self.db: Optional[InferenceDb] = None
        self.db_path = db_path or str(Path.home() / ".rzem-ai-inference" / "inference.db")

        # Queue system
        self.queue_manager = QueueManager()
        self.queue_processor: Optional[QueueProcessor] = None

        # Event callbacks
        self.event_handlers = []

        logger.info(f"AppState initialized in {runtime_config.mode} mode")

    async def initialize(self) -> None:
        """Initialize application state"""
        logger.info("Initializing application state")

        # Initialize database
        if not self.db:
            self.db = InferenceDb(self.db_path)
            await self.db.connect()
            logger.info(f"Database connected: {self.db_path}")

        # Initialize queue processor
        if not self.queue_processor:
            self.queue_processor = QueueProcessor(
                self.queue_manager,
                self.db,
                output_dir=str(Path.home() / ".rzem-ai-inference" / "outputs"),
            )

            # Register event callbacks
            self.queue_manager.register_event_callback(self._handle_event)
            self.queue_processor.register_event_callback(self._handle_event)

            # Start processor
            await self.queue_processor.start()
            logger.info("Queue processor started")

        logger.info("Application state initialized")

    async def shutdown(self) -> None:
        """Shutdown application state"""
        logger.info("Shutting down application state")

        # Stop queue processor
        if self.queue_processor:
            await self.queue_processor.stop()

        # Disconnect database
        if self.db:
            await self.db.disconnect()

        logger.info("Application state shutdown complete")

    def register_event_handler(self, handler) -> None:
        """Register an event handler for frontend updates"""
        self.event_handlers.append(handler)

    async def _handle_event(self, event_name: str, payload: dict) -> None:
        """Handle events from queue/processor"""
        # Push to event queue for frontend polling
        await events.push_event(event_name, payload)

        # Also call registered handlers
        for handler in self.event_handlers:
            try:
                await handler(event_name, payload)
            except Exception as e:
                logger.error(f"Error in event handler: {e}")
