"""
ComfyUI to native FLUX format converter.

Port of src-tauri/src/models/converter.rs.
Strips the `model.diffusion_model.` prefix from tensor names in safetensors files,
making them compatible with Candle-based inference.
"""

import json
import struct
from pathlib import Path
from typing import Optional

from loguru import logger

PREFIX = "model.diffusion_model."


def is_comfyui_format(path: str) -> bool:
    """
    Check if a safetensors file is in ComfyUI format.

    ComfyUI format tensors have the `model.diffusion_model.` prefix.
    """
    file_path = Path(path)
    if not file_path.exists():
        return False

    try:
        header = _read_safetensors_header(file_path)
        return any(k.startswith(PREFIX) for k in header if k != "__metadata__")
    except Exception as e:
        logger.debug(f"Failed to check format: {e}")
        return False


def convert_comfyui_to_native(
    input_path: str, output_path: Optional[str] = None
) -> str:
    """
    Convert a ComfyUI/InvokeAI format model to native FLUX format.

    Strips `model.diffusion_model.` prefix from all tensor names.
    Returns the output file path.
    """
    source = Path(input_path)
    if not source.exists():
        raise FileNotFoundError(f"Input file does not exist: {input_path}")

    # Determine output path
    if output_path:
        dest = Path(output_path)
    else:
        dest = source.parent / f"{source.stem}-native.safetensors"

    logger.info(f"Starting ComfyUI -> Native FLUX conversion")
    logger.info(f"  Input:  {source}")
    logger.info(f"  Output: {dest}")

    # Read the entire file
    buffer = source.read_bytes()
    file_size_mb = len(buffer) // 1_000_000
    logger.info(f"  File size: {file_size_mb} MB")

    # Parse header
    if len(buffer) < 8:
        raise ValueError("File too small to be a safetensors file")

    header_len = struct.unpack("<Q", buffer[:8])[0]
    if header_len > 10 * 1024 * 1024:
        raise ValueError(f"Header too large: {header_len} bytes")

    header_bytes = buffer[8 : 8 + header_len]
    header = json.loads(header_bytes)

    # Count ComfyUI tensors
    tensor_names = [k for k in header if k != "__metadata__"]
    total_tensors = len(tensor_names)
    comfyui_count = sum(1 for n in tensor_names if n.startswith(PREFIX))

    logger.info(f"  Found {total_tensors} tensors, {comfyui_count} with ComfyUI prefix")

    if comfyui_count == 0:
        raise ValueError("Input file does not appear to be in ComfyUI format")

    # Build new header with renamed keys
    new_header = {}

    # Preserve metadata
    if "__metadata__" in header:
        new_header["__metadata__"] = header["__metadata__"]

    # Rename tensor entries (strip prefix)
    for name, info in header.items():
        if name == "__metadata__":
            continue
        new_name = name[len(PREFIX) :] if name.startswith(PREFIX) else name
        new_header[new_name] = info

    # Serialize new header
    new_header_json = json.dumps(new_header, separators=(",", ":")).encode("utf-8")
    new_header_len = struct.pack("<Q", len(new_header_json))

    # Write output: new header length + new header + original tensor data
    data_start = 8 + header_len
    dest.write_bytes(new_header_len + new_header_json + buffer[data_start:])

    output_size_mb = dest.stat().st_size // 1_000_000
    logger.info(f"  Output written: {output_size_mb} MB")
    logger.info("Conversion complete!")

    return str(dest)


def _read_safetensors_header(path: Path) -> dict:
    """Read and parse the JSON header from a safetensors file."""
    with open(path, "rb") as f:
        header_len_bytes = f.read(8)
        if len(header_len_bytes) < 8:
            raise ValueError("File too small")
        header_len = struct.unpack("<Q", header_len_bytes)[0]
        if header_len > 10 * 1024 * 1024:
            raise ValueError(f"Header too large: {header_len}")
        header_bytes = f.read(header_len)
        return json.loads(header_bytes)
