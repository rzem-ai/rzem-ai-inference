"""FLUX.1 image generation pipeline using Diffusers"""

import torch
from typing import Optional, List, Dict, Any, Callable
from pathlib import Path
from loguru import logger
from diffusers import FluxPipeline as DiffusersFluxPipeline
from diffusers import FluxTransformer2DModel
import random

from .device import select_device, get_optimal_dtype, log_memory_stats
from job_queue.types import GenerationParams, LoraConfig


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

        self.pipeline: Optional[DiffusersFluxPipeline] = None
        self.is_loaded = False

        logger.info(f"FluxPipeline initialized with device: {self.device}, dtype: {self.dtype}")

    def load_pipeline(self, force_reload: bool = False) -> None:
        """
        Lazy load the FLUX pipeline.

        Args:
            force_reload: Force reload even if already loaded
        """
        if self.is_loaded and not force_reload:
            logger.debug("Pipeline already loaded")
            return

        if self.pipeline is not None and force_reload:
            logger.info("Unloading existing pipeline")
            self.unload_pipeline()

        logger.info(f"Loading FLUX pipeline from {self.model_path}")
        log_memory_stats(self.device)

        try:
            # Load the pipeline with optimal settings
            self.pipeline = DiffusersFluxPipeline.from_pretrained(
                self.model_path,
                torch_dtype=self.dtype,
            )

            # Move to device
            self.pipeline = self.pipeline.to(self.device)

            # Enable memory optimizations
            if self.device.type == "cuda":
                # Enable attention slicing for lower memory usage
                self.pipeline.enable_attention_slicing()

                # Enable VAE slicing for large images
                if hasattr(self.pipeline, "enable_vae_slicing"):
                    self.pipeline.enable_vae_slicing()

            self.is_loaded = True
            logger.info("FLUX pipeline loaded successfully")
            log_memory_stats(self.device)

        except Exception as e:
            logger.error(f"Failed to load pipeline: {e}")
            raise

    def unload_pipeline(self) -> None:
        """Unload the pipeline to free memory"""
        if self.pipeline is not None:
            logger.info("Unloading pipeline")
            del self.pipeline
            self.pipeline = None
            self.is_loaded = False

            # Clear GPU cache
            if self.device.type == "cuda":
                torch.cuda.empty_cache()
                torch.cuda.synchronize()

            log_memory_stats(self.device)

    def apply_loras(self, loras: List[LoraConfig]) -> None:
        """
        Apply LoRA adapters to the pipeline.

        Args:
            loras: List of LoRA configurations
        """
        if not self.is_loaded or not self.pipeline:
            raise RuntimeError("Pipeline not loaded")

        if not loras:
            return

        logger.info(f"Applying {len(loras)} LoRA adapters")

        for lora in loras:
            try:
                logger.debug(f"Loading LoRA: {lora.path} with strength {lora.strength}")

                # Load LoRA weights
                self.pipeline.load_lora_weights(
                    lora.path,
                    adapter_name=lora.name or Path(lora.path).stem,
                )

                # Set LoRA scale
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

    def generate(
        self,
        params: GenerationParams,
        progress_callback: Optional[Callable[[int, int, float], None]] = None,
    ) -> torch.Tensor:
        """
        Generate an image from parameters.

        Args:
            params: Generation parameters
            progress_callback: Optional callback(step, total_steps, latent)

        Returns:
            Generated image as PIL Image
        """
        # Ensure pipeline is loaded
        self.load_pipeline()

        if not self.pipeline:
            raise RuntimeError("Pipeline failed to load")

        # Apply LoRAs if specified
        if params.loras:
            self.apply_loras(params.loras)

        try:
            # Prepare seed
            seed = params.seed
            if seed < 0:
                seed = random.randint(0, 2**32 - 1)

            generator = torch.Generator(device=self.device).manual_seed(seed)
            logger.info(f"Generating with seed: {seed}")

            # Prepare callback wrapper
            def callback_wrapper(step: int, timestep: int, latents: torch.Tensor):
                if progress_callback:
                    progress = (step + 1) / params.steps
                    progress_callback(step + 1, params.steps, progress)

            # Generate image
            logger.info(f"Starting generation: {params.width}x{params.height}, {params.steps} steps")

            result = self.pipeline(
                prompt=params.prompt,
                negative_prompt=params.negative_prompt,
                width=params.width,
                height=params.height,
                num_inference_steps=params.steps,
                guidance_scale=params.cfg_scale,
                generator=generator,
                callback=callback_wrapper,
                callback_steps=1,
            )

            image = result.images[0]
            logger.info("Generation completed successfully")

            return image

        finally:
            # Always cleanup LoRAs
            if params.loras:
                self.remove_loras()

    def __del__(self):
        """Cleanup on deletion"""
        self.unload_pipeline()
