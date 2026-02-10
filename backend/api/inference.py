"""Inference API exposed to frontend via pywebview."""

from __future__ import annotations

import base64
import logging
from pathlib import Path
from typing import Any

from rzem_ai_inference_engine import JobParams, LoraParams, TransformerType

from backend.services.inference_service import InferenceService

logger = logging.getLogger(__name__)

MODEL_PRESETS = [
    {
        "id": "flux1_dev",
        "label": "FLUX.1 Dev",
        "description": "Black Forest Labs FLUX.1-dev with CLIP + T5 text encoders",
        "transformer_type": "flux1_dev",
        "transformer_model": "black-forest-labs/FLUX.1-dev",
        "vae_model": "black-forest-labs/FLUX.1-dev",
        "text_encoders": {
            "clip_tokenizer": "openai/clip-vit-large-patch14",
            "clip_encoder": "openai/clip-vit-large-patch14",
            "t5_tokenizer": "google/t5-v1_1-xxl",
            "t5_encoder": "google/t5-v1_1-xxl",
        },
    },
    {
        "id": "flux2_dev",
        "label": "FLUX.2 Dev",
        "description": "Black Forest Labs FLUX.2-dev with Qwen3 text encoder",
        "transformer_type": "flux2_dev",
        "transformer_model": "black-forest-labs/FLUX.2-dev",
        "vae_model": "black-forest-labs/FLUX.2-dev",
        "text_encoders": {
            "qwen3_tokenizer": "Qwen/Qwen3-0.6B",
            "qwen3_encoder": "Qwen/Qwen3-0.6B",
        },
    },
    {
        "id": "z_image",
        "label": "Z-Image",
        "description": "Z-Image transformer with Qwen3 text encoder",
        "transformer_type": "z_image",
        "transformer_model": "rzem-ai/z-image",
        "vae_model": "rzem-ai/z-image",
        "text_encoders": {
            "qwen3_tokenizer": "Qwen/Qwen3-0.6B",
            "qwen3_encoder": "Qwen/Qwen3-0.6B",
        },
    },
    {
        "id": "qwen_image",
        "label": "Qwen-Image",
        "description": "Qwen-Image transformer with Qwen3 text encoder",
        "transformer_type": "qwen_image",
        "transformer_model": "rzem-ai/qwen-image",
        "vae_model": "rzem-ai/qwen-image",
        "text_encoders": {
            "qwen3_tokenizer": "Qwen/Qwen3-0.6B",
            "qwen3_encoder": "Qwen/Qwen3-0.6B",
        },
    },
]


class InferenceAPI:
    """pywebview js_api mixin for inference operations.

    Every public method is callable from the frontend as
    ``window.pywebview.api.<method>(args)``.
    """

    def __init__(self, inference: InferenceService) -> None:
        self._inference = inference

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
            job_id = self._inference.submit(params)
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

    def get_model_presets(self) -> dict[str, Any]:
        """Return available model presets."""
        return {"status": "success", "presets": MODEL_PRESETS}
