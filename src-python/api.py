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
        """
        Initialize the database.

        Args:
            db_path: Path to the database file

        Returns:
            {"status": "success", "message": "..."}
        """
        try:
            self._app_state.db_path = db_path
            self._run_async(self._app_state.initialize())
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
            job_id = self._run_async(self._app_state.queue_manager.add_job(gen_params))

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
            jobs = self._run_async(self._app_state.queue_manager.get_all_jobs())
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
            job = self._run_async(self._app_state.queue_manager.get_job(job_id))
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
            success = self._run_async(self._app_state.queue_manager.cancel_job(job_id))
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
            self._run_async(self._app_state.queue_manager.clear_completed_jobs())
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
            if not self._app_state.db:
                return []
            images = self._run_async(self._app_state.db.get_all_images(limit))
            return images
        except Exception as e:
            # This is expected for empty/new databases
            logger.debug(f"No images in gallery (database may be empty): {e}")
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
            if not self._app_state.db:
                return None
            image = self._run_async(self._app_state.db.get_image_by_id(image_id))
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
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_image(image_id))
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
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.toggle_favorite(image_id))
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

    # ==================== System Stats ====================

    def get_system_stats(self) -> Dict[str, Any]:
        """
        Get system statistics (CPU, RAM, GPU usage).

        Returns:
            System stats dict
        """
        # TODO: Implement actual system stats collection
        # For now, return mock data
        return {
            "cpu_usage": 25.0,
            "ram_usage": 40.0,
            "ram_total": 16384,  # MB
            "ram_used": 6554,    # MB
            "is_generating": False,
            "gpu_name": None,
            "gpu_usage": 0.0,
            "vram_usage": 0.0,
            "vram_total": 0,
            "vram_used": 0,
        }

    # ==================== Styles ====================

    def get_all_styles(self) -> List[Dict[str, Any]]:
        """
        Get all styles.

        Returns:
            List of style dicts
        """
        # TODO: Implement styles persistence
        # For now, return empty list
        return []

    def create_style(self, style: Dict[str, Any]) -> Dict[str, Any]:
        """
        Create a new style.

        Args:
            style: Style dict

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        # TODO: Implement styles persistence
        logger.info(f"Style creation requested: {style}")
        return {"status": "success", "id": "temp-id"}

    def update_style(self, style_id: str, style: Dict[str, Any]) -> Dict[str, str]:
        """
        Update a style.

        Args:
            style_id: Style ID
            style: Style dict

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement styles persistence
        logger.info(f"Style update requested: {style_id}")
        return {"status": "success"}

    def delete_style(self, style_id: str) -> Dict[str, str]:
        """
        Delete a style.

        Args:
            style_id: Style ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement styles persistence
        logger.info(f"Style deletion requested: {style_id}")
        return {"status": "success"}

    # ==================== Client Mode ====================

    def client_get_queue_jobs(self) -> List[Dict[str, Any]]:
        """
        Get all queue jobs (client mode alias).

        This is an alias for get_all_jobs() for client mode compatibility.

        Returns:
            List of job dicts
        """
        return self.get_all_jobs()

    def get_gallery_images(self, limit: Optional[int] = None) -> List[Dict[str, Any]]:
        """
        Get gallery images (alias for get_all_images).

        Args:
            limit: Optional limit on number of images

        Returns:
            List of image dicts
        """
        return self.get_all_images(limit)

    # ==================== Folders ====================

    def get_folder_tree(self) -> List[Dict[str, Any]]:
        """
        Get folder tree.

        Returns:
            List of folder dicts
        """
        # TODO: Implement folders persistence
        return []

    def create_folder(self, folder: Dict[str, Any]) -> Dict[str, Any]:
        """
        Create a folder.

        Args:
            folder: Folder dict

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        # TODO: Implement folders persistence
        logger.info(f"Folder creation requested: {folder}")
        return {"status": "success", "id": "temp-id"}

    def update_folder(self, folder_id: str, folder: Dict[str, Any]) -> Dict[str, str]:
        """
        Update a folder.

        Args:
            folder_id: Folder ID
            folder: Folder dict

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement folders persistence
        logger.info(f"Folder update requested: {folder_id}")
        return {"status": "success"}

    def delete_folder(self, folder_id: str) -> Dict[str, str]:
        """
        Delete a folder.

        Args:
            folder_id: Folder ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement folders persistence
        logger.info(f"Folder deletion requested: {folder_id}")
        return {"status": "success"}

    # ==================== Tags ====================

    def get_all_tags(self) -> List[Dict[str, Any]]:
        """
        Get all tags.

        Returns:
            List of tag dicts
        """
        # TODO: Implement tags persistence
        return []

    def create_tag(self, tag: Dict[str, Any]) -> Dict[str, Any]:
        """
        Create a tag.

        Args:
            tag: Tag dict

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        # TODO: Implement tags persistence
        logger.info(f"Tag creation requested: {tag}")
        return {"status": "success", "id": "temp-id"}

    def delete_tag(self, tag_id: str) -> Dict[str, str]:
        """
        Delete a tag.

        Args:
            tag_id: Tag ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement tags persistence
        logger.info(f"Tag deletion requested: {tag_id}")
        return {"status": "success"}

    # ==================== Auto-Tag ====================

    def get_auto_tag_settings(self) -> Dict[str, Any]:
        """
        Get auto-tag settings.

        Returns:
            Settings dict
        """
        # TODO: Implement auto-tag settings persistence
        return {
            "enabled": False,
            "auto_tag_on_generation": False,
            "preferred_backend": "claude",
            "min_confidence": 0.6,
        }

    def update_auto_tag_settings(self, settings: Dict[str, Any]) -> Dict[str, str]:
        """
        Update auto-tag settings.

        Args:
            settings: Settings dict

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement auto-tag settings persistence
        logger.info(f"Auto-tag settings update requested: {settings}")
        return {"status": "success"}

    def check_vision_model_status(self) -> Dict[str, Any]:
        """
        Check vision model status.

        Returns:
            Status dict
        """
        # TODO: Implement vision model status check
        return {
            "is_downloaded": False,
            "download_progress": None,
            "model_size": 0,
            "model_size_display": "0 MB",
            "error": None,
        }

    # ==================== Models ====================

    def get_all_models(self) -> List[Dict[str, Any]]:
        """
        Get all models.

        Returns:
            List of model dicts
        """
        # TODO: Implement models listing
        return []

    def download_model(self, model_id: str) -> Dict[str, Any]:
        """
        Download a model.

        Args:
            model_id: Model ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement model download
        logger.info(f"Model download requested: {model_id}")
        return {"status": "success", "message": "Model download started"}

    def delete_model(self, model_id: str) -> Dict[str, str]:
        """
        Delete a model.

        Args:
            model_id: Model ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement model deletion
        logger.info(f"Model deletion requested: {model_id}")
        return {"status": "success"}

    # ==================== Bundles ====================

    def get_all_bundles(self) -> List[Dict[str, Any]]:
        """
        Get all bundles (model collections).

        Returns:
            List of bundle dicts
        """
        # TODO: Implement bundles listing
        return []

    def create_bundle(self, bundle: Dict[str, Any]) -> Dict[str, Any]:
        """
        Create a bundle.

        Args:
            bundle: Bundle dict

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        # TODO: Implement bundle creation
        logger.info(f"Bundle creation requested: {bundle}")
        return {"status": "success", "id": "temp-id"}

    def delete_bundle(self, bundle_id: str) -> Dict[str, str]:
        """
        Delete a bundle.

        Args:
            bundle_id: Bundle ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement bundle deletion
        logger.info(f"Bundle deletion requested: {bundle_id}")
        return {"status": "success"}

    # ==================== LoRAs ====================

    def get_loras(self) -> List[Dict[str, Any]]:
        """
        Get all LoRAs.

        Returns:
            List of LoRA dicts
        """
        # TODO: Implement LoRA listing
        return []

    def add_lora(self, lora: Dict[str, Any]) -> Dict[str, Any]:
        """
        Add a LoRA.

        Args:
            lora: LoRA dict

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        # TODO: Implement LoRA addition
        logger.info(f"LoRA addition requested: {lora}")
        return {"status": "success", "id": "temp-id"}

    def delete_lora(self, lora_id: str) -> Dict[str, str]:
        """
        Delete a LoRA.

        Args:
            lora_id: LoRA ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        # TODO: Implement LoRA deletion
        logger.info(f"LoRA deletion requested: {lora_id}")
        return {"status": "success"}
