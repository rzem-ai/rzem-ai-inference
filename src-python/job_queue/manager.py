"""Queue manager for handling generation jobs"""

import asyncio
from typing import Dict, List, Optional, Callable
from datetime import datetime
from loguru import logger

from .types import GenerationJob, GenerationParams, JobStatus


class QueueManager:
    """Manages the queue of generation jobs"""

    def __init__(self):
        self.jobs: Dict[str, GenerationJob] = {}
        self.queue: asyncio.Queue[GenerationJob] = asyncio.Queue()
        self.lock = asyncio.Lock()
        self.event_callbacks: List[Callable] = []

    async def add_job(self, params: GenerationParams) -> str:
        """Add a new job to the queue"""
        job = GenerationJob(params=params)

        async with self.lock:
            self.jobs[job.id] = job
            await self.queue.put(job)

        logger.info(f"Job {job.id} added to queue")
        await self._emit_event("job-update", {
            "job_id": job.id,
            "status": "pending",
            "progress": 0.0,
        })

        return job.id

    async def get_job(self, job_id: str) -> Optional[GenerationJob]:
        """Get job by ID"""
        return self.jobs.get(job_id)

    async def get_all_jobs(self) -> List[GenerationJob]:
        """Get all jobs"""
        return list(self.jobs.values())

    async def get_pending_jobs(self) -> List[GenerationJob]:
        """Get all pending jobs"""
        return [job for job in self.jobs.values() if job.status == JobStatus.PENDING]

    async def update_job_status(
        self,
        job_id: str,
        status: JobStatus,
        progress: Optional[float] = None,
        error: Optional[str] = None,
        result_path: Optional[str] = None,
        current_step: Optional[int] = None,
        total_steps: Optional[int] = None,
        stage: Optional[str] = None,
    ) -> None:
        """Update job status and emit a single job-update event."""
        async with self.lock:
            job = self.jobs.get(job_id)
            if not job:
                return

            job.status = status
            if progress is not None:
                job.progress = progress
            if error is not None:
                job.error = error
            if result_path is not None:
                job.result_path = result_path

            if status == JobStatus.RUNNING and not job.started_at:
                job.started_at = datetime.utcnow()
            elif status in [JobStatus.COMPLETED, JobStatus.FAILED, JobStatus.CANCELLED]:
                job.completed_at = datetime.utcnow()

        await self._emit_event("job-update", {
            "job_id": job_id,
            "status": status.value,
            "progress": job.progress,
            "error": error,
            "result_path": result_path,
            "current_step": current_step,
            "total_steps": total_steps,
            "stage": stage,
        })

    async def cancel_job(self, job_id: str) -> bool:
        """Cancel a job"""
        async with self.lock:
            job = self.jobs.get(job_id)
            if not job or job.status not in [JobStatus.PENDING, JobStatus.RUNNING]:
                return False

            job.status = JobStatus.CANCELLED
            job.completed_at = datetime.utcnow()

        await self._emit_event("job-update", {
            "job_id": job_id,
            "status": "cancelled",
            "progress": job.progress,
        })
        logger.info(f"Job {job_id} cancelled")
        return True

    async def clear_completed_jobs(self) -> None:
        """Clear all completed/failed/cancelled jobs"""
        async with self.lock:
            to_remove = [
                job_id for job_id, job in self.jobs.items()
                if job.status in [JobStatus.COMPLETED, JobStatus.FAILED, JobStatus.CANCELLED]
            ]
            for job_id in to_remove:
                del self.jobs[job_id]

        logger.info(f"Cleared {len(to_remove)} completed jobs")

    def register_event_callback(self, callback: Callable) -> None:
        """Register a callback for events"""
        self.event_callbacks.append(callback)

    async def _emit_event(self, event_name: str, payload: dict) -> None:
        """Emit event to all registered callbacks"""
        for callback in self.event_callbacks:
            try:
                if asyncio.iscoroutinefunction(callback):
                    await callback(event_name, payload)
                else:
                    callback(event_name, payload)
            except Exception as e:
                logger.error(f"Error in event callback: {e}")
