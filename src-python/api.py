"""
API Bridge for pywebview

This module exposes Python functions to the JavaScript frontend.
It replaces Tauri commands with pywebview's JS API.
"""

import asyncio
from typing import Optional, List, Dict, Any
from loguru import logger

from app_state import AppState
from job_queue.types import GenerationParams, GenerationJob, JobStatus
import events
from updater import get_updater, CURRENT_VERSION


class Api:
    """
    API class that exposes methods to JavaScript frontend.

    All methods in this class are automatically exposed to the JavaScript
    via pywebview.api. Frontend can call them using:

        await window.pywebview.api.method_name(args)
    """

    def __init__(self, app_state: AppState):
        self.app_state = app_state
        self.loop: Optional[asyncio.AbstractEventLoop] = None

    def set_event_loop(self, loop: asyncio.AbstractEventLoop) -> None:
        """Set the asyncio event loop for async operations"""
        self.loop = loop

    def _run_async(self, coro):
        """Helper to run async coroutines from sync context"""
        if not self.loop:
            raise RuntimeError("Event loop not set")
        return asyncio.run_coroutine_threadsafe(coro, self.loop).result()

    # ==================== Health ====================

    def health_check(self) -> str:
        """Health check endpoint"""
        return "OK"

    # ==================== Database ====================

    def init_database(self, db_path: str) -> Dict[str, str]:
        """
        Initialize the database.

        Args:
            db_path: Path to the database file

        Returns:
            {"status": "success", "message": "..."}
        """
        try:
            self.app_state.db_path = db_path
            self._run_async(self.app_state.initialize())
            return {"status": "success", "message": "Database initialized"}
        except Exception as e:
            logger.error(f"Failed to initialize database: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Queue/Generation ====================

    def queue_generation(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """
        Queue a new generation job.

        Args:
            params: Generation parameters dict

        Returns:
            {"job_id": "...", "status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            # Parse params
            gen_params = GenerationParams(**params)

            # Validate
            if not gen_params.prompt:
                return {"status": "error", "message": "Prompt cannot be empty"}

            # Add to queue
            job_id = self._run_async(self.app_state.queue_manager.add_job(gen_params))

            logger.info(f"Job queued: {job_id}")
            return {"status": "success", "job_id": job_id}

        except Exception as e:
            logger.error(f"Failed to queue generation: {e}")
            return {"status": "error", "message": str(e)}

    def get_all_jobs(self) -> List[Dict[str, Any]]:
        """
        Get all generation jobs.

        Returns:
            List of job dicts
        """
        try:
            jobs = self._run_async(self.app_state.queue_manager.get_all_jobs())
            return [job.model_dump() for job in jobs]
        except Exception as e:
            logger.error(f"Failed to get jobs: {e}")
            return []

    def get_job(self, job_id: str) -> Optional[Dict[str, Any]]:
        """
        Get a specific job by ID.

        Args:
            job_id: Job ID

        Returns:
            Job dict or None
        """
        try:
            job = self._run_async(self.app_state.queue_manager.get_job(job_id))
            return job.model_dump() if job else None
        except Exception as e:
            logger.error(f"Failed to get job {job_id}: {e}")
            return None

    def cancel_job(self, job_id: str) -> Dict[str, Any]:
        """
        Cancel a job.

        Args:
            job_id: Job ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            success = self._run_async(self.app_state.queue_manager.cancel_job(job_id))
            if success:
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Job not found or cannot be cancelled"}
        except Exception as e:
            logger.error(f"Failed to cancel job {job_id}: {e}")
            return {"status": "error", "message": str(e)}

    def clear_completed_jobs(self) -> Dict[str, str]:
        """
        Clear all completed/failed/cancelled jobs.

        Returns:
            {"status": "success"}
        """
        try:
            self._run_async(self.app_state.queue_manager.clear_completed_jobs())
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to clear jobs: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Gallery ====================

    def get_all_images(self, limit: Optional[int] = None) -> List[Dict[str, Any]]:
        """
        Get all images from gallery.

        Args:
            limit: Optional limit on number of images

        Returns:
            List of image dicts
        """
        try:
            if not self.app_state.db:
                return []
            images = self._run_async(self.app_state.db.get_all_images(limit))
            return images
        except Exception as e:
            logger.error(f"Failed to get images: {e}")
            return []

    def get_image_by_id(self, image_id: str) -> Optional[Dict[str, Any]]:
        """
        Get image by ID.

        Args:
            image_id: Image ID

        Returns:
            Image dict or None
        """
        try:
            if not self.app_state.db:
                return None
            image = self._run_async(self.app_state.db.get_image_by_id(image_id))
            return image
        except Exception as e:
            logger.error(f"Failed to get image {image_id}: {e}")
            return None

    def delete_image(self, image_id: str) -> Dict[str, Any]:
        """
        Delete an image.

        Args:
            image_id: Image ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self.app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self.app_state.db.delete_image(image_id))
            if success:
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Image not found"}
        except Exception as e:
            logger.error(f"Failed to delete image {image_id}: {e}")
            return {"status": "error", "message": str(e)}

    def toggle_favorite(self, image_id: str) -> Dict[str, Any]:
        """
        Toggle favorite status of an image.

        Args:
            image_id: Image ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self.app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self.app_state.db.toggle_favorite(image_id))
            if success:
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Image not found"}
        except Exception as e:
            logger.error(f"Failed to toggle favorite {image_id}: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Settings ====================

    def get_settings(self) -> Dict[str, Any]:
        """
        Get application settings.

        Returns:
            Settings dict
        """
        # TODO: Implement settings persistence
        return {
            "theme": "dark",
            "default_steps": 4,
            "default_width": 1024,
            "default_height": 1024,
        }

    def save_settings(self, settings: Dict[str, Any]) -> Dict[str, str]:
        """
        Save application settings.

        Args:
            settings: Settings dict

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement settings persistence
        logger.info(f"Settings saved: {settings}")
        return {"status": "success"}

    # ==================== Events ====================

    def poll_events(self, max_events: int = 50) -> List[Dict[str, Any]]:
        """
        Poll for events from the backend.

        This replaces Tauri's event system with a polling-based approach.
        Frontend should call this periodically to get updates.

        Args:
            max_events: Maximum number of events to return

        Returns:
            List of events: [{"event": "job-progress", "payload": {...}}, ...]
        """
        try:
            events_list = self._run_async(events.pop_events(max_events))
            return events_list
        except Exception as e:
            logger.error(f"Failed to poll events: {e}")
            return []

    # ==================== Auto-Update ====================

    def get_version(self) -> Dict[str, str]:
        """
        Get current application version.

        Returns:
            {"version": "0.1.0", "status": "success"}
        """
        return {"version": CURRENT_VERSION, "status": "success"}

    def check_for_updates(self) -> Dict[str, Any]:
        """
        Check for available updates.

        Returns:
            {
                "status": "success",
                "update_available": bool,
                "current_version": "0.1.0",
                "latest_version": "0.2.0",
                "download_url": "https://...",
            }
        """
        try:
            updater = get_updater()
            update_available = self._run_async(updater.check_for_updates())

            result = {
                "status": "success",
                "update_available": update_available,
                "current_version": str(updater.current_version),
            }

            if update_available and updater.latest_version:
                result["latest_version"] = str(updater.latest_version)
                result["download_url"] = updater.get_download_url()
                result["release_notes"] = updater.latest_release.get("body", "")

            return result

        except Exception as e:
            logger.error(f"Failed to check for updates: {e}")
            return {"status": "error", "message": str(e)}

    def download_update(self) -> Dict[str, Any]:
        """
        Download and install available update.

        This will download the update, verify it, install it, and restart the app.

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            updater = get_updater()

            if not updater.update_available:
                return {"status": "error", "message": "No update available"}

            # Download and install in background
            async def download_and_install():
                def progress_callback(downloaded: int, total: int):
                    progress = downloaded / total if total > 0 else 0
                    self._run_async(events.push_event("update-progress", {
                        "downloaded": downloaded,
                        "total": total,
                        "progress": progress,
                    }))

                success = await updater.download_and_install(progress_callback)
                if success:
                    await events.push_event("update-installed", {})
                else:
                    await events.push_event("update-failed", {
                        "error": "Installation failed"
                    })

            # Start download in background
            self._run_async(download_and_install())

            return {"status": "success", "message": "Update download started"}

        except Exception as e:
            logger.error(f"Failed to download update: {e}")
            return {"status": "error", "message": str(e)}
