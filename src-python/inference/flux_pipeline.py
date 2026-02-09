"""FLUX.1 image generation pipeline using Diffusers"""

import torch
from typing import Optional, List, Callable
from pathlib import Path
from loguru import logger
from PIL import Image
from diffusers.pipelines.flux.pipeline_flux import FluxPipeline as DiffusersFluxPipeline
from diffusers import (
    FlowMatchEulerDiscreteScheduler,
    EulerAncestralDiscreteScheduler,
    DPMSolverMultistepScheduler,
)
import random

from .device import select_device, get_optimal_dtype, log_memory_stats
from job_queue.types import GenerationParams, LoraConfig, SamplerType, SchedulerType

# Maps sampler name → Diffusers scheduler class
_SAMPLER_CLASS_MAP = {
    SamplerType.EULER: FlowMatchEulerDiscreteScheduler,
    SamplerType.EULER_A: EulerAncestralDiscreteScheduler,
    SamplerType.DPM_PP_2M: DPMSolverMultistepScheduler,
}


class FluxPipeline:
    """
    FLUX.1 Schnell/Dev image generation pipeline.

    This wraps the Diffusers FluxPipeline with lazy loading,
    memory management, and progress callbacks.
    """

    def __init__(
        self,
        device: Optional[torch.device] = None,
        model_path: Optional[str] = None,
    ):
        self.device = device or select_device()
        self.dtype = get_optimal_dtype(self.device)
        self.model_path = model_path or "black-forest-labs/FLUX.1-schnell"
        # Cache key tracks (base_pipeline, transformer_path) for reload avoidance
        self._loaded_key: Optional[str] = None

        self.pipeline: Optional[DiffusersFluxPipeline] = None
        self.is_loaded = False
        self._original_scheduler_config: Optional[dict] = None

        logger.info(f"FluxPipeline initialized with device: {self.device}, dtype: {self.dtype}")

    def _check_gpu_memory(self) -> None:
        """Log available GPU memory before loading."""
        if self.device.type != "cuda":
            return

        free_mem = torch.cuda.get_device_properties(0).total_memory
        allocated = torch.cuda.memory_allocated(0)
        available = free_mem - allocated
        logger.info(
            f"GPU memory check: {available / 1024**3:.1f} GB available "
            f"({allocated / 1024**3:.1f} GB allocated / {free_mem / 1024**3:.1f} GB total)"
        )

    def load_pipeline(
        self,
        model_path: Optional[str] = None,
        transformer_path: Optional[str] = None,
        force_reload: bool = False,
    ) -> None:
        """
        Lazy load the FLUX pipeline, or reload if the model changed.

        Args:
            model_path: Base pipeline repo (e.g., "black-forest-labs/FLUX.1-dev")
            transformer_path: Optional path to a GGUF transformer file to swap in
            force_reload: Force reload even if already loaded
        """
        base = model_path or self.model_path
        cache_key = f"{base}|{transformer_path or ''}"

        # Skip reload if already loaded with the same config
        if self.is_loaded and not force_reload and cache_key == self._loaded_key:
            logger.debug("Pipeline already loaded with correct model")
            return

        # Unload if switching models or force reloading
        if self.pipeline is not None:
            logger.info(f"Unloading pipeline (switching to {base}, transformer={transformer_path})")
            self.unload_pipeline()

        logger.info(f"Loading FLUX pipeline from {base}")
        log_memory_stats(self.device)
        self._check_gpu_memory()

        try:
            extra_kwargs = {}

            # For GGUF transformers, load the transformer separately and inject it
            if transformer_path and transformer_path.lower().endswith(".gguf"):
                from diffusers import FluxTransformer2DModel, GGUFQuantizationConfig
                logger.info(f"Loading GGUF transformer from {transformer_path}")
                transformer = FluxTransformer2DModel.from_single_file(
                    transformer_path,
                    quantization_config=GGUFQuantizationConfig(compute_dtype=self.dtype),
                    torch_dtype=self.dtype,
                )
                extra_kwargs["transformer"] = transformer
                logger.info("GGUF transformer loaded")

            logger.info("Calling from_pretrained (this may take a few minutes)...")
            self.pipeline = DiffusersFluxPipeline.from_pretrained(
                base,
                torch_dtype=self.dtype,
                device_map="balanced",
                **extra_kwargs,
            )
            logger.info("Pipeline loaded onto device")

            # Save the original scheduler config for reset
            self._original_scheduler_config = dict(self.pipeline.scheduler.config)

            # Enable memory optimizations
            if self.device.type == "cuda":
                self.pipeline.enable_attention_slicing()
                if hasattr(self.pipeline, "enable_vae_slicing"):
                    self.pipeline.enable_vae_slicing()

            self.is_loaded = True
            self._loaded_key = cache_key
            logger.info("FLUX pipeline loaded successfully")
            log_memory_stats(self.device)

        except torch.cuda.OutOfMemoryError as e:
            logger.error(f"GPU out of memory loading pipeline: {e}")
            self._cleanup_on_error()
            raise RuntimeError(
                f"Not enough GPU memory to load FLUX model. "
                f"Try closing other GPU applications or using a quantized model."
            ) from e
        except Exception as e:
            logger.error(f"Failed to load pipeline: {e}")
            self._cleanup_on_error()
            raise

    def _cleanup_on_error(self) -> None:
        """Cleanup after a failed load attempt."""
        self.pipeline = None
        self.is_loaded = False
        if self.device.type == "cuda":
            torch.cuda.empty_cache()

    def unload_pipeline(self) -> None:
        """Unload the pipeline to free memory"""
        if self.pipeline is not None:
            logger.info("Unloading pipeline")
            del self.pipeline
            self.pipeline = None
            self.is_loaded = False
            self._loaded_key = None
            self._original_scheduler_config = None

            if self.device.type == "cuda":
                torch.cuda.empty_cache()
                torch.cuda.synchronize()

            log_memory_stats(self.device)

    def apply_loras(self, loras: List[LoraConfig]) -> None:
        """Apply LoRA adapters to the pipeline."""
        if not self.is_loaded or not self.pipeline:
            raise RuntimeError("Pipeline not loaded")

        if not loras:
            return

        logger.info(f"Applying {len(loras)} LoRA adapters")

        for lora in loras:
            try:
                logger.debug(f"Loading LoRA: {lora.path} with strength {lora.strength}")
                self.pipeline.load_lora_weights(
                    lora.path,
                    adapter_name=lora.name or Path(lora.path).stem,
                )
                if hasattr(self.pipeline, "set_adapters"):
                    self.pipeline.set_adapters(
                        [lora.name or Path(lora.path).stem],
                        adapter_weights=[lora.strength],
                    )
            except Exception as e:
                logger.error(f"Failed to apply LoRA {lora.path}: {e}")
                raise

    def remove_loras(self) -> None:
        """Remove all LoRA adapters"""
        if not self.is_loaded or not self.pipeline:
            return

        try:
            if hasattr(self.pipeline, "unload_lora_weights"):
                self.pipeline.unload_lora_weights()
                logger.debug("LoRA adapters removed")
        except Exception as e:
            logger.warning(f"Failed to remove LoRAs: {e}")

    def _configure_scheduler(
        self,
        sampler: Optional[SamplerType],
        scheduler: Optional[SchedulerType],
    ) -> None:
        """
        Swap the pipeline's scheduler based on sampler + scheduler type.

        The Diffusers "scheduler" object combines what the UI calls "sampler"
        (the algorithm) and "scheduler" (the sigma/noise schedule).
        """
        if not self.pipeline:
            return

        sampler = sampler or SamplerType.EULER
        scheduler = scheduler or SchedulerType.NORMAL

        scheduler_cls = _SAMPLER_CLASS_MAP.get(sampler, FlowMatchEulerDiscreteScheduler)

        # Build config overrides for sigma schedule
        config_overrides: dict = {}
        if scheduler == SchedulerType.KARRAS:
            config_overrides["use_karras_sigmas"] = True
        elif scheduler == SchedulerType.EXPONENTIAL:
            config_overrides["use_exponential_sigmas"] = True
        # NORMAL and SIMPLE use defaults (no overrides)

        # Only swap if something changed
        current = self.pipeline.scheduler
        needs_swap = (
            type(current) is not scheduler_cls
            or any(
                getattr(current.config, k, None) != v
                for k, v in config_overrides.items()
            )
        )

        if needs_swap:
            base_config = dict(self._original_scheduler_config or current.config)
            # Reset sigma flags before applying new ones
            base_config.pop("use_karras_sigmas", None)
            base_config.pop("use_exponential_sigmas", None)
            base_config.update(config_overrides)

            try:
                self.pipeline.scheduler = scheduler_cls.from_config(base_config)
                logger.info(f"Scheduler configured: {scheduler_cls.__name__} (schedule={scheduler.value})")
            except Exception as e:
                logger.warning(f"Failed to configure scheduler {scheduler_cls.__name__}: {e}, keeping current")

    def generate(
        self,
        params: GenerationParams,
        progress_callback: Optional[Callable[[int, int, float], None]] = None,
    ) -> Image.Image:
        """
        Generate an image from parameters.

        Returns:
            Generated image as PIL Image
        """
        self.load_pipeline()

        if not self.pipeline:
            raise RuntimeError("Pipeline failed to load")

        # Configure scheduler/sampler before generation
        self._configure_scheduler(params.sampler, params.scheduler)

        if params.loras:
            self.apply_loras(params.loras)

        try:
            seed = params.seed
            if seed < 0:
                seed = random.randint(0, 2**32 - 1)

            generator = torch.Generator(device="cpu").manual_seed(seed)
            logger.info(f"Generating with seed: {seed}")

            def callback_wrapper(pipe, step: int, timestep: int, callback_kwargs):
                if progress_callback:
                    progress = (step + 1) / params.steps
                    progress_callback(step + 1, params.steps, progress)
                return callback_kwargs

            logger.info(
                f"Starting generation: {params.width}x{params.height}, "
                f"{params.steps} steps, sampler={params.sampler}, scheduler={params.scheduler}"
            )

            result = self.pipeline(
                prompt=params.prompt,
                width=params.width,
                height=params.height,
                num_inference_steps=params.steps,
                guidance_scale=params.cfg_scale,
                generator=generator,
                callback_on_step_end=callback_wrapper,
            )

            image = result.images[0]
            logger.info("Generation completed successfully")

            return image

        finally:
            if params.loras:
                self.remove_loras()

    def __del__(self):
        """Cleanup on deletion"""
        self.unload_pipeline()
