"""Queue processor for executing generation jobs"""

import asyncio
from typing import Optional
from pathlib import Path
from datetime import datetime
import uuid
from loguru import logger
from PIL import Image

from .types import GenerationJob, JobStatus, LoraConfig
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

    # Pipeline progress ranges for each stage (must sum to ~1.0)
    _STAGE_RANGES = {
        "loading_models": (0.0, 0.10),
        "encoding_t5":    (0.10, 0.20),
        "denoising":      (0.20, 0.90),
        "decoding_vae":   (0.90, 0.95),
        "encoding_png":   (0.95, 1.0),
    }

    async def _report_progress(self, job_id: str, step: int, total_steps: int) -> None:
        """Report denoising step progress (called from thread pool via run_coroutine_threadsafe)."""
        lo, hi = self._STAGE_RANGES["denoising"]
        pipeline_progress = lo + (step / total_steps) * (hi - lo)
        await self.queue_manager.update_job_status(
            job_id,
            JobStatus.RUNNING,
            progress=pipeline_progress,
            current_step=step,
            total_steps=total_steps,
            stage="denoising",
        )

    async def _emit_stage(self, job_id: str, stage: str) -> None:
        """Emit a pipeline stage event at the start of that stage's progress range."""
        lo, _hi = self._STAGE_RANGES[stage]
        await self.queue_manager.update_job_status(
            job_id, JobStatus.RUNNING, progress=lo, stage=stage,
        )

    # Maps architecture → base Diffusers pipeline repo
    _BASE_PIPELINE_MAP = {
        "flux-dev": "black-forest-labs/FLUX.1-dev",
        "flux-schnell": "black-forest-labs/FLUX.1-schnell",
    }

    async def _resolve_pipeline_config(self, component_id: str) -> dict:
        """
        Resolve a model component ID into pipeline loading config.

        Returns a dict with:
          - base_pipeline: HuggingFace repo for the full Diffusers pipeline
          - transformer_path: (optional) local path for GGUF/single-file transformer
          - format: component format (e.g., "gguf", "safetensors")
        """
        if not self.db or not component_id:
            return {}

        component = await self.db.get_model_component_by_id(component_id)
        if not component:
            logger.warning(f"Model component {component_id} not found in database")
            return {}

        fmt = (component.get("format") or "").lower()
        arch = component.get("architecture") or ""

        if fmt == "gguf":
            # GGUF: load base pipeline from architecture, swap transformer from file
            base = self._BASE_PIPELINE_MAP.get(arch, "black-forest-labs/FLUX.1-schnell")
            file_path = component.get("filePath")
            logger.info(
                f"Resolved GGUF component {component_id}: "
                f"base_pipeline={base}, transformer={file_path}"
            )
            return {
                "base_pipeline": base,
                "transformer_path": file_path,
                "format": fmt,
            }
        else:
            # Standard format: repo_id is a full Diffusers pipeline repo
            path = component.get("repoId") or component.get("filePath")
            logger.info(f"Resolved component {component_id} → {path}")
            return {"base_pipeline": path, "format": fmt}

    async def _resolve_loras(self, loras: list[LoraConfig]) -> list[LoraConfig]:
        """Resolve LoRA IDs to file paths via database lookup."""
        if not self.db or not loras:
            return loras

        resolved = []
        for lora in loras:
            if lora.path:
                # Already has a path, use as-is
                resolved.append(lora)
                continue

            if not lora.id:
                logger.warning("LoRA config has neither id nor path, skipping")
                continue

            db_lora = await self.db.get_lora_by_id(lora.id)
            if not db_lora:
                logger.warning(f"LoRA {lora.id} not found in database, skipping")
                continue

            resolved.append(LoraConfig(
                id=lora.id,
                path=db_lora["path"],
                strength=lora.strength,
                name=db_lora.get("name") or lora.name,
            ))
            logger.info(f"Resolved LoRA {lora.id} → {db_lora['path']}")

        return resolved

    async def _process_job(self, job: GenerationJob) -> None:
        """Process a single generation job"""
        self.current_job_id = job.id
        logger.info(f"Processing job {job.id}")

        try:
            loop = asyncio.get_event_loop()

            # Resolve model component ID → pipeline loading config
            pipe_config = await self._resolve_pipeline_config(job.params.model_component_id)

            # Resolve LoRA IDs → file paths
            if job.params.loras:
                job.params.loras = await self._resolve_loras(job.params.loras)

            # Stage 1: Loading models (lazy load — fast if already cached)
            await self._emit_stage(job.id, "loading_models")
            await loop.run_in_executor(
                None,
                lambda: self.pipeline.load_pipeline(
                    model_path=pipe_config.get("base_pipeline"),
                    transformer_path=pipe_config.get("transformer_path"),
                ),
            )

            # Stage 2: Encoding (T5+CLIP happen inside pipeline(), but emit
            #          the stage now so the UI shows it before denoising starts)
            await self._emit_stage(job.id, "encoding_t5")

            # Create progress callback that bridges thread pool → event loop
            def sync_progress_callback(step: int, total_steps: int, _progress: float):
                future = asyncio.run_coroutine_threadsafe(
                    self._report_progress(job.id, step, total_steps),
                    loop,
                )
                future.add_done_callback(
                    lambda f: f.exception() and logger.error(f"Progress update failed: {f.exception()}")
                )

            # Stage 3: Denoising (+ encoding + VAE decode, all inside pipeline())
            image = await loop.run_in_executor(
                None,
                lambda: self.pipeline.generate(
                    job.params,
                    progress_callback=sync_progress_callback,
                )
            )

            # Stage 4: VAE decode already done, but show it briefly
            await self._emit_stage(job.id, "decoding_vae")

            # Stage 5: Save to disk
            await self._emit_stage(job.id, "encoding_png")
            output_path = await self._save_image(job, image)

            # Save to database if available
            image_id = None
            if self.db:
                image_id = await self._save_to_database(job, output_path)

            # Complete
            await self.queue_manager.update_job_status(
                job.id,
                JobStatus.COMPLETED,
                progress=1.0,
                result_path=str(output_path),
            )

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
            "model_name": params.model_type or "flux-schnell",
            "model_component_id": params.model_component_id,
            "t5_component_id": params.t5_component_id,
            "clip_component_id": params.clip_component_id,
            "vae_component_id": params.vae_component_id,
            "bundle_id": params.bundle_id,
            "created_at": int(datetime.now().timestamp()),
        }

        await self.db.insert_image(image_data)
        logger.debug(f"Image metadata saved to database: {image_id}")
        return image_id
