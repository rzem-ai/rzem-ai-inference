"""Inference API exposed to frontend via pywebview."""

from __future__ import annotations

import base64
import logging
import json
import re
from pathlib import Path
from typing import Any

from rzem_ai_inference_engine import JobParams, LoraParams, TransformerType

from backend.db.database import Database
from backend.services.inference_manager import InferenceServiceManager

from loguru import logger


class InferenceAPI:
    """pywebview js_api mixin for inference operations.

    Every public method is callable from the frontend as
    ``window.pywebview.api.<method>(args)``.
    """

    def __init__(self, inference: InferenceServiceManager, db: Database) -> None:
        self._inference = inference
        self._inference_db = db

    def get_gpu_info(self) -> dict[str, Any]:
        """Return GPU device type, name, and total VRAM."""
        try:
            info = self._inference.active.get_gpu_info()
            return {"status": "success", **info}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def start_engine(self, device: str = "auto", vram_limit_gb: float | None = None) -> dict[str, Any]:
        """Initialize the inference engine."""
        try:
            self._inference.active.start(device=device, vram_limit_gb=vram_limit_gb)
            return {"status": "success"}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def stop_engine(self) -> dict[str, Any]:
        """Shut down the inference engine."""
        try:
            self._inference.active.shutdown()
            return {"status": "success"}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def engine_ready(self) -> dict[str, Any]:
        return {"status": "success", "ready": self._inference.active.ready}

    def submit_job(
        self,
        prompt: str,
        transformer_model: str,
        transformer_type: str,
        vae_model: str,
        steps: int = 20,
        cfg_scale: float = 1.0,
        width: int = 1024,
        height: int = 1024,
        seed: int = -1,
        sampler: str = "euler",
        scheduler: str = "normal",
        loras: list[dict] | None = None,
        bundle_id: str | None = None,
        fal_endpoint: str | None = None,
        **kwargs,
    ) -> dict[str, Any]:
        """Submit a generation job. Returns ``{status, job_id}``."""
        try:
            lora_params = [LoraParams(**l) for l in (loras or [])]

            # Inject FAL API key from DB for cloud jobs
            extra_kwargs: dict[str, Any] = {}
            if transformer_type == "fal_cloud":
                fal_api_key = self._inference_db.get_setting("FAL_KEY")
                if not fal_api_key:
                    return {"status": "error", "message": "FAL API key not configured"}
                extra_kwargs["fal_endpoint"] = fal_endpoint
                extra_kwargs["fal_api_key"] = fal_api_key

            params = JobParams(
                prompt=prompt,
                transformer_model=transformer_model,
                transformer_type=TransformerType(transformer_type),
                vae_model=vae_model,
                steps=steps,
                cfg_scale=cfg_scale,
                width=width,
                height=height,
                seed=seed,
                sampler=sampler,
                scheduler=scheduler,
                loras=lora_params,
                **extra_kwargs,
                **kwargs,
            )

            logger.info(f"Submit image generation job: prompt: '{prompt}'")
            logger.info(f"Submit image generation job:   seed: '{seed}'")
            logger.info(f"Submit image generation job:  steps: '{steps}'")
            logger.info(f"Submit image generation job:  width: '{width}'")
            logger.info(f"Submit image generation job: height: '{height}'")

            job_id = self._inference.active.submit(params, bundle_id=bundle_id)
            return {"status": "success", "job_id": job_id}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def cancel_job(self, job_id: str) -> dict[str, Any]:
        try:
            self._inference.active.cancel(job_id)
            return {"status": "success"}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def poll_events(self) -> dict[str, Any]:
        """Drain all buffered engine events. Called by frontend on an interval."""
        return {"status": "success", "events": self._inference.active.drain_events()}

    def get_image_base64(self, image_path: str) -> dict[str, Any]:
        """Read an image file and return its base64 data URL."""
        try:
            path = Path(image_path)
            if not path.is_file():
                return {"status": "error", "message": f"File not found: {image_path}"}
            data = path.read_bytes()
            b64 = base64.b64encode(data).decode("ascii")
            return {"status": "success", "data_url": f"data:image/png;base64,{b64}"}
        except Exception as e:
            logger.error("Failed to read image %s: %s", image_path, e)
            return {"status": "error", "message": str(e)}

    def get_debug_images(self) -> dict[str, Any]:
        """Find the most recent generation's preview + output images for debug UI."""
        try:
            output_dir = self._inference.local._output_dir
            # Find the most recent output image by mtime
            outputs = sorted(
                output_dir.glob("*_output.png"),
                key=lambda p: p.stat().st_mtime,
                reverse=True,
            )
            if not outputs:
                return {"status": "success", "output": None, "previews": {}}

            latest = outputs[0]
            # Extract job prefix: everything before "_output.png"
            prefix = latest.name.rsplit("_output.png", 1)[0]

            # Find all preview images for this job
            previews: dict[str, str] = {}
            for p in output_dir.glob(f"{prefix}_preview_*.png"):
                match = re.search(r"_preview_(\d+)\.png$", p.name)
                if match:
                    previews[match.group(1)] = str(p)

            return {
                "status": "success",
                "output": str(latest),
                "previews": previews,
            }
        except Exception as e:
            logger.error("Failed to get debug images: %s", e)
            return {"status": "error", "message": str(e)}

