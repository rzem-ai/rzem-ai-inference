"""System stats and auto-update API methods."""

from typing import Dict, Any
from loguru import logger

from updater import get_updater, CURRENT_VERSION
import events
from api.base import ApiBase


class SystemApiMixin(ApiBase):

    # ==================== File Server ====================

    def get_file_server_port(self) -> Dict[str, Any]:
        """Return the port of the local file server for image URLs."""
        return {"port": self._app_state.file_server_port}

    # ==================== System Stats ====================

    def get_system_stats(self) -> Dict[str, Any]:
        """Get system statistics (CPU, RAM, GPU usage)."""
        try:
            import psutil

            cpu_percent = psutil.cpu_percent(interval=0.1)

            memory = psutil.virtual_memory()
            memory_used = memory.used
            memory_total = memory.total
            memory_percent = memory.percent

            gpu_memory_used = None
            gpu_memory_total = None
            gpu_usage_percent = None
            gpu_name = None

            try:
                import pynvml
                pynvml.nvmlInit()
                device_count = pynvml.nvmlDeviceGetCount()

                if device_count > 0:
                    handle = pynvml.nvmlDeviceGetHandleByIndex(0)
                    gpu_name = pynvml.nvmlDeviceGetName(handle)

                    mem_info = pynvml.nvmlDeviceGetMemoryInfo(handle)
                    gpu_memory_used = mem_info.used
                    gpu_memory_total = mem_info.total

                    utilization = pynvml.nvmlDeviceGetUtilizationRates(handle)
                    gpu_usage_percent = float(utilization.gpu)

                pynvml.nvmlShutdown()
            except ImportError:
                logger.debug("pynvml not installed, GPU stats unavailable")
            except Exception as e:
                logger.debug(f"GPU stats unavailable: {e}")

            is_generating = False
            if self._app_state.queue_processor:
                is_generating = getattr(self._app_state.queue_processor, 'is_processing', False)

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

    # ==================== Auto-Update ====================

    def get_version(self) -> Dict[str, str]:
        """Get current application version."""
        return {"version": CURRENT_VERSION, "status": "success"}

    def check_for_updates(self) -> Dict[str, Any]:
        """Check for available updates."""
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
        """Download and install available update."""
        try:
            updater = get_updater()

            if not updater.update_available:
                return {"status": "error", "message": "No update available"}

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

            self._run_async(download_and_install())

            return {"status": "success", "message": "Update download started"}

        except Exception as e:
            logger.error(f"Failed to download update: {e}")
            return {"status": "error", "message": str(e)}
