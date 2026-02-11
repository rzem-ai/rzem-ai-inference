"""Inference API exposed to frontend via pywebview."""

from __future__ import annotations

import base64
import logging
import re
from pathlib import Path
from typing import Any

from rzem_ai_inference_engine import JobParams, LoraParams, TransformerType

from backend.services.inference_service import InferenceService

logger = logging.getLogger(__name__)


class InferenceAPI:
    """pywebview js_api mixin for inference operations.

    Every public method is callable from the frontend as
    ``window.pywebview.api.<method>(args)``.
    """

    def __init__(self, inference: InferenceService) -> None:
        self._inference = inference

    def get_gpu_info(self) -> dict[str, Any]:
        """Return GPU device type, name, and total VRAM."""
        try:
            info = self._inference.get_gpu_info()
            return {"status": "success", **info}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def start_engine(self, device: str = "auto", vram_limit_gb: float | None = None) -> dict[str, Any]:
        """Initialize the inference engine."""
        try:
            self._inference.start(device=device, vram_limit_gb=vram_limit_gb)
            return {"status": "success"}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def stop_engine(self) -> dict[str, Any]:
        """Shut down the inference engine."""
        try:
            self._inference.shutdown()
            return {"status": "success"}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def engine_ready(self) -> dict[str, Any]:
        return {"status": "success", "ready": self._inference.ready}

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
        **kwargs,
    ) -> dict[str, Any]:
        """Submit a generation job. Returns ``{status, job_id}``."""
        try:
            lora_params = [LoraParams(**l) for l in (loras or [])]

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
                **kwargs,
            )
            job_id = self._inference.submit(params, bundle_id=bundle_id)
            return {"status": "success", "job_id": job_id}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def cancel_job(self, job_id: str) -> dict[str, Any]:
        try:
            self._inference.cancel(job_id)
            return {"status": "success"}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def poll_events(self) -> dict[str, Any]:
        """Drain all buffered engine events. Called by frontend on an interval."""
        return {"status": "success", "events": self._inference.drain_events()}

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
            output_dir = self._inference._output_dir
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

