"""
Model scanner - discovers model components in HuggingFace cache and arbitrary directories.

Port of src-tauri/src/models/scanner.rs.
Identifies component types by inspecting safetensors headers and GGUF metadata.
"""

import json
import os
import struct
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional

from loguru import logger


# Component types that can be discovered
COMPONENT_TYPES = {
    "transformer",
    "t5_encoder",
    "clip_encoder",
    "vae",
    "clip_tokenizer",
    "t5_tokenizer",
    "lora",
}

# Model formats
FORMAT_SAFETENSORS = "safetensors"
FORMAT_GGUF = "gguf"
FORMAT_JSON = "json"

# Unsupported model variant keywords
_UNSUPPORTED_KEYWORDS = [
    "kontext", "fill", "canny", "depth", "redux",
    "bnb_nf4", "bnb_int8", "bitsandbytes",
    "_nf4", "_int8", "-nf4-", "-int8-",
]

# Known quantization patterns
_QUANT_PATTERNS = ["Q3_K_L", "Q4_K_S", "Q5_K_M", "Q8_0", "Q4_0", "Q5_0"]

# Progress callback type: (current_index, total, item_name)
ProgressCallback = Callable[[int, int, str], None]


class DiscoveredComponent:
    """Information about a discovered model component."""

    __slots__ = (
        "component_type", "architecture", "format", "path", "repo_id",
        "repo_snapshot", "file_size", "file_hash", "quantization",
        "is_sharded", "shard_count", "vram_mb", "metadata",
    )

    def __init__(
        self,
        component_type: str,
        architecture: str,
        format: str,
        path: str,
        repo_id: str,
        repo_snapshot: Optional[str] = None,
        file_size: int = 0,
        file_hash: Optional[str] = None,
        quantization: Optional[str] = None,
        is_sharded: bool = False,
        shard_count: Optional[int] = None,
        vram_mb: Optional[int] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ):
        self.component_type = component_type
        self.architecture = architecture
        self.format = format
        self.path = path
        self.repo_id = repo_id
        self.repo_snapshot = repo_snapshot
        self.file_size = file_size
        self.file_hash = file_hash
        self.quantization = quantization
        self.is_sharded = is_sharded
        self.shard_count = shard_count
        self.vram_mb = vram_mb
        self.metadata = metadata or {}

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for database storage."""
        return {
            "componentType": self.component_type,
            "architecture": self.architecture,
            "format": self.format,
            "filePath": self.path,
            "repoId": self.repo_id,
            "repoSnapshot": self.repo_snapshot,
            "fileSize": self.file_size,
            "fileHash": self.file_hash,
            "quantization": self.quantization,
            "isSharded": self.is_sharded,
            "shardCount": self.shard_count,
            "vramMb": self.vram_mb,
            "name": _generate_component_name(
                self.repo_id, self.component_type, self.quantization, self.is_sharded
            ),
            "supportsLoras": self.component_type == "transformer",
            "metadata": self.metadata,
        }


# ============================================================================
# HuggingFace Cache Scanner
# ============================================================================


def get_hf_cache_dir() -> Path:
    """Get the HuggingFace cache directory."""
    if hf_cache := os.environ.get("HF_HUB_CACHE"):
        return Path(hf_cache)
    if hf_home := os.environ.get("HF_HOME"):
        return Path(hf_home) / "hub"
    return Path.home() / ".cache" / "huggingface" / "hub"


def scan_hf_cache(
    progress_callback: Optional[ProgressCallback] = None,
) -> List[DiscoveredComponent]:
    """
    Scan the HuggingFace cache for all model components.

    Discovers transformers, text encoders, VAE decoders, and tokenizers
    in the standard HF cache layout (models--owner--name/snapshots/hash/).
    """
    cache_dir = get_hf_cache_dir()
    components: List[DiscoveredComponent] = []

    logger.info(f"Starting HuggingFace cache scan: {cache_dir}")

    if not cache_dir.exists():
        logger.warning(f"HF cache directory does not exist: {cache_dir}")
        return components

    # Find all model--owner--name directories
    model_dirs = sorted(
        [d for d in cache_dir.iterdir() if d.is_dir() and d.name.startswith("models--")]
    )
    total_repos = len(model_dirs)
    logger.info(f"Found {total_repos} model repositories to scan")

    for idx, repo_dir in enumerate(model_dirs):
        # Parse repo name: models--owner--name -> owner/name
        parts = repo_dir.name.split("--")
        if len(parts) < 3:
            continue
        repo_name = f"{parts[1]}/{parts[2]}"

        logger.info(f"[{idx + 1}/{total_repos}] Scanning: {repo_name}")
        if progress_callback:
            progress_callback(idx + 1, total_repos, repo_name)

        try:
            snapshot_dir = _get_active_snapshot(repo_dir)
            if not snapshot_dir:
                continue

            snapshot_hash = snapshot_dir.name
            repo_components = _scan_repo_snapshot(snapshot_dir, repo_name, snapshot_hash)

            if repo_components:
                logger.info(f"  Found {len(repo_components)} component(s)")
                for comp in repo_components:
                    size_mb = comp.file_size // 1_000_000
                    quant = f" [{comp.quantization}]" if comp.quantization else ""
                    logger.info(f"    {comp.component_type}: {comp.architecture} ({size_mb} MB){quant}")
                components.extend(repo_components)
        except Exception as e:
            logger.warning(f"  Failed to scan {repo_name}: {e}")

    _log_scan_summary(components)
    return components


def _get_active_snapshot(repo_dir: Path) -> Optional[Path]:
    """Get the active snapshot directory for a model repo."""
    # Try refs/main first
    refs_main = repo_dir / "refs" / "main"
    if refs_main.exists():
        hash_str = refs_main.read_text().strip()
        snapshot = repo_dir / "snapshots" / hash_str
        if snapshot.exists():
            return snapshot

    # Fallback: first snapshot directory
    snapshots_dir = repo_dir / "snapshots"
    if snapshots_dir.exists():
        for entry in sorted(snapshots_dir.iterdir()):
            if entry.is_dir():
                return entry

    return None


def _scan_repo_snapshot(
    snapshot: Path, repo_id: str, snapshot_hash: str
) -> List[DiscoveredComponent]:
    """Scan a single repo snapshot directory for all component types."""
    components: List[DiscoveredComponent] = []

    components.extend(_find_transformers(snapshot, repo_id, snapshot_hash))
    components.extend(_find_t5_encoders(snapshot, repo_id, snapshot_hash))
    components.extend(_find_clip_encoders(snapshot, repo_id, snapshot_hash))
    components.extend(_find_vae_decoders(snapshot, repo_id, snapshot_hash))
    components.extend(_find_tokenizers(snapshot, repo_id, snapshot_hash))

    return components


def _find_transformers(
    snapshot: Path, repo_id: str, snapshot_hash: str
) -> List[DiscoveredComponent]:
    """Find transformer model files."""
    components: List[DiscoveredComponent] = []

    # Known single-file transformer patterns
    patterns = [
        "flux1-schnell.safetensors",
        "flux1-schnell.gguf",
        "flux1-dev.safetensors",
        "flux1-dev.gguf",
        "flux1-dev-Q8_0.gguf",
        "flux1-dev-Q5_K_S.gguf",
        "diffusion_pytorch_model.safetensors",
    ]

    for pattern in patterns:
        path = snapshot / pattern
        if path.exists():
            comp = _create_component_from_file(
                path, "transformer", repo_id, snapshot_hash
            )
            if comp:
                components.append(comp)

    # Check transformer/ subdirectory for sharded models
    transformer_dir = snapshot / "transformer"
    if transformer_dir.exists():
        shards = sorted(
            p for p in transformer_dir.iterdir()
            if p.suffix == ".safetensors"
        )
        if shards:
            comp = _create_sharded_component(
                shards, "transformer", repo_id, snapshot_hash
            )
            if comp:
                components.append(comp)

    return components


def _find_t5_encoders(
    snapshot: Path, repo_id: str, snapshot_hash: str
) -> List[DiscoveredComponent]:
    """Find T5 encoder files."""
    components: List[DiscoveredComponent] = []

    # GGUF quantized T5 files
    t5_patterns = [
        "t5-v1_1-xxl-encoder-Q5_K_M.gguf",
        "t5-v1_1-xxl-encoder-Q8_0.gguf",
        "t5-v1_1-xxl-encoder.gguf",
    ]
    for pattern in t5_patterns:
        path = snapshot / pattern
        if path.exists():
            comp = _create_component_from_file(
                path, "t5_encoder", repo_id, snapshot_hash
            )
            if comp:
                components.append(comp)

    # Check text_encoder_2/ for sharded safetensors
    t5_dir = snapshot / "text_encoder_2"
    if t5_dir.exists() and (t5_dir / "config.json").exists():
        shards = sorted(
            p for p in t5_dir.iterdir()
            if p.name.startswith("model-") and p.suffix == ".safetensors"
        )
        if shards:
            comp = _create_sharded_component(
                shards, "t5_encoder", repo_id, snapshot_hash
            )
            if comp:
                components.append(comp)

    return components


def _find_clip_encoders(
    snapshot: Path, repo_id: str, snapshot_hash: str
) -> List[DiscoveredComponent]:
    """Find CLIP encoder files."""
    clip_model = snapshot / "text_encoder" / "model.safetensors"
    if clip_model.exists():
        comp = _create_component_from_file(
            clip_model, "clip_encoder", repo_id, snapshot_hash
        )
        if comp:
            return [comp]
    return []


def _find_vae_decoders(
    snapshot: Path, repo_id: str, snapshot_hash: str
) -> List[DiscoveredComponent]:
    """Find VAE decoder files."""
    # Check native FLUX VAE
    vae_path = snapshot / "ae.safetensors"
    if vae_path.exists():
        comp = _create_component_from_file(
            vae_path, "vae", repo_id, snapshot_hash
        )
        if comp:
            return [comp]

    # Fallback: diffusers layout
    vae_model = snapshot / "vae" / "diffusion_pytorch_model.safetensors"
    if vae_model.exists():
        comp = _create_component_from_file(
            vae_model, "vae", repo_id, snapshot_hash
        )
        if comp:
            return [comp]

    return []


def _find_tokenizers(
    snapshot: Path, repo_id: str, snapshot_hash: str
) -> List[DiscoveredComponent]:
    """Find tokenizer files."""
    components: List[DiscoveredComponent] = []

    # CLIP tokenizer: tokenizer/ directory
    clip_tok_dir = snapshot / "tokenizer"
    if clip_tok_dir.is_dir():
        tok_file = clip_tok_dir / "tokenizer.json"
        if not tok_file.exists():
            tok_file = clip_tok_dir / "vocab.json"
        if tok_file.exists():
            comp = _create_tokenizer_component(
                tok_file, "clip_tokenizer", repo_id, snapshot_hash
            )
            if comp:
                components.append(comp)

    # T5 tokenizer: tokenizer_2/ directory
    t5_tok_dir = snapshot / "tokenizer_2"
    if t5_tok_dir.is_dir():
        tok_file = t5_tok_dir / "tokenizer.json"
        if not tok_file.exists():
            tok_file = t5_tok_dir / "spiece.model"
        if tok_file.exists():
            comp = _create_tokenizer_component(
                tok_file, "t5_tokenizer", repo_id, snapshot_hash
            )
            if comp:
                components.append(comp)

    # Standalone T5 tokenizer
    standalone = snapshot / "t5-v1_1-xxl.tokenizer.json"
    if standalone.exists():
        comp = _create_tokenizer_component(
            standalone, "t5_tokenizer", repo_id, snapshot_hash
        )
        if comp:
            components.append(comp)

    return components


# ============================================================================
# Directory Scanner
# ============================================================================


def scan_directory(
    dir_path: str,
    progress_callback: Optional[ProgressCallback] = None,
) -> List[DiscoveredComponent]:
    """
    Scan a directory recursively for model files.

    Identifies component types by inspecting safetensors headers and GGUF metadata.
    """
    directory = Path(dir_path)
    if not directory.exists():
        raise FileNotFoundError(f"Directory does not exist: {dir_path}")
    if not directory.is_dir():
        raise NotADirectoryError(f"Not a directory: {dir_path}")

    logger.info(f"Starting directory scan: {dir_path}")

    # Phase 1: discover model files
    model_files = _discover_model_files(directory)
    total = len(model_files)
    logger.info(f"Found {total} potential model files")

    if total == 0:
        return []

    # Phase 2: identify component types
    components: List[DiscoveredComponent] = []
    for idx, file_path in enumerate(model_files):
        filename = file_path.name

        if progress_callback:
            progress_callback(idx + 1, total, filename)

        # Skip unsupported variants
        if _is_unsupported_model(filename.lower()):
            logger.info(f"  [{idx + 1}/{total}] {filename} -> Unsupported variant, skipping")
            continue

        try:
            comp_type = identify_model_file(file_path)
            if comp_type:
                logger.info(f"  [{idx + 1}/{total}] {filename} -> {comp_type}")
                comp = _create_component_from_loose_file(file_path, comp_type)
                if comp and not comp.is_sharded:
                    components.append(comp)
            else:
                logger.debug(f"  [{idx + 1}/{total}] {filename} -> Unknown format")
        except Exception as e:
            logger.warning(f"  [{idx + 1}/{total}] {filename} -> Error: {e}")

    _log_scan_summary(components)
    return components


def _discover_model_files(directory: Path) -> List[Path]:
    """Recursively discover all safetensors and gguf files."""
    files: List[Path] = []
    skip_dirs = {".", "node_modules", "__pycache__", ".git"}

    for root, dirs, filenames in os.walk(directory):
        # Skip hidden and irrelevant directories
        dirs[:] = [d for d in dirs if d not in skip_dirs and not d.startswith(".")]

        for filename in filenames:
            ext = Path(filename).suffix.lower()
            if ext in (".safetensors", ".gguf"):
                files.append(Path(root) / filename)

    return sorted(files)


# ============================================================================
# File Identification
# ============================================================================


def identify_model_file(path: Path) -> Optional[str]:
    """Identify a model file's component type by inspecting its contents."""
    ext = path.suffix.lower()
    if ext == ".safetensors":
        return _identify_safetensors(path)
    elif ext == ".gguf":
        return _identify_gguf(path)
    return None


def _identify_safetensors(path: Path) -> Optional[str]:
    """Identify component type from a safetensors file by reading its header."""
    try:
        with open(path, "rb") as f:
            # Read header length (first 8 bytes, little-endian u64)
            header_len_bytes = f.read(8)
            if len(header_len_bytes) < 8:
                return None
            header_len = struct.unpack("<Q", header_len_bytes)[0]

            # Sanity check: max 10MB header
            if header_len > 10 * 1024 * 1024:
                logger.warning(f"Safetensors header too large: {header_len} bytes")
                return None

            header_bytes = f.read(header_len)
            header = json.loads(header_bytes)

        # Get tensor names (exclude __metadata__)
        tensor_names = [k for k in header.keys() if k != "__metadata__"]
        if not tensor_names:
            return None

        return _identify_from_tensor_names(tensor_names[:50])

    except Exception as e:
        logger.debug(f"Failed to parse safetensors header: {e}")
        return None


def _identify_gguf(path: Path) -> Optional[str]:
    """Identify component type from GGUF file metadata."""
    try:
        with open(path, "rb") as f:
            # Read magic bytes
            magic = f.read(4)
            if magic != b"GGUF":
                return None

            # Version (u32 LE)
            version = struct.unpack("<I", f.read(4))[0]
            if version < 2 or version > 3:
                logger.warning(f"Unsupported GGUF version: {version}")
                return None

            # Tensor count + KV count (both u64 LE)
            _tensor_count = struct.unpack("<Q", f.read(8))[0]
            kv_count = struct.unpack("<Q", f.read(8))[0]

            # Read metadata key-value pairs
            for _ in range(min(kv_count, 100)):
                try:
                    key = _read_gguf_string(f)
                    value_type = struct.unpack("<I", f.read(4))[0]

                    if value_type == 8:  # string
                        value = _read_gguf_string(f)
                        if key == "general.architecture":
                            arch = value.lower()
                            if "flux" in arch or arch == "diffusion":
                                return "transformer"
                            if "t5" in arch:
                                return "t5_encoder"
                            if "clip" in arch:
                                return "clip_encoder"
                        if key == "general.name":
                            name_lower = value.lower()
                            if "flux" in name_lower:
                                return "transformer"
                            if "t5" in name_lower:
                                return "t5_encoder"
                            if "clip" in name_lower:
                                return "clip_encoder"
                    else:
                        _skip_gguf_value(f, value_type)
                except Exception:
                    break

    except Exception as e:
        logger.debug(f"Failed to parse GGUF metadata: {e}")

    # Fallback: check filename
    filename = path.name.lower()
    if "flux" in filename or "schnell" in filename or "dev" in filename:
        return "transformer"
    if "t5" in filename:
        return "t5_encoder"
    if "clip" in filename:
        return "clip_encoder"
    if "vae" in filename or filename.startswith("ae."):
        return "vae"

    return None


def _identify_from_tensor_names(names: List[str]) -> Optional[str]:
    """Identify component type from tensor naming patterns."""
    # Reject ComfyUI format (model.diffusion_model. prefix)
    if any(n.startswith("model.diffusion_model.") for n in names):
        logger.debug("ComfyUI format detected (unsupported - use native FLUX format)")
        return None

    # LoRA patterns - check first since LoRAs can contain base model layer names
    has_lora = any(
        "lora_up" in n or "lora_down" in n or
        "lora_A" in n or "lora_B" in n or
        ".lora_a." in n or ".lora_b." in n or
        n.endswith(".alpha")
        for n in names
    )
    if has_lora:
        return "lora"

    # FLUX Transformer patterns
    if any("double_blocks" in n or "single_blocks" in n or "img_attn" in n or "img_mlp" in n for n in names):
        return "transformer"
    if any("transformer_blocks" in n for n in names):
        return "transformer"

    # T5 Encoder patterns
    if any("encoder.block" in n for n in names):
        return "t5_encoder"
    if any("shared.weight" in n for n in names) and any("SelfAttention" in n for n in names):
        return "t5_encoder"

    # CLIP Encoder patterns
    if any("text_model.encoder" in n for n in names):
        return "clip_encoder"

    # VAE patterns (diffusers format)
    if any("decoder.up_blocks" in n or "decoder.mid_block" in n for n in names):
        return "vae"
    if any("encoder.down_blocks" in n or "encoder.mid_block" in n for n in names):
        return "vae"
    if any("quant_conv" in n or "post_quant_conv" in n for n in names):
        return "vae"
    # VAE patterns (native FLUX format: decoder.conv_in, decoder.mid, decoder.up)
    if any("decoder.conv_in" in n for n in names) and any("decoder.mid" in n for n in names):
        return "vae"

    return None


# ============================================================================
# GGUF Helpers
# ============================================================================


def _read_gguf_string(f) -> str:
    """Read a GGUF length-prefixed string."""
    length = struct.unpack("<Q", f.read(8))[0]
    if length > 1024 * 1024:
        raise ValueError(f"GGUF string too long: {length}")
    return f.read(length).decode("utf-8", errors="replace")


def _skip_gguf_value(f, value_type: int) -> None:
    """Skip a GGUF value of a given type."""
    fixed_sizes = {
        0: 1, 1: 1, 2: 2, 3: 2, 4: 4, 5: 4, 6: 4, 7: 1,
        10: 8, 11: 8, 12: 8,
    }
    if value_type == 8:  # string
        length = struct.unpack("<Q", f.read(8))[0]
        f.seek(length, 1)
    elif value_type == 9:  # array
        arr_type = struct.unpack("<I", f.read(4))[0]
        arr_count = struct.unpack("<Q", f.read(8))[0]
        elem_size = fixed_sizes.get(arr_type, 0)
        if elem_size > 0:
            f.seek(elem_size * arr_count, 1)
    elif value_type in fixed_sizes:
        f.seek(fixed_sizes[value_type], 1)


# ============================================================================
# Component Builders
# ============================================================================


def _create_component_from_file(
    path: Path,
    comp_type: str,
    repo_id: str,
    snapshot_hash: str,
) -> Optional[DiscoveredComponent]:
    """Create a DiscoveredComponent from a single file."""
    try:
        file_size = path.stat().st_size
    except OSError:
        return None

    fmt = _format_from_path(path)
    if not fmt:
        return None

    quantization = _extract_quantization(path)
    architecture = _infer_architecture(path, repo_id, comp_type)
    vram_mb = _estimate_vram(file_size, comp_type, quantization)

    return DiscoveredComponent(
        component_type=comp_type,
        architecture=architecture,
        format=fmt,
        path=str(path),
        repo_id=repo_id,
        repo_snapshot=snapshot_hash,
        file_size=file_size,
        quantization=quantization,
        vram_mb=vram_mb,
    )


def _create_sharded_component(
    shards: List[Path],
    comp_type: str,
    repo_id: str,
    snapshot_hash: str,
) -> Optional[DiscoveredComponent]:
    """Create a DiscoveredComponent from sharded files."""
    if not shards:
        return None

    total_size = sum(p.stat().st_size for p in shards if p.exists())
    architecture = _infer_architecture(shards[0], repo_id, comp_type)
    vram_mb = _estimate_vram(total_size, comp_type, None)

    # Use parent directory as path for sharded models
    directory = shards[0].parent

    return DiscoveredComponent(
        component_type=comp_type,
        architecture=architecture,
        format=FORMAT_SAFETENSORS,
        path=str(directory),
        repo_id=repo_id,
        repo_snapshot=snapshot_hash,
        file_size=total_size,
        is_sharded=True,
        shard_count=len(shards),
        vram_mb=vram_mb,
        metadata={"shards": len(shards)},
    )


def _create_tokenizer_component(
    path: Path,
    comp_type: str,
    repo_id: str,
    snapshot_hash: str,
) -> Optional[DiscoveredComponent]:
    """Create a DiscoveredComponent for a tokenizer file."""
    if not path.is_file():
        return None

    try:
        file_size = path.stat().st_size
    except OSError:
        return None

    return DiscoveredComponent(
        component_type=comp_type,
        architecture="tokenizer",
        format=FORMAT_JSON,
        path=str(path),
        repo_id=repo_id,
        repo_snapshot=snapshot_hash,
        file_size=file_size,
    )


def _create_component_from_loose_file(
    path: Path, comp_type: str
) -> Optional[DiscoveredComponent]:
    """Create a DiscoveredComponent from a loose file (not in HF cache)."""
    fmt = _format_from_path(path)
    if not fmt:
        return None

    try:
        file_size = path.stat().st_size
    except OSError:
        return None

    quantization = _extract_quantization(path)
    vram_mb = _estimate_vram(file_size, comp_type, quantization)

    # Derive repo_id from parent directory
    repo_id = path.parent.name

    # Infer architecture from filename
    filename = path.name.lower()
    if comp_type == "transformer":
        if "schnell" in filename:
            architecture = "flux-schnell"
        elif "dev" in filename:
            architecture = "flux-dev"
        else:
            architecture = "flux"
    elif comp_type == "t5_encoder":
        architecture = "t5-v1_1-xxl"
    elif comp_type == "clip_encoder":
        architecture = "clip-l"
    elif comp_type == "vae":
        architecture = "flux-vae"
    else:
        architecture = comp_type

    return DiscoveredComponent(
        component_type=comp_type,
        architecture=architecture,
        format=fmt,
        path=str(path),
        repo_id=repo_id,
        file_size=file_size,
        quantization=quantization,
        vram_mb=vram_mb,
        metadata={"source": "directory_scan"},
    )


# ============================================================================
# Helpers
# ============================================================================


def _format_from_path(path: Path) -> Optional[str]:
    """Determine model format from file extension."""
    ext = path.suffix.lower()
    return {
        ".safetensors": FORMAT_SAFETENSORS,
        ".gguf": FORMAT_GGUF,
        ".json": FORMAT_JSON,
    }.get(ext)


def _extract_quantization(path: Path) -> Optional[str]:
    """Extract quantization info from filename."""
    filename = path.name
    for quant in _QUANT_PATTERNS:
        if quant in filename:
            return quant
    return None


def _infer_architecture(path: Path, repo_id: str, comp_type: str) -> str:
    """Infer architecture from path and repo."""
    filename = path.name.lower()

    if comp_type == "transformer":
        if "schnell" in filename:
            return "flux-schnell"
        if "dev" in filename:
            return "flux-dev"
        if "z-image" in repo_id.lower() or "zimage" in repo_id.lower():
            return "z-image-turbo"
        return "flux"
    elif comp_type == "t5_encoder":
        return "t5-v1_1-xxl"
    elif comp_type == "clip_encoder":
        return "clip-l"
    elif comp_type == "vae":
        return "flux-vae"
    elif comp_type == "clip_tokenizer":
        return "clip-tokenizer"
    elif comp_type == "t5_tokenizer":
        return "t5-tokenizer"
    elif comp_type == "lora":
        return "flux-lora"
    return comp_type


def _estimate_vram(file_size: int, comp_type: str, quantization: Optional[str]) -> Optional[int]:
    """Estimate VRAM usage in MB."""
    size_mb = file_size // 1_000_000

    if comp_type == "transformer":
        return size_mb + 1000
    elif comp_type == "t5_encoder":
        return 3500 if quantization else 9000
    elif comp_type == "clip_encoder":
        return 500
    elif comp_type == "vae":
        return 200
    return None


def _is_unsupported_model(filename_lower: str) -> bool:
    """Check if a filename indicates an unsupported variant."""
    return any(kw in filename_lower for kw in _UNSUPPORTED_KEYWORDS)


def _generate_component_name(
    repo_id: str, comp_type: str, quantization: Optional[str], is_sharded: bool
) -> str:
    """Generate a descriptive display name for a component."""
    repo_lower = repo_id.lower()

    if comp_type == "transformer":
        if "schnell" in repo_lower:
            base = "FLUX.1 Schnell Transformer"
        elif "dev" in repo_lower:
            base = "FLUX.1 Dev Transformer"
        elif "z-image" in repo_lower:
            base = "Z-Image Turbo Transformer"
        else:
            base = "FLUX Transformer"
    elif comp_type == "t5_encoder":
        base = "T5-XXL Text Encoder"
    elif comp_type == "clip_encoder":
        base = "CLIP-L Text Encoder"
    elif comp_type == "vae":
        base = "FLUX VAE Decoder"
    elif comp_type == "clip_tokenizer":
        base = "CLIP Tokenizer"
    elif comp_type == "t5_tokenizer":
        base = "T5 Tokenizer"
    elif comp_type == "lora":
        base = "LoRA Adapter"
    else:
        base = comp_type

    if quantization:
        base += f" ({quantization})"
    if is_sharded:
        base += " [Sharded]"

    return base


def filter_sharded_components(
    components: List[DiscoveredComponent],
) -> List[DiscoveredComponent]:
    """Filter out sharded components when a non-sharded alternative exists.

    For each (repo_id, component_type) pair, sharded models are only kept
    when no non-sharded alternative exists for that specific component type.
    """
    non_sharded_pairs = {
        (c.repo_id, c.component_type)
        for c in components
        if not c.is_sharded
    }

    return [
        c for c in components
        if not c.is_sharded
        or (c.repo_id, c.component_type) not in non_sharded_pairs
    ]


def _log_scan_summary(components: List[DiscoveredComponent]) -> None:
    """Log a summary of discovered components."""
    by_type = {}
    for comp in components:
        by_type[comp.component_type] = by_type.get(comp.component_type, 0) + 1

    logger.info(f"Scan complete: {len(components)} total components")
    for comp_type, count in sorted(by_type.items()):
        logger.info(f"  - {comp_type}: {count}")
