"""Settings API mixin — GPU info, VRAM, engine status, HF cache, and data paths."""

from __future__ import annotations

import logging
import os
from pathlib import Path
from typing import Any

from backend.config import AppConfig
from backend.services.inference_manager import InferenceServiceManager

logger = logging.getLogger(__name__)


def _dir_size_bytes(path: Path) -> int:
    """Recursively compute total bytes of all files under *path*."""
    total = 0
    try:
        for entry in os.scandir(path):
            if entry.is_file(follow_symlinks=False):
                total += entry.stat().st_size
            elif entry.is_dir(follow_symlinks=False):
                total += _dir_size_bytes(Path(entry.path))
    except (PermissionError, FileNotFoundError):
        pass
    return total


class SettingsAPI:
    """pywebview js_api mixin for settings pages."""

    def __init__(self, inference: InferenceServiceManager, config: AppConfig) -> None:
        self._inference = inference
        self._config = config

    # ── VRAM ──────────────────────────────────────────────────────

    def get_vram_usage(self) -> dict[str, Any]:
        """Return VRAM breakdown: allocated, reserved, free, total (bytes)."""
        try:
            import torch
            from rzem_ai_inference_engine.models.memory import detect_device, get_total_vram

            if not torch.cuda.is_available():
                return {"status": "success", "available": False}

            allocated = torch.cuda.memory_allocated()
            reserved = torch.cuda.memory_reserved()
            total = get_total_vram(detect_device())

            return {
                "status": "success",
                "available": True,
                "allocated": allocated,
                "reserved": reserved,
                "free": total - reserved,
                "total": total,
            }
        except Exception as e:
            logger.error("Failed to get VRAM usage: %s", e)
            return {"status": "error", "message": str(e)}

    def clear_vram_cache(self) -> dict[str, Any]:
        """Free unused VRAM held by PyTorch's caching allocator."""
        try:
            import torch

            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to clear VRAM cache: %s", e)
            return {"status": "error", "message": str(e)}

    # ── Engine ────────────────────────────────────────────────────

    def get_engine_status(self) -> dict[str, Any]:
        """Return engine readiness, uptime, and job count."""
        try:
            status = self._inference.active.get_engine_status()
            return {"status": "success", **status}
        except Exception as e:
            logger.error("Failed to get engine status: %s", e)
            return {"status": "error", "message": str(e)}

    def get_cuda_version(self) -> dict[str, Any]:
        """Return CUDA toolkit version string."""
        try:
            import torch
            return {"status": "success", "cuda_version": torch.version.cuda}
        except Exception as e:
            logger.error("Failed to get CUDA version: %s", e)
            return {"status": "error", "message": str(e)}

    def reset_engine(self) -> dict[str, Any]:
        """Stop engine, clear VRAM, restart."""
        try:
            self._inference.local.shutdown()
            import torch
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to reset engine: %s", e)
            return {"status": "error", "message": str(e)}

    # ── HuggingFace Cache ─────────────────────────────────────────

    def get_cache_info(self) -> dict[str, Any]:
        """List cached HuggingFace models with size information."""
        try:
            from huggingface_hub import scan_cache_dir

            cache_info = scan_cache_dir()
            models = []
            for repo in cache_info.repos:
                size_bytes = repo.size_on_disk
                revisions = []
                for rev in repo.revisions:
                    revisions.append({
                        "commit_hash": rev.commit_hash,
                        "size_on_disk": rev.size_on_disk,
                        "last_modified": rev.last_modified.timestamp(),
                    })
                models.append({
                    "repo_id": repo.repo_id,
                    "repo_type": repo.repo_type,
                    "size_on_disk": size_bytes,
                    "nb_files": repo.nb_files,
                    "revisions": revisions,
                    "last_accessed": repo.last_accessed,
                    "last_modified": repo.last_modified,
                })

            return {
                "status": "success",
                "models": models,
                "total_size": cache_info.size_on_disk,
            }
        except Exception as e:
            logger.error("Failed to get cache info: %s", e)
            return {"status": "error", "message": str(e)}

    def delete_cached_model(self, repo_id: str, **kwargs) -> dict[str, Any]:
        """Remove a specific cached model from the HuggingFace cache."""
        try:
            from huggingface_hub import scan_cache_dir

            cache_info = scan_cache_dir()
            # Collect all revision commit hashes for this repo
            hashes_to_delete = set()
            for repo in cache_info.repos:
                if repo.repo_id == repo_id:
                    for rev in repo.revisions:
                        hashes_to_delete.add(rev.commit_hash)
                    break

            if not hashes_to_delete:
                return {"status": "error", "message": f"Model '{repo_id}' not found in cache"}

            strategy = cache_info.delete_revisions(*hashes_to_delete)
            strategy.execute()
            logger.info("Deleted cached model %s (%d revisions)", repo_id, len(hashes_to_delete))
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to delete cached model %s: %s", repo_id, e)
            return {"status": "error", "message": str(e)}

    # ── Data Paths & Disk Usage ───────────────────────────────────

    def get_data_paths(self) -> dict[str, Any]:
        """Return key directory paths for display."""
        try:
            hf_cache_dir = None
            try:
                from huggingface_hub.constants import HF_HUB_CACHE
                hf_cache_dir = str(HF_HUB_CACHE)
            except ImportError:
                pass

            return {
                "status": "success",
                "data_dir": str(self._config.data_dir),
                "output_dir": str(self._config.output_dir),
                "hf_cache_dir": hf_cache_dir,
            }
        except Exception as e:
            logger.error("Failed to get data paths: %s", e)
            return {"status": "error", "message": str(e)}

    def get_disk_usage(self) -> dict[str, Any]:
        """Return disk usage for output dir and HF cache."""
        try:
            output_size = _dir_size_bytes(self._config.output_dir)

            hf_cache_size = 0
            try:
                from huggingface_hub.constants import HF_HUB_CACHE
                hf_cache_size = _dir_size_bytes(Path(HF_HUB_CACHE))
            except ImportError:
                pass

            return {
                "status": "success",
                "output_size": output_size,
                "hf_cache_size": hf_cache_size,
            }
        except Exception as e:
            logger.error("Failed to get disk usage: %s", e)
            return {"status": "error", "message": str(e)}
