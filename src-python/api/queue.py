"""Queue and generation job API methods."""

from datetime import datetime
from typing import Optional, List, Dict, Any
from loguru import logger

from job_queue.types import GenerationParams
from api.base import ApiBase


def _serialize_job(job) -> Dict[str, Any]:
    """Serialize a GenerationJob to a JSON-safe dict.

    model_dump() returns raw datetime objects that pywebview can't
    JSON-serialize.  Convert them to integer timestamps (what the
    frontend expects).
    """
    d = job.model_dump()
    for field in ("created_at", "started_at", "completed_at"):
        val = d.get(field)
        if isinstance(val, datetime):
            d[field] = int(val.timestamp())
    return d


class QueueApiMixin(ApiBase):

    def queue_generation(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """Queue a new generation job."""
        try:
            gen_params = GenerationParams(**params)

            if not gen_params.prompt:
                return {"status": "error", "message": "Prompt cannot be empty"}

            job_id = self._run_async(self._app_state.queue_manager.add_job(gen_params))
            logger.info(f"Job queued: {job_id}")
            return {"status": "success", "job_id": job_id}
        except Exception as e:
            logger.error(f"Failed to queue generation: {e}")
            return {"status": "error", "message": str(e)}

    def get_all_jobs(self) -> List[Dict[str, Any]]:
        """Get all generation jobs."""
        try:
            jobs = self._run_async(self._app_state.queue_manager.get_all_jobs())
            return [_serialize_job(job) for job in jobs]
        except Exception as e:
            logger.error(f"Failed to get jobs: {e}")
            return []

    def get_job(self, job_id: str) -> Optional[Dict[str, Any]]:
        """Get a specific job by ID."""
        try:
            job = self._run_async(self._app_state.queue_manager.get_job(job_id))
            return _serialize_job(job) if job else None
        except Exception as e:
            logger.error(f"Failed to get job {job_id}: {e}")
            return None

    def cancel_job(self, job_id: str) -> Dict[str, Any]:
        """Cancel a job."""
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
        """Clear all completed/failed/cancelled jobs."""
        try:
            self._run_async(self._app_state.queue_manager.clear_completed_jobs())
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to clear jobs: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Client Mode Aliases ====================

    def client_add_to_queue(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """Add generation job to queue (client mode alias)."""
        return self.queue_generation(params)

    def client_get_queue_jobs(self) -> List[Dict[str, Any]]:
        """Get all queue jobs (client mode alias)."""
        return self.get_all_jobs()

    def client_get_queue_job(self, job_id: str) -> Optional[Dict[str, Any]]:
        """Get specific queue job (client mode alias)."""
        return self.get_job(job_id)

    def client_cancel_queue_job(self, job_id: str) -> Dict[str, Any]:
        """Cancel a queue job (client mode alias)."""
        return self.cancel_job(job_id)
