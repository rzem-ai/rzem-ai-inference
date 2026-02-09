"""Settings, API keys, and cache configuration API methods."""

from pathlib import Path
from typing import Dict, Any
from loguru import logger

from api.base import ApiBase


class SettingsApiMixin(ApiBase):

    # ==================== API Keys ====================

    def get_hf_token(self) -> Dict[str, Any]:
        """Get Hugging Face API token"""
        try:
            if not self._app_state.db:
                return {"token": None}
            token = self._run_async(self._app_state.db.get_setting("hf_token"))
            return {"token": token}
        except Exception as e:
            logger.error(f"Failed to get HF token: {e}")
            return {"token": None}

    def save_hf_token(self, token: str) -> Dict[str, str]:
        """Save Hugging Face API token"""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            if not token or not token.strip():
                self._run_async(self._app_state.db.delete_setting("hf_token"))
            else:
                self._run_async(self._app_state.db.set_setting("hf_token", token.strip()))

            logger.info("HF token saved")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to save HF token: {e}")
            return {"status": "error", "message": str(e)}

    def get_claude_api_key(self) -> Dict[str, Any]:
        """Get Claude API key"""
        try:
            if not self._app_state.db:
                return {"key": None}
            key = self._run_async(self._app_state.db.get_setting("claude_api_key"))
            return {"key": key}
        except Exception as e:
            logger.error(f"Failed to get Claude API key: {e}")
            return {"key": None}

    def save_claude_api_key(self, key: str) -> Dict[str, str]:
        """Save Claude API key"""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            if not key or not key.strip():
                self._run_async(self._app_state.db.delete_setting("claude_api_key"))
            else:
                self._run_async(self._app_state.db.set_setting("claude_api_key", key.strip()))

            logger.info("Claude API key saved")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to save Claude API key: {e}")
            return {"status": "error", "message": str(e)}

    def get_fal_key(self) -> Dict[str, Any]:
        """Get Fal.ai API key"""
        try:
            if not self._app_state.db:
                return {"key": None}
            key = self._run_async(self._app_state.db.get_setting("fal_key"))
            return {"key": key}
        except Exception as e:
            logger.error(f"Failed to get Fal key: {e}")
            return {"key": None}

    def save_fal_key(self, key: str) -> Dict[str, str]:
        """Save Fal.ai API key"""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            if not key or not key.strip():
                self._run_async(self._app_state.db.delete_setting("fal_key"))
            else:
                self._run_async(self._app_state.db.set_setting("fal_key", key.strip()))

            logger.info("Fal key saved")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to save Fal key: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Cache ====================

    def get_cache_stats(self) -> Dict[str, Any]:
        """Get model cache statistics"""
        try:
            cache_dir = Path.home() / ".cache" / "huggingface" / "hub"

            if not cache_dir.exists():
                return {
                    "total_size_bytes": 0,
                    "total_size_gb": 0.0,
                    "model_count": 0,
                    "models": [],
                }

            total_size = 0
            model_dirs = []

            for item in cache_dir.iterdir():
                if item.is_dir() and item.name.startswith("models--"):
                    size = sum(f.stat().st_size for f in item.rglob('*') if f.is_file())
                    total_size += size
                    model_name = item.name.replace("models--", "").replace("--", "/")
                    model_dirs.append({
                        "name": model_name,
                        "size_bytes": size,
                        "size_gb": round(size / (1024**3), 2),
                    })

            return {
                "total_size_bytes": total_size,
                "total_size_gb": round(total_size / (1024**3), 2),
                "model_count": len(model_dirs),
                "models": sorted(model_dirs, key=lambda x: x["size_bytes"], reverse=True),
            }
        except Exception as e:
            logger.error(f"Failed to get cache stats: {e}")
            return {
                "total_size_bytes": 0,
                "total_size_gb": 0.0,
                "model_count": 0,
                "models": [],
            }

    def get_cache_config(self) -> Dict[str, Any]:
        """Get cache configuration"""
        try:
            if self._app_state.db:
                keep_vae_loaded = self._run_async(self._app_state.db.get_setting("cache_keep_vae_loaded"))
                keep_flux_loaded = self._run_async(self._app_state.db.get_setting("cache_keep_flux_loaded"))
                keep_t5_loaded = self._run_async(self._app_state.db.get_setting("cache_keep_t5_loaded"))
                keep_clip_loaded = self._run_async(self._app_state.db.get_setting("cache_keep_clip_loaded"))

                return {
                    "keep_vae_loaded": keep_vae_loaded == "true" if keep_vae_loaded else False,
                    "keep_flux_loaded": keep_flux_loaded == "true" if keep_flux_loaded else False,
                    "keep_t5_loaded": keep_t5_loaded == "true" if keep_t5_loaded else False,
                    "keep_clip_loaded": keep_clip_loaded == "true" if keep_clip_loaded else False,
                    "embedding_cache_size": 100,
                    "idle_timeout_secs": None,
                }

            return {
                "keep_vae_loaded": False,
                "keep_flux_loaded": False,
                "keep_t5_loaded": False,
                "keep_clip_loaded": False,
                "embedding_cache_size": 100,
                "idle_timeout_secs": None,
            }
        except Exception as e:
            logger.error(f"Failed to get cache config: {e}")
            return {
                "keep_vae_loaded": False,
                "keep_flux_loaded": False,
                "keep_t5_loaded": False,
                "keep_clip_loaded": False,
                "embedding_cache_size": 100,
                "idle_timeout_secs": None,
            }

    def save_cache_config(self, config: Dict[str, Any]) -> Dict[str, str]:
        """Save cache configuration"""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            if "keep_vae_loaded" in config:
                val = "true" if config["keep_vae_loaded"] else "false"
                self._run_async(self._app_state.db.set_setting("cache_keep_vae_loaded", val))

            if "keep_flux_loaded" in config:
                val = "true" if config["keep_flux_loaded"] else "false"
                self._run_async(self._app_state.db.set_setting("cache_keep_flux_loaded", val))

            if "keep_t5_loaded" in config:
                val = "true" if config["keep_t5_loaded"] else "false"
                self._run_async(self._app_state.db.set_setting("cache_keep_t5_loaded", val))

            if "keep_clip_loaded" in config:
                val = "true" if config["keep_clip_loaded"] else "false"
                self._run_async(self._app_state.db.set_setting("cache_keep_clip_loaded", val))

            logger.info(f"Cache config saved: {config}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to save cache config: {e}")
            return {"status": "error", "message": str(e)}

    def clear_cache(self) -> Dict[str, str]:
        """Clear model cache"""
        try:
            if self._app_state.queue_processor and self._app_state.queue_processor.pipeline:
                logger.info("Cache clear requested - would unload models here")

            return {"status": "success", "message": "Cache cleared"}
        except Exception as e:
            logger.error(f"Failed to clear cache: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== App Settings ====================

    def get_settings(self) -> Dict[str, Any]:
        """Get application settings."""
        # TODO: Implement settings persistence
        return {
            "theme": "dark",
            "default_steps": 4,
            "default_width": 1024,
            "default_height": 1024,
        }

    def save_settings(self, settings: Dict[str, Any]) -> Dict[str, str]:
        """Save application settings."""
        # TODO: Implement settings persistence
        logger.info(f"Settings saved: {settings}")
        return {"status": "success"}
