"""Device selection and management"""

import torch
from loguru import logger
from typing import Optional


def select_device(device_str: Optional[str] = None) -> torch.device:
    """
    Select the best available device for inference.

    Priority: CUDA > MPS (Apple Silicon) > CPU

    Args:
        device_str: Optional device string (cuda, mps, cpu)

    Returns:
        torch.device
    """
    if device_str:
        device = torch.device(device_str)
        logger.info(f"Using specified device: {device}")
        return device

    # Try CUDA first
    if torch.cuda.is_available():
        device = torch.device("cuda")
        logger.info(f"Using CUDA GPU: {torch.cuda.get_device_name(0)}")
        logger.info(f"CUDA memory: {torch.cuda.get_device_properties(0).total_memory / 1024**3:.2f} GB")
        return device

    # Try MPS (Apple Silicon)
    if hasattr(torch.backends, "mps") and torch.backends.mps.is_available():
        device = torch.device("mps")
        logger.info("Using Apple Silicon MPS")
        return device

    # Fallback to CPU
    device = torch.device("cpu")
    logger.warning("No GPU available, falling back to CPU (this will be slow)")
    return device


def get_optimal_dtype(device: torch.device) -> torch.dtype:
    """
    Get optimal dtype for the device.

    Args:
        device: torch.device

    Returns:
        torch.dtype
    """
    if device.type == "cuda":
        # Use float16 for CUDA if available
        if torch.cuda.is_available() and torch.cuda.get_device_capability()[0] >= 7:
            return torch.float16
        return torch.float32
    elif device.type == "mps":
        # MPS works best with float16
        return torch.float16
    else:
        # CPU uses float32
        return torch.float32


def log_memory_stats(device: torch.device) -> None:
    """Log current memory usage"""
    if device.type == "cuda":
        allocated = torch.cuda.memory_allocated(device) / 1024**3
        reserved = torch.cuda.memory_reserved(device) / 1024**3
        logger.info(f"GPU Memory - Allocated: {allocated:.2f} GB, Reserved: {reserved:.2f} GB")
    elif device.type == "mps":
        logger.info("MPS memory stats not available")
