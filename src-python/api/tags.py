"""Tag management and auto-tagging API methods."""

from typing import Optional, List, Dict, Any
from loguru import logger

from api.base import ApiBase


class TagApiMixin(ApiBase):

    def get_all_tags(self) -> List[Dict[str, Any]]:
        """Get all tags."""
        try:
            if not self._app_state.db:
                return []
            tags = self._run_async(self._app_state.db.get_all_tags())
            return tags
        except Exception as e:
            logger.error(f"Failed to get tags: {e}")
            return []

    def update_tag(
        self,
        id: int,
        name: Optional[str] = None,
        color: Optional[str] = None,
        category: Optional[str] = None,
    ) -> Dict[str, str]:
        """Update a tag."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.update_tag(
                tag_id=id,
                name=name,
                color=color,
                category=category,
            ))
            logger.info(f"Tag updated: {id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to update tag: {e}")
            return {"status": "error", "message": str(e)}

    def delete_tag(self, tag_id: int) -> Dict[str, str]:
        """Delete a tag."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_tag(tag_id))
            if success:
                logger.info(f"Tag deleted: {tag_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Tag not found"}
        except Exception as e:
            logger.error(f"Failed to delete tag: {e}")
            return {"status": "error", "message": str(e)}

    def add_image_tag(self, image_id: str, tag: str) -> Dict[str, str]:
        """Add tag to image."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.add_image_tag(image_id, tag))
            logger.info(f"Tag added to image: {tag} -> {image_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to add image tag: {e}")
            return {"status": "error", "message": str(e)}

    def remove_image_tag(self, image_id: str, tag: str) -> Dict[str, str]:
        """Remove tag from image."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.remove_image_tag(image_id, tag))
            logger.info(f"Tag removed from image: {tag} -> {image_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to remove image tag: {e}")
            return {"status": "error", "message": str(e)}

    def bulk_add_tag(self, image_ids: List[str], tag: str) -> Dict[str, str]:
        """Add tag to multiple images."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.bulk_add_tag(image_ids, tag))
            logger.info(f"Tag bulk added: {tag} to {len(image_ids)} images")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to bulk add tag: {e}")
            return {"status": "error", "message": str(e)}

    def bulk_remove_tag(self, image_ids: List[str], tag: str) -> Dict[str, str]:
        """Remove tag from multiple images."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.bulk_remove_tag(image_ids, tag))
            logger.info(f"Tag bulk removed: {tag} from {len(image_ids)} images")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to bulk remove tag: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Auto-Tag ====================

    def get_auto_tag_settings(self) -> Dict[str, Any]:
        """Get auto-tag settings."""
        try:
            if self._app_state.db:
                enabled = self._run_async(self._app_state.db.get_setting("auto_tag_enabled"))
                auto_tag_on_gen = self._run_async(self._app_state.db.get_setting("auto_tag_on_generation"))
                backend = self._run_async(self._app_state.db.get_setting("auto_tag_backend"))
                min_conf = self._run_async(self._app_state.db.get_setting("auto_tag_min_confidence"))

                return {
                    "enabled": enabled == "true" if enabled else False,
                    "autoTagOnGeneration": auto_tag_on_gen == "true" if auto_tag_on_gen else False,
                    "preferredBackend": backend if backend else "claude",
                    "minConfidence": float(min_conf) if min_conf else 0.6,
                }
        except Exception as e:
            logger.error(f"Failed to get auto-tag settings: {e}")

        return {
            "enabled": False,
            "auto_tag_on_generation": False,
            "preferred_backend": "claude",
            "min_confidence": 0.6,
        }

    def update_auto_tag_settings(self, settings: Dict[str, Any]) -> Dict[str, str]:
        """Update auto-tag settings."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            if "enabled" in settings:
                val = "true" if settings["enabled"] else "false"
                self._run_async(self._app_state.db.set_setting("auto_tag_enabled", val))

            if "autoTagOnGeneration" in settings:
                val = "true" if settings["autoTagOnGeneration"] else "false"
                self._run_async(self._app_state.db.set_setting("auto_tag_on_generation", val))

            if "preferredBackend" in settings:
                self._run_async(self._app_state.db.set_setting("auto_tag_backend", settings["preferredBackend"]))

            if "minConfidence" in settings:
                self._run_async(self._app_state.db.set_setting("auto_tag_min_confidence", str(settings["minConfidence"])))

            logger.info("Auto-tag settings updated")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to update auto-tag settings: {e}")
            return {"status": "error", "message": str(e)}

    def check_vision_model_status(self) -> Dict[str, Any]:
        """Check vision model status."""
        logger.debug("check_vision_model_status stub called")
        return {
            "isDownloaded": False,
            "downloadProgress": None,
            "modelSize": 0,
            "modelSizeDisplay": "0 MB",
            "error": None,
        }

    def download_vision_model(self) -> Dict[str, str]:
        """Download vision model."""
        logger.debug("download_vision_model stub called")
        return {"status": "error", "message": "Vision model download not implemented"}

    def clear_vision_model_locks(self) -> Dict[str, str]:
        """Clear vision model lock files."""
        logger.debug("clear_vision_model_locks stub called")
        return {"status": "success"}

    def auto_tag_images(self, image_ids: List[str]) -> Dict[str, str]:
        """Auto-tag images using vision model."""
        logger.debug(f"auto_tag_images stub called: {len(image_ids)} images")
        return {"status": "error", "message": "Auto-tagging not implemented"}
