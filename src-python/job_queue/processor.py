"""Queue processor for executing generation jobs"""

import asyncio
from typing import Optional, Callable
from pathlib import Path
from datetime import datetime
import uuid
from loguru import logger
from PIL import Image

from .types import GenerationJob, JobStatus, ProgressUpdate
from .manager import QueueManager
from inference.flux_pipeline import FluxPipeline
from db.database import InferenceDb


class QueueProcessor:
    """Processes generation jobs from the queue"""

    def __init__(
        self,
        queue_manager: QueueManager,
        db: Optional[InferenceDb] = None,
        output_dir: str = "./outputs",
    ):
        self.queue_manager = queue_manager
        self.db = db
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

        self.pipeline: Optional[FluxPipeline] = None
        self.running = False
        self.current_job_id: Optional[str] = None
        self.event_callbacks: list[Callable] = []

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

    async def start(self) -> None:
        """Start processing jobs from the queue"""
        if self.running:
            logger.warning("Processor already running")
            return

        self.running = True
        logger.info("Queue processor started")

        # Initialize pipeline
        if not self.pipeline:
            self.pipeline = FluxPipeline()

        # Start processing loop
        asyncio.create_task(self._process_loop())

    async def stop(self) -> None:
        """Stop processing jobs"""
        self.running = False
        logger.info("Queue processor stopped")

        # Cleanup pipeline
        if self.pipeline:
            self.pipeline.unload_pipeline()

    async def _process_loop(self) -> None:
        """Main processing loop"""
        while self.running:
            try:
                # Get next job from queue (with timeout)
                try:
                    job = await asyncio.wait_for(
                        self.queue_manager.queue.get(),
                        timeout=1.0
                    )
                except asyncio.TimeoutError:
                    continue

                # Check if job was cancelled while waiting
                if job.status == JobStatus.CANCELLED:
                    continue

                # Process the job
                await self._process_job(job)

            except Exception as e:
                logger.error(f"Error in processing loop: {e}")
                await asyncio.sleep(1.0)

    async def _process_job(self, job: GenerationJob) -> None:
        """Process a single generation job"""
        self.current_job_id = job.id
        logger.info(f"Processing job {job.id}")

        try:
            # Update status to running
            await self.queue_manager.update_job_status(
                job.id,
                JobStatus.RUNNING,
                progress=0.0,
            )

            # Create progress callback
            async def progress_callback(step: int, total_steps: int, progress: float):
                await self.queue_manager.update_job_status(
                    job.id,
                    JobStatus.RUNNING,
                    progress=progress,
                )
                await self._emit_event("job-progress", {
                    "job_id": job.id,
                    "progress": progress,
                    "current_step": step,
                    "total_steps": total_steps,
                    "stage": "generating",
                })

            # Generate image (run in thread pool to not block event loop)
            loop = asyncio.get_event_loop()
            image = await loop.run_in_executor(
                None,
                lambda: self.pipeline.generate(
                    job.params,
                    progress_callback=lambda s, t, p: asyncio.run(progress_callback(s, t, p))
                )
            )

            # Save image
            output_path = await self._save_image(job, image)

            # Save to database if available
            image_id = None
            if self.db:
                image_id = await self._save_to_database(job, output_path)

            # Update job as completed
            await self.queue_manager.update_job_status(
                job.id,
                JobStatus.COMPLETED,
                progress=1.0,
                result_path=str(output_path),
            )

            # Update job with image_id
            job.image_id = image_id

            logger.info(f"Job {job.id} completed successfully: {output_path}")

        except Exception as e:
            logger.error(f"Job {job.id} failed: {e}")
            await self.queue_manager.update_job_status(
                job.id,
                JobStatus.FAILED,
                error=str(e),
            )

        finally:
            self.current_job_id = None

    async def _save_image(self, job: GenerationJob, image: Image.Image) -> Path:
        """Save generated image to disk"""
        # Generate filename with timestamp and seed
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        seed = job.params.seed if job.params.seed >= 0 else "random"
        filename = f"{timestamp}_seed{seed}.png"
        output_path = self.output_dir / filename

        # Save image in thread pool to not block
        loop = asyncio.get_event_loop()
        await loop.run_in_executor(None, image.save, str(output_path))

        logger.debug(f"Image saved to {output_path}")
        return output_path

    async def _save_to_database(self, job: GenerationJob, output_path: Path) -> str:
        """Save image metadata to database"""
        if not self.db:
            return None

        image_id = str(uuid.uuid4())
        params = job.params

        image_data = {
            "id": image_id,
            "file_path": str(output_path),
            "prompt": params.prompt,
            "negative_prompt": params.negative_prompt,
            "width": params.width,
            "height": params.height,
            "steps": params.steps,
            "cfg_scale": params.cfg_scale,
            "seed": params.seed,
            "sampler": params.sampler.value if params.sampler else None,
            "scheduler": params.scheduler.value if params.scheduler else None,
            "model_type": params.model_type,
            "model_component_id": params.model_component_id,
            "t5_component_id": params.t5_component_id,
            "clip_component_id": params.clip_component_id,
            "vae_component_id": params.vae_component_id,
            "bundle_id": params.bundle_id,
            "folder_id": None,
        }

        await self.db.insert_image(image_data)
        logger.debug(f"Image metadata saved to database: {image_id}")
        return image_id
