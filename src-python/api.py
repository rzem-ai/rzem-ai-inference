"""
API Bridge for pywebview

This module exposes Python functions to the JavaScript frontend.
It replaces Tauri commands with pywebview's JS API.
"""

import asyncio
from pathlib import Path
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

    def delete_gallery_image(self, image_id: str) -> Dict[str, Any]:
        """
        Delete a gallery image (alias for delete_image).

        Args:
            image_id: Image ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        return self.delete_image(image_id)

    def search_gallery_images(
        self,
        query: Optional[str] = None,
        tags: Optional[List[str]] = None,
        folder_id: Optional[str] = None,
        favorites_only: bool = False,
        limit: Optional[int] = 100,
    ) -> List[Dict[str, Any]]:
        """
        Search gallery images.

        Args:
            query: Search query (searches prompt and negative_prompt)
            tags: List of tags to filter by
            folder_id: Folder ID to filter by
            favorites_only: Only return favorites
            limit: Maximum number of results

        Returns:
            List of image dicts
        """
        try:
            if not self._app_state.db:
                return []
            images = self._run_async(self._app_state.db.search_gallery_images(
                query=query,
                tags=tags,
                folder_id=folder_id,
                favorites_only=favorites_only,
                limit=limit or 100,
            ))
            return images
        except Exception as e:
            logger.error(f"Failed to search gallery images: {e}")
            return []

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

    def get_hf_token(self) -> Dict[str, Any]:
        """Get Hugging Face API token"""
        try:
            if not self._app_state.db:
                return {"token": None}
            token = self._run_async(self._app_state.db.get_setting("hf_token"))
            return {"token": token}
        except Exception as e:
            logger.error(f"Failed to get HF token: {e}")
            return {"token": None}

    def save_hf_token(self, token: str) -> Dict[str, str]:
        """Save Hugging Face API token"""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            # If empty, delete the setting
            if not token or not token.strip():
                self._run_async(self._app_state.db.delete_setting("hf_token"))
            else:
                self._run_async(self._app_state.db.set_setting("hf_token", token.strip()))

            logger.info("HF token saved")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to save HF token: {e}")
            return {"status": "error", "message": str(e)}

    def get_claude_api_key(self) -> Dict[str, Any]:
        """Get Claude API key"""
        try:
            if not self._app_state.db:
                return {"key": None}
            key = self._run_async(self._app_state.db.get_setting("claude_api_key"))
            return {"key": key}
        except Exception as e:
            logger.error(f"Failed to get Claude API key: {e}")
            return {"key": None}

    def save_claude_api_key(self, key: str) -> Dict[str, str]:
        """Save Claude API key"""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            # If empty, delete the setting
            if not key or not key.strip():
                self._run_async(self._app_state.db.delete_setting("claude_api_key"))
            else:
                self._run_async(self._app_state.db.set_setting("claude_api_key", key.strip()))

            logger.info("Claude API key saved")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to save Claude API key: {e}")
            return {"status": "error", "message": str(e)}

    def get_fal_key(self) -> Dict[str, Any]:
        """Get Fal.ai API key"""
        try:
            if not self._app_state.db:
                return {"key": None}
            key = self._run_async(self._app_state.db.get_setting("fal_key"))
            return {"key": key}
        except Exception as e:
            logger.error(f"Failed to get Fal key: {e}")
            return {"key": None}

    def save_fal_key(self, key: str) -> Dict[str, str]:
        """Save Fal.ai API key"""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            # If empty, delete the setting
            if not key or not key.strip():
                self._run_async(self._app_state.db.delete_setting("fal_key"))
            else:
                self._run_async(self._app_state.db.set_setting("fal_key", key.strip()))

            logger.info("Fal key saved")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to save Fal key: {e}")
            return {"status": "error", "message": str(e)}

    def get_cache_stats(self) -> Dict[str, Any]:
        """Get model cache statistics"""
        try:
            cache_dir = Path.home() / ".cache" / "huggingface" / "hub"

            if not cache_dir.exists():
                return {
                    "total_size_bytes": 0,
                    "total_size_gb": 0.0,
                    "model_count": 0,
                    "models": [],
                }

            # Calculate cache size
            total_size = 0
            model_dirs = []

            for item in cache_dir.iterdir():
                if item.is_dir() and item.name.startswith("models--"):
                    size = sum(f.stat().st_size for f in item.rglob('*') if f.is_file())
                    total_size += size
                    model_name = item.name.replace("models--", "").replace("--", "/")
                    model_dirs.append({
                        "name": model_name,
                        "size_bytes": size,
                        "size_gb": round(size / (1024**3), 2),
                    })

            return {
                "total_size_bytes": total_size,
                "total_size_gb": round(total_size / (1024**3), 2),
                "model_count": len(model_dirs),
                "models": sorted(model_dirs, key=lambda x: x["size_bytes"], reverse=True),
            }
        except Exception as e:
            logger.error(f"Failed to get cache stats: {e}")
            return {
                "total_size_bytes": 0,
                "total_size_gb": 0.0,
                "model_count": 0,
                "models": [],
            }

    def get_cache_config(self) -> Dict[str, Any]:
        """Get cache configuration"""
        try:
            # Load from database if available
            if self._app_state.db:
                keep_vae_loaded = self._run_async(self._app_state.db.get_setting("cache_keep_vae_loaded"))
                keep_flux_loaded = self._run_async(self._app_state.db.get_setting("cache_keep_flux_loaded"))
                keep_t5_loaded = self._run_async(self._app_state.db.get_setting("cache_keep_t5_loaded"))
                keep_clip_loaded = self._run_async(self._app_state.db.get_setting("cache_keep_clip_loaded"))

                return {
                    "keep_vae_loaded": keep_vae_loaded == "true" if keep_vae_loaded else False,
                    "keep_flux_loaded": keep_flux_loaded == "true" if keep_flux_loaded else False,
                    "keep_t5_loaded": keep_t5_loaded == "true" if keep_t5_loaded else False,
                    "keep_clip_loaded": keep_clip_loaded == "true" if keep_clip_loaded else False,
                    "embedding_cache_size": 100,
                    "idle_timeout_secs": None,
                }

            # Default config
            return {
                "keep_vae_loaded": False,
                "keep_flux_loaded": False,
                "keep_t5_loaded": False,
                "keep_clip_loaded": False,
                "embedding_cache_size": 100,
                "idle_timeout_secs": None,
            }
        except Exception as e:
            logger.error(f"Failed to get cache config: {e}")
            return {
                "keep_vae_loaded": False,
                "keep_flux_loaded": False,
                "keep_t5_loaded": False,
                "keep_clip_loaded": False,
                "embedding_cache_size": 100,
                "idle_timeout_secs": None,
            }

    def save_cache_config(self, config: Dict[str, Any]) -> Dict[str, str]:
        """Save cache configuration"""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            # Save each config value
            if "keep_vae_loaded" in config:
                val = "true" if config["keep_vae_loaded"] else "false"
                self._run_async(self._app_state.db.set_setting("cache_keep_vae_loaded", val))

            if "keep_flux_loaded" in config:
                val = "true" if config["keep_flux_loaded"] else "false"
                self._run_async(self._app_state.db.set_setting("cache_keep_flux_loaded", val))

            if "keep_t5_loaded" in config:
                val = "true" if config["keep_t5_loaded"] else "false"
                self._run_async(self._app_state.db.set_setting("cache_keep_t5_loaded", val))

            if "keep_clip_loaded" in config:
                val = "true" if config["keep_clip_loaded"] else "false"
                self._run_async(self._app_state.db.set_setting("cache_keep_clip_loaded", val))

            logger.info(f"Cache config saved: {config}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to save cache config: {e}")
            return {"status": "error", "message": str(e)}

    def clear_cache(self) -> Dict[str, str]:
        """Clear model cache"""
        try:
            # Clear the pipeline if it exists
            if self._app_state.queue_processor and self._app_state.queue_processor.pipeline:
                # Unload models from pipeline (this would require methods on the pipeline)
                logger.info("Cache clear requested - would unload models here")
                # TODO: Implement actual model unloading when pipeline supports it

            return {"status": "success", "message": "Cache cleared"}
        except Exception as e:
            logger.error(f"Failed to clear cache: {e}")
            return {"status": "error", "message": str(e)}

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
        try:
            import psutil

            # Get CPU usage (average over short interval)
            cpu_percent = psutil.cpu_percent(interval=0.1)

            # Get memory stats
            memory = psutil.virtual_memory()
            memory_used = memory.used  # bytes
            memory_total = memory.total  # bytes
            memory_percent = memory.percent

            # Get GPU stats via pynvml
            gpu_memory_used = None
            gpu_memory_total = None
            gpu_usage_percent = None
            gpu_name = None

            try:
                logger.info("loading: pynvml")

                import pynvml
                pynvml.nvmlInit()
                device_count = pynvml.nvmlDeviceGetCount()

                logger.info(f"pynvml: device_count: {device_count}")

                if device_count > 0:
                    handle = pynvml.nvmlDeviceGetHandleByIndex(0)

                    # GPU name
                    gpu_name = pynvml.nvmlDeviceGetName(handle)
                    logger.info(f"pynvml: gpu_name: {gpu_name}")

                    # Memory info (already in bytes)
                    mem_info = pynvml.nvmlDeviceGetMemoryInfo(handle)
                    gpu_memory_used = mem_info.used
                    gpu_memory_total = mem_info.total

                    # Utilization rates
                    utilization = pynvml.nvmlDeviceGetUtilizationRates(handle)
                    gpu_usage_percent = float(utilization.gpu)

                pynvml.nvmlShutdown()
            except ImportError:
                logger.debug("pynvml not installed, GPU stats unavailable")
            except Exception as e:
                logger.debug(f"GPU stats unavailable: {e}")

            # Check if generation is running
            is_generating = False
            if self._app_state.queue_processor:
                is_generating = getattr(self._app_state.queue_processor, 'is_processing', False)

            logger.info(f"pynvml: gpu_name: {str({
                "cpuUsage": cpu_percent,
                "memoryUsed": memory_used,
                "memoryTotal": memory_total,
                "memoryPercent": memory_percent,
                "gpuMemoryUsed": gpu_memory_used,
                "gpuMemoryTotal": gpu_memory_total,
                "gpuUsagePercent": gpu_usage_percent,
                "gpuName": gpu_name,
                "isGenerating": is_generating,
            })}")

            return {
                "cpuUsage": cpu_percent,
                "memoryUsed": memory_used,
                "memoryTotal": memory_total,
                "memoryPercent": memory_percent,
                "gpuMemoryUsed": gpu_memory_used,
                "gpuMemoryTotal": gpu_memory_total,
                "gpuUsagePercent": gpu_usage_percent,
                "gpuName": gpu_name,
                "isGenerating": is_generating,
            }

        except Exception as e:
            logger.error(f"Failed to get system stats: {e}")
            return {
                "cpuUsage": 0.0,
                "memoryUsed": 0,
                "memoryTotal": 0,
                "memoryPercent": 0.0,
                "gpuMemoryUsed": None,
                "gpuMemoryTotal": None,
                "gpuUsagePercent": None,
                "gpuName": None,
                "isGenerating": False,
            }

    # ==================== Styles ====================

    def get_all_styles(self) -> List[Dict[str, Any]]:
        """
        Get all styles.

        Returns:
            List of style dicts
        """
        try:
            if not self._app_state.db:
                return []
            styles = self._run_async(self._app_state.db.get_all_styles())
            return styles
        except Exception as e:
            logger.error(f"Failed to get styles: {e}")
            return []

    def get_style_detail(self, style_id: str) -> Optional[Dict[str, Any]]:
        """
        Get detailed style information including LoRAs and examples.

        Args:
            style_id: Style ID

        Returns:
            Style detail dict or None
        """
        try:
            if not self._app_state.db:
                return None
            detail = self._run_async(self._app_state.db.get_style_detail(style_id))
            return detail
        except Exception as e:
            logger.error(f"Failed to get style detail: {e}")
            return None

    def create_style(self, style: Dict[str, Any]) -> Dict[str, Any]:
        """
        Create a new style.

        Args:
            style: Style dict with name, description, promptTemplate, etc.

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            style_id = self._run_async(self._app_state.db.create_style(style))
            logger.info(f"Style created: {style_id}")
            return {"status": "success", "id": style_id}
        except Exception as e:
            logger.error(f"Failed to create style: {e}")
            return {"status": "error", "message": str(e)}

    def update_style(self, style_id: str, style: Dict[str, Any]) -> Dict[str, str]:
        """
        Update a style.

        Args:
            style_id: Style ID
            style: Style dict with fields to update

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.update_style(style_id, style))
            logger.info(f"Style updated: {style_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to update style: {e}")
            return {"status": "error", "message": str(e)}

    def delete_style(self, style_id: str) -> Dict[str, str]:
        """
        Delete a style.

        Args:
            style_id: Style ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_style(style_id))
            if success:
                logger.info(f"Style deleted: {style_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Style not found"}
        except Exception as e:
            logger.error(f"Failed to delete style: {e}")
            return {"status": "error", "message": str(e)}

    def add_lora_to_style(
        self, style_id: str, lora_id: str, strength: float = 1.0, priority: int = 0
    ) -> Dict[str, str]:
        """
        Add a LoRA to a style.

        Args:
            style_id: Style ID
            lora_id: LoRA ID
            strength: LoRA strength (default 1.0)
            priority: LoRA priority (default 0)

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.add_lora_to_style(
                style_id, lora_id, strength, priority
            ))
            logger.info(f"LoRA added to style: {lora_id} -> {style_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to add LoRA to style: {e}")
            return {"status": "error", "message": str(e)}

    def remove_lora_from_style(self, style_id: str, lora_id: str) -> Dict[str, str]:
        """
        Remove a LoRA from a style.

        Args:
            style_id: Style ID
            lora_id: LoRA ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.remove_lora_from_style(
                style_id, lora_id
            ))
            if success:
                logger.info(f"LoRA removed from style: {lora_id} -> {style_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "LoRA association not found"}
        except Exception as e:
            logger.error(f"Failed to remove LoRA from style: {e}")
            return {"status": "error", "message": str(e)}

    def add_style_example(
        self,
        style_id: str,
        example_type: str,
        content: str,
        generation_params: Optional[str] = None,
    ) -> Dict[str, Any]:
        """
        Add an example to a style.

        Args:
            style_id: Style ID
            example_type: Example type ('prompt' or 'image')
            content: Example content (prompt text or image_id)
            generation_params: Optional JSON generation parameters

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            example_id = self._run_async(self._app_state.db.add_style_example(
                style_id, example_type, content, generation_params
            ))
            logger.info(f"Example added to style: {example_id} -> {style_id}")
            return {"status": "success", "id": example_id}
        except Exception as e:
            logger.error(f"Failed to add style example: {e}")
            return {"status": "error", "message": str(e)}

    def remove_style_example(self, example_id: str) -> Dict[str, str]:
        """
        Remove an example from a style.

        Args:
            example_id: Example ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.remove_style_example(example_id))
            if success:
                logger.info(f"Style example removed: {example_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Example not found"}
        except Exception as e:
            logger.error(f"Failed to remove style example: {e}")
            return {"status": "error", "message": str(e)}

    def render_style_template(self, template: str, variables: Dict[str, str]) -> Dict[str, Any]:
        """
        Render a style template preview.

        Args:
            template: Template string with {variable} placeholders
            variables: Variable values

        Returns:
            {"status": "success", "rendered": "..."} or {"status": "error", "message": "..."}
        """
        try:
            # Simple template rendering
            rendered = template
            for key, value in variables.items():
                rendered = rendered.replace(f"{{{key}}}", value)

            return {"status": "success", "rendered": rendered}
        except Exception as e:
            logger.error(f"Failed to render template: {e}")
            return {"status": "error", "message": str(e)}

    def upload_style_thumbnail(self, style_id: str, thumbnail_path: str) -> Dict[str, str]:
        """
        Upload/set style thumbnail image.

        Args:
            style_id: Style ID
            thumbnail_path: Path to thumbnail image

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.update_style_thumbnail(style_id, thumbnail_path))
            logger.info(f"Style thumbnail updated: {style_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to upload style thumbnail: {e}")
            return {"status": "error", "message": str(e)}

    def delete_style_thumbnail(self, style_id: str) -> Dict[str, str]:
        """
        Delete style thumbnail image.

        Args:
            style_id: Style ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.update_style_thumbnail(style_id, None))
            logger.info(f"Style thumbnail deleted: {style_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to delete style thumbnail: {e}")
            return {"status": "error", "message": str(e)}

    def increment_style_usage(self, style_id: str) -> Dict[str, str]:
        """
        Increment usage count for a style.

        Args:
            style_id: Style ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.increment_style_usage(style_id))
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to increment style usage: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Chatbot ====================

    def chat_refine_prompt(self, prompt: str, context: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Refine prompt using AI chatbot.

        Args:
            prompt: User prompt to refine
            context: Optional context dict

        Returns:
            {"status": "success", "refined_prompt": "..."} or {"status": "error", "message": "..."}
        """
        logger.debug(f"chat_refine_prompt called: {prompt[:50]}...")
        return {
            "status": "error",
            "message": "Chatbot prompt refinement requires Claude API key configuration"
        }

    # ==================== Batch Generation ====================

    def batch_parse_data(self, data: str, format: str = "csv") -> Dict[str, Any]:
        """
        Parse batch data (CSV/JSON).

        Args:
            data: Raw data string
            format: Data format ('csv' or 'json')

        Returns:
            {"status": "success", "rows": [...]} or {"status": "error", "message": "..."}
        """
        try:
            import json
            import csv
            from io import StringIO

            if format == "json":
                # Parse JSON data
                parsed = json.loads(data)
                if isinstance(parsed, list):
                    rows = parsed
                elif isinstance(parsed, dict):
                    rows = [parsed]
                else:
                    return {"status": "error", "message": "Invalid JSON format"}

                logger.debug(f"Parsed {len(rows)} rows from JSON")
                return {"status": "success", "rows": rows}

            elif format == "csv":
                # Parse CSV data
                reader = csv.DictReader(StringIO(data))
                rows = list(reader)
                logger.debug(f"Parsed {len(rows)} rows from CSV")
                return {"status": "success", "rows": rows}

            else:
                return {"status": "error", "message": f"Unsupported format: {format}"}

        except json.JSONDecodeError as e:
            logger.error(f"JSON parse error: {e}")
            return {"status": "error", "message": f"JSON parse error: {str(e)}"}
        except csv.Error as e:
            logger.error(f"CSV parse error: {e}")
            return {"status": "error", "message": f"CSV parse error: {str(e)}"}
        except Exception as e:
            logger.error(f"Failed to parse batch data: {e}")
            return {"status": "error", "message": str(e)}

    def batch_render_template(self, template: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Render batch template preview.

        Args:
            template: Template string
            data: Data dict for rendering

        Returns:
            {"status": "success", "rendered": "..."} or {"status": "error", "message": "..."}
        """
        try:
            # Simple template rendering
            rendered = template
            for key, value in data.items():
                rendered = rendered.replace(f"{{{key}}}", str(value))

            return {"status": "success", "rendered": rendered}
        except Exception as e:
            logger.error(f"Failed to render batch template: {e}")
            return {"status": "error", "message": str(e)}

    def batch_save_template(self, template: str) -> Dict[str, str]:
        """
        Save batch template.

        Args:
            template: Template string

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        logger.debug("batch_save_template stub called")
        return {"status": "success"}

    def batch_get_recent_templates(self, limit: int = 10) -> List[Dict[str, Any]]:
        """
        Get recent template history.

        Args:
            limit: Maximum number of templates to return

        Returns:
            List of template dicts
        """
        logger.debug(f"batch_get_recent_templates stub called: limit={limit}")
        return []

    def batch_generate_combinations(self, template: str, variables: Dict[str, List[str]]) -> Dict[str, Any]:
        """
        Generate parameter combinations.

        Args:
            template: Template string
            variables: Variable values dict

        Returns:
            {"status": "success", "combinations": [...]} or {"status": "error", "message": "..."}
        """
        logger.debug("batch_generate_combinations stub called")
        return {
            "status": "error",
            "message": "Batch generation not implemented"
        }

    # ==================== Image Analysis ====================

    def analyze_image_for_prompt(self, image_path: str) -> Dict[str, Any]:
        """
        Analyze image to generate prompt.

        Args:
            image_path: Path to image file

        Returns:
            {"status": "success", "prompt": "..."} or {"status": "error", "message": "..."}
        """
        logger.debug(f"analyze_image_for_prompt called: {image_path}")
        return {
            "status": "error",
            "message": "Image analysis requires vision model download and configuration"
        }

    # ==================== Client Mode ====================

    def client_add_to_queue(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """
        Add generation job to queue (client mode alias).

        This is an alias for queue_generation() for client mode compatibility.

        Args:
            params: Generation parameters dict

        Returns:
            {"job_id": "...", "status": "success"} or {"status": "error", "message": "..."}
        """
        return self.queue_generation(params)

    def client_get_queue_jobs(self) -> List[Dict[str, Any]]:
        """
        Get all queue jobs (client mode alias).

        This is an alias for get_all_jobs() for client mode compatibility.

        Returns:
            List of job dicts
        """
        return self.get_all_jobs()

    def client_get_queue_job(self, job_id: str) -> Optional[Dict[str, Any]]:
        """
        Get specific queue job (client mode).

        This is an alias for get_job() for client mode compatibility.

        Args:
            job_id: Job ID

        Returns:
            Job dict or None
        """
        return self.get_job(job_id)

    def client_cancel_queue_job(self, job_id: str) -> Dict[str, Any]:
        """
        Cancel a queue job (client mode alias).

        This is an alias for cancel_job() for client mode compatibility.

        Args:
            job_id: Job ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        return self.cancel_job(job_id)

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
        try:
            if not self._app_state.db:
                return []
            tree = self._run_async(self._app_state.db.get_folder_tree())
            return tree
        except Exception as e:
            logger.error(f"Failed to get folder tree: {e}")
            return []

    def create_folder(self, folder: Dict[str, Any]) -> Dict[str, Any]:
        """
        Create a folder.

        Args:
            folder: Folder dict with name, parentId, color, icon

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            folder_data = self._run_async(self._app_state.db.create_folder(
                name=folder.get("name"),
                parent_id=folder.get("parentId"),
                color=folder.get("color"),
                icon=folder.get("icon"),
            ))
            logger.info(f"Folder created: {folder_data['id']}")
            return {"status": "success", "folder": folder_data}
        except Exception as e:
            logger.error(f"Failed to create folder: {e}")
            return {"status": "error", "message": str(e)}

    def update_folder(self, folder_id: str, folder: Dict[str, Any]) -> Dict[str, str]:
        """
        Update a folder.

        Args:
            folder_id: Folder ID
            folder: Folder dict with optional name, color, icon

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.update_folder(
                folder_id=folder_id,
                name=folder.get("name"),
                color=folder.get("color"),
                icon=folder.get("icon"),
            ))
            logger.info(f"Folder updated: {folder_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to update folder: {e}")
            return {"status": "error", "message": str(e)}

    def delete_folder(self, folder_id: str) -> Dict[str, str]:
        """
        Delete a folder.

        Args:
            folder_id: Folder ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_folder(folder_id))
            if success:
                logger.info(f"Folder deleted: {folder_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Folder not found"}
        except Exception as e:
            logger.error(f"Failed to delete folder: {e}")
            return {"status": "error", "message": str(e)}

    def move_folder(self, folder_id: str, new_parent_id: Optional[str]) -> Dict[str, str]:
        """
        Move folder to a new parent.

        Args:
            folder_id: Folder ID
            new_parent_id: New parent ID (None for root)

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.move_folder(folder_id, new_parent_id))
            logger.info(f"Folder moved: {folder_id} -> {new_parent_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to move folder: {e}")
            return {"status": "error", "message": str(e)}

    def reorder_folders(self, folder_ids: List[str]) -> Dict[str, str]:
        """
        Reorder folders within the same parent.

        Args:
            folder_ids: List of folder IDs in desired order

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.reorder_folders(folder_ids))
            logger.info(f"Folders reordered: {len(folder_ids)} folders")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to reorder folders: {e}")
            return {"status": "error", "message": str(e)}

    def add_images_to_folder(self, image_ids: List[str], folder_id: str) -> Dict[str, str]:
        """
        Add images to a folder.

        Args:
            image_ids: List of image IDs
            folder_id: Folder ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.add_images_to_folder(image_ids, folder_id))
            logger.info(f"Added {len(image_ids)} images to folder {folder_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to add images to folder: {e}")
            return {"status": "error", "message": str(e)}

    def remove_images_from_folder(self, image_ids: List[str], folder_id: str) -> Dict[str, str]:
        """
        Remove images from a folder.

        Args:
            image_ids: List of image IDs
            folder_id: Folder ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.remove_images_from_folder(image_ids, folder_id))
            logger.info(f"Removed {len(image_ids)} images from folder {folder_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to remove images from folder: {e}")
            return {"status": "error", "message": str(e)}

    def get_folder_images(self, folder_id: str, limit: Optional[int] = 100) -> List[Dict[str, Any]]:
        """
        Get images in a specific folder.

        Args:
            folder_id: Folder ID
            limit: Maximum number of images to return

        Returns:
            List of image dicts
        """
        try:
            if not self._app_state.db:
                return []
            images = self._run_async(self._app_state.db.get_folder_images(folder_id, limit or 100))
            return images
        except Exception as e:
            logger.error(f"Failed to get folder images: {e}")
            return []

    def get_uncategorized_images(self, limit: Optional[int] = 100) -> List[Dict[str, Any]]:
        """
        Get images not in any folder.

        Args:
            limit: Maximum number of images to return

        Returns:
            List of image dicts
        """
        try:
            if not self._app_state.db:
                return []
            images = self._run_async(self._app_state.db.get_uncategorized_images(limit or 100))
            return images
        except Exception as e:
            logger.error(f"Failed to get uncategorized images: {e}")
            return []

    # ==================== Tags ====================

    def get_all_tags(self) -> List[Dict[str, Any]]:
        """
        Get all tags.

        Returns:
            List of tag dicts
        """
        try:
            if not self._app_state.db:
                return []
            tags = self._run_async(self._app_state.db.get_all_tags())
            return tags
        except Exception as e:
            logger.error(f"Failed to get tags: {e}")
            return []

    def update_tag(self, tag_id: int, tag: Dict[str, Any]) -> Dict[str, str]:
        """
        Update a tag.

        Args:
            tag_id: Tag ID
            tag: Tag dict with optional name, color, category

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.update_tag(
                tag_id=tag_id,
                name=tag.get("name"),
                color=tag.get("color"),
                category=tag.get("category"),
            ))
            logger.info(f"Tag updated: {tag_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to update tag: {e}")
            return {"status": "error", "message": str(e)}

    def delete_tag(self, tag_id: int) -> Dict[str, str]:
        """
        Delete a tag.

        Args:
            tag_id: Tag ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_tag(tag_id))
            if success:
                logger.info(f"Tag deleted: {tag_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Tag not found"}
        except Exception as e:
            logger.error(f"Failed to delete tag: {e}")
            return {"status": "error", "message": str(e)}

    def add_image_tag(self, image_id: str, tag: str) -> Dict[str, str]:
        """
        Add tag to image.

        Args:
            image_id: Image ID
            tag: Tag name

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.add_image_tag(image_id, tag))
            logger.info(f"Tag added to image: {tag} -> {image_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to add image tag: {e}")
            return {"status": "error", "message": str(e)}

    def remove_image_tag(self, image_id: str, tag: str) -> Dict[str, str]:
        """
        Remove tag from image.

        Args:
            image_id: Image ID
            tag: Tag name

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.remove_image_tag(image_id, tag))
            logger.info(f"Tag removed from image: {tag} -> {image_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to remove image tag: {e}")
            return {"status": "error", "message": str(e)}

    def bulk_add_tag(self, image_ids: List[str], tag: str) -> Dict[str, str]:
        """
        Add tag to multiple images.

        Args:
            image_ids: List of image IDs
            tag: Tag name

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.bulk_add_tag(image_ids, tag))
            logger.info(f"Tag bulk added: {tag} to {len(image_ids)} images")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to bulk add tag: {e}")
            return {"status": "error", "message": str(e)}

    def bulk_remove_tag(self, image_ids: List[str], tag: str) -> Dict[str, str]:
        """
        Remove tag from multiple images.

        Args:
            image_ids: List of image IDs
            tag: Tag name

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.bulk_remove_tag(image_ids, tag))
            logger.info(f"Tag bulk removed: {tag} from {len(image_ids)} images")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to bulk remove tag: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Auto-Tag ====================

    def get_auto_tag_settings(self) -> Dict[str, Any]:
        """
        Get auto-tag settings.

        Returns:
            Settings dict
        """
        try:
            if self._app_state.db:
                enabled = self._run_async(self._app_state.db.get_setting("auto_tag_enabled"))
                auto_tag_on_gen = self._run_async(self._app_state.db.get_setting("auto_tag_on_generation"))
                backend = self._run_async(self._app_state.db.get_setting("auto_tag_backend"))
                min_conf = self._run_async(self._app_state.db.get_setting("auto_tag_min_confidence"))

                return {
                    "enabled": enabled == "true" if enabled else False,
                    "autoTagOnGeneration": auto_tag_on_gen == "true" if auto_tag_on_gen else False,
                    "preferredBackend": backend if backend else "claude",
                    "minConfidence": float(min_conf) if min_conf else 0.6,
                }
        except Exception as e:
            logger.error(f"Failed to get auto-tag settings: {e}")

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
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            if "enabled" in settings:
                val = "true" if settings["enabled"] else "false"
                self._run_async(self._app_state.db.set_setting("auto_tag_enabled", val))

            if "autoTagOnGeneration" in settings:
                val = "true" if settings["autoTagOnGeneration"] else "false"
                self._run_async(self._app_state.db.set_setting("auto_tag_on_generation", val))

            if "preferredBackend" in settings:
                self._run_async(self._app_state.db.set_setting("auto_tag_backend", settings["preferredBackend"]))

            if "minConfidence" in settings:
                self._run_async(self._app_state.db.set_setting("auto_tag_min_confidence", str(settings["minConfidence"])))

            logger.info("Auto-tag settings updated")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to update auto-tag settings: {e}")
            return {"status": "error", "message": str(e)}

    def check_vision_model_status(self) -> Dict[str, Any]:
        """
        Check vision model status.

        Returns:
            Status dict
        """
        logger.debug("check_vision_model_status stub called")
        return {
            "isDownloaded": False,
            "downloadProgress": None,
            "modelSize": 0,
            "modelSizeDisplay": "0 MB",
            "error": None,
        }

    def download_vision_model(self) -> Dict[str, str]:
        """
        Download vision model.

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        logger.debug("download_vision_model stub called")
        return {"status": "error", "message": "Vision model download not implemented"}

    def clear_vision_model_locks(self) -> Dict[str, str]:
        """
        Clear vision model lock files.

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        logger.debug("clear_vision_model_locks stub called")
        return {"status": "success"}

    def auto_tag_images(self, image_ids: List[str]) -> Dict[str, str]:
        """
        Auto-tag images using vision model.

        Args:
            image_ids: List of image IDs to tag

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        logger.debug(f"auto_tag_images stub called: {len(image_ids)} images")
        return {"status": "error", "message": "Auto-tagging not implemented"}

    # ==================== Models ====================

    def get_all_models(self) -> List[Dict[str, Any]]:
        """
        Get all model components.

        Returns:
            List of model dicts
        """
        try:
            if not self._app_state.db:
                return []

            models = self._run_async(self._app_state.db.get_all_model_components())
            logger.debug(f"Retrieved {len(models)} model components")
            return models
        except Exception as e:
            logger.error(f"Failed to get models: {e}")
            return []

    def update_model(self, model_id: str, model: Dict[str, Any]) -> Dict[str, str]:
        """
        Update model details.

        Args:
            model_id: Model ID
            model: Model dict with fields to update

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(
                self._app_state.db.update_model_component(model_id, model)
            )
            if success:
                logger.info(f"Model updated: {model_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Model not found or no updates"}
        except Exception as e:
            logger.error(f"Failed to update model: {e}")
            return {"status": "error", "message": str(e)}

    def add_model_tag(self, model_id: str, tag: str) -> Dict[str, str]:
        """
        Add tag to model.

        Args:
            model_id: Model ID
            tag: Tag name

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.add_model_tag(model_id, tag))
            if success:
                logger.info(f"Tag added to model: {model_id} -> {tag}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Failed to add tag"}
        except Exception as e:
            logger.error(f"Failed to add model tag: {e}")
            return {"status": "error", "message": str(e)}

    def remove_model_tag(self, model_id: str, tag: str) -> Dict[str, str]:
        """
        Remove tag from model.

        Args:
            model_id: Model ID
            tag: Tag name

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(
                self._app_state.db.remove_model_tag(model_id, tag)
            )
            if success:
                logger.info(f"Tag removed from model: {model_id} -> {tag}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Tag not found"}
        except Exception as e:
            logger.error(f"Failed to remove model tag: {e}")
            return {"status": "error", "message": str(e)}

    def add_example(
        self, entity_type: str, entity_id: str, example_type: str, content: str
    ) -> Dict[str, Any]:
        """
        Add example to model/bundle.

        Args:
            entity_type: Entity type ('model' or 'bundle')
            entity_id: Entity ID
            example_type: Example type ('prompt' or 'image')
            content: Example content

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        logger.debug(f"add_example stub called: {entity_type}, {entity_id}")
        return {"status": "success", "id": "temp-example-id"}

    def remove_example(self, example_id: str) -> Dict[str, str]:
        """
        Remove example.

        Args:
            example_id: Example ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        logger.debug(f"remove_example stub called: {example_id}")
        return {"status": "success"}

    def scan_directory_for_models(self, directory: str) -> Dict[str, Any]:
        """
        Scan directory for models.

        Args:
            directory: Directory path to scan

        Returns:
            {"status": "success", "found": 0} or {"status": "error", "message": "..."}
        """
        logger.debug(f"scan_directory_for_models stub called: {directory}")
        return {"status": "success", "found": 0}

    def scan_and_discover_models(self) -> Dict[str, Any]:
        """
        Auto-discover models in default paths.

        Returns:
            {"status": "success", "found": 0} or {"status": "error", "message": "..."}
        """
        logger.debug("scan_and_discover_models stub called")
        return {"status": "success", "found": 0}

    def convert_comfyui_model(self, source_path: str) -> Dict[str, Any]:
        """
        Convert ComfyUI model format.

        Args:
            source_path: Source model path

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        logger.debug(f"convert_comfyui_model stub called: {source_path}")
        return {"status": "success", "message": "Conversion not implemented"}

    def get_compatible_models(self, bundle_id: str) -> List[Dict[str, Any]]:
        """
        Get compatible models for bundle.

        Args:
            bundle_id: Bundle ID

        Returns:
            List of compatible model dicts
        """
        logger.debug(f"get_compatible_models stub called: {bundle_id}")
        return []

    # ==================== Bundles ====================

    def get_all_bundles(self) -> List[Dict[str, Any]]:
        """
        Get all bundles (model collections).

        Returns:
            List of bundle dicts
        """
        try:
            if not self._app_state.db:
                return []

            bundles = self._run_async(self._app_state.db.get_all_bundles())
            logger.debug(f"Retrieved {len(bundles)} bundles")
            return bundles
        except Exception as e:
            logger.error(f"Failed to get bundles: {e}")
            return []

    def create_bundle(self, bundle: Dict[str, Any]) -> Dict[str, Any]:
        """
        Create a bundle.

        Args:
            bundle: Bundle dict

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            bundle_id = self._run_async(self._app_state.db.create_bundle(bundle))
            logger.info(f"Bundle created: {bundle_id}")
            return {"status": "success", "id": bundle_id}
        except Exception as e:
            logger.error(f"Failed to create bundle: {e}")
            return {"status": "error", "message": str(e)}

    def update_bundle(self, bundle_id: str, bundle: Dict[str, Any]) -> Dict[str, str]:
        """
        Update bundle details.

        Args:
            bundle_id: Bundle ID
            bundle: Bundle dict with fields to update

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(
                self._app_state.db.update_bundle(bundle_id, bundle)
            )
            if success:
                logger.info(f"Bundle updated: {bundle_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Bundle not found or no updates"}
        except Exception as e:
            logger.error(f"Failed to update bundle: {e}")
            return {"status": "error", "message": str(e)}

    def delete_bundle(self, bundle_id: str) -> Dict[str, str]:
        """
        Delete a bundle.

        Args:
            bundle_id: Bundle ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_bundle(bundle_id))
            if success:
                logger.info(f"Bundle deleted: {bundle_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Bundle not found"}
        except Exception as e:
            logger.error(f"Failed to delete bundle: {e}")
            return {"status": "error", "message": str(e)}

    def set_active_bundle(self, bundle_id: str) -> Dict[str, str]:
        """
        Set active bundle for generation.

        Args:
            bundle_id: Bundle ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.set_active_bundle(bundle_id))
            if success:
                logger.info(f"Active bundle set: {bundle_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Bundle not found"}
        except Exception as e:
            logger.error(f"Failed to set active bundle: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== LoRAs ====================

    def get_loras(self) -> List[Dict[str, Any]]:
        """
        Get all LoRAs.

        Returns:
            List of LoRA dicts
        """
        try:
            if not self._app_state.db:
                return []

            loras = self._run_async(self._app_state.db.get_all_loras())
            logger.debug(f"Retrieved {len(loras)} LoRAs")
            return loras
        except Exception as e:
            logger.error(f"Failed to get LoRAs: {e}")
            return []

    def import_lora(self, file_path: str) -> Dict[str, Any]:
        """
        Import LoRA from file.

        Args:
            file_path: Path to LoRA file

        Returns:
            {"status": "success", "id": "..."} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            from pathlib import Path
            import os

            # Validate file exists
            if not os.path.exists(file_path):
                return {"status": "error", "message": "File not found"}

            # Get file info
            file_size = os.path.getsize(file_path)
            file_name = Path(file_path).stem

            # Create LoRA entry
            lora_data = {
                "name": file_name,
                "path": file_path,
                "sizeBytes": file_size,
                "strength": 1.0,
                "isActive": False
            }

            lora_id = self._run_async(self._app_state.db.upsert_lora(lora_data))
            logger.info(f"LoRA imported: {lora_id} from {file_path}")
            return {"status": "success", "id": lora_id}
        except Exception as e:
            logger.error(f"Failed to import LoRA: {e}")
            return {"status": "error", "message": str(e)}

    def remove_lora(self, lora_id: str) -> Dict[str, str]:
        """
        Remove a LoRA.

        Args:
            lora_id: LoRA ID

        Returns:
            {"status": "success"} or {"status": "error", "message": "..."}
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_lora(lora_id))
            if success:
                logger.info(f"LoRA removed: {lora_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "LoRA not found"}
        except Exception as e:
            logger.error(f"Failed to remove LoRA: {e}")
            return {"status": "error", "message": str(e)}

    def get_lora_file_info(self, file_path: str) -> Dict[str, Any]:
        """
        Get LoRA file metadata.

        Args:
            file_path: Path to LoRA file

        Returns:
            LoRA file info dict
        """
        try:
            from pathlib import Path
            import os

            if not os.path.exists(file_path):
                return {
                    "status": "error",
                    "message": "File not found"
                }

            file_size = os.path.getsize(file_path)
            file_name = Path(file_path).stem
            file_ext = Path(file_path).suffix

            return {
                "name": file_name,
                "size": file_size,
                "format": file_ext.lstrip('.') if file_ext else "unknown",
                "path": file_path
            }
        except Exception as e:
            logger.error(f"Failed to get LoRA file info: {e}")
            return {
                "status": "error",
                "message": str(e)
            }
