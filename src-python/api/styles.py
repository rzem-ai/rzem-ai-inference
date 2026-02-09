"""Style management API methods."""

from typing import Optional, List, Dict, Any
from loguru import logger

from api.base import ApiBase


class StyleApiMixin(ApiBase):

    def get_all_styles(self) -> List[Dict[str, Any]]:
        """Get all styles."""
        try:
            if not self._app_state.db:
                return []
            styles = self._run_async(self._app_state.db.get_all_styles())
            return styles
        except Exception as e:
            logger.error(f"Failed to get styles: {e}")
            return []

    def get_style_detail(self, style_id: str) -> Optional[Dict[str, Any]]:
        """Get detailed style information including LoRAs and examples."""
        try:
            if not self._app_state.db:
                return None
            detail = self._run_async(self._app_state.db.get_style_detail(style_id))
            return detail
        except Exception as e:
            logger.error(f"Failed to get style detail: {e}")
            return None

    def create_style(self, style: Dict[str, Any]) -> Dict[str, Any]:
        """Create a new style."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            style_id = self._run_async(self._app_state.db.create_style(style))
            logger.info(f"Style created: {style_id}")
            return {"status": "success", "id": style_id}
        except Exception as e:
            logger.error(f"Failed to create style: {e}")
            return {"status": "error", "message": str(e)}

    def update_style(
        self,
        style_id: str,
        name: Optional[str] = None,
        description: Optional[str] = None,
        prompt_template: Optional[str] = None,
        default_strength: Optional[float] = None,
        strength_min: Optional[float] = None,
        strength_max: Optional[float] = None,
        category: Optional[str] = None,
        is_favorite: Optional[bool] = None,
        **kwargs,
    ) -> Dict[str, str]:
        """Update a style."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            # Rebuild camelCase dict for DB method
            style_data = {
                "name": name,
                "description": description,
                "promptTemplate": prompt_template,
                "defaultStrength": default_strength,
                "strengthMin": strength_min,
                "strengthMax": strength_max,
                "category": category,
                "isFavorite": is_favorite,
            }

            self._run_async(self._app_state.db.update_style(style_id, style_data))
            logger.info(f"Style updated: {style_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to update style: {e}")
            return {"status": "error", "message": str(e)}

    def delete_style(self, style_id: str) -> Dict[str, str]:
        """Delete a style."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_style(style_id))
            if success:
                logger.info(f"Style deleted: {style_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Style not found"}
        except Exception as e:
            logger.error(f"Failed to delete style: {e}")
            return {"status": "error", "message": str(e)}

    def add_lora_to_style(
        self, style_id: str, lora_id: str, strength: float = 1.0, priority: int = 0
    ) -> Dict[str, str]:
        """Add a LoRA to a style."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.add_lora_to_style(
                style_id, lora_id, strength, priority
            ))
            logger.info(f"LoRA added to style: {lora_id} -> {style_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to add LoRA to style: {e}")
            return {"status": "error", "message": str(e)}

    def remove_lora_from_style(self, style_id: str, lora_id: str) -> Dict[str, str]:
        """Remove a LoRA from a style."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.remove_lora_from_style(
                style_id, lora_id
            ))
            if success:
                logger.info(f"LoRA removed from style: {lora_id} -> {style_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "LoRA association not found"}
        except Exception as e:
            logger.error(f"Failed to remove LoRA from style: {e}")
            return {"status": "error", "message": str(e)}

    def add_style_example(
        self,
        style_id: str,
        example_type: str,
        content: str,
        generation_params: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Add an example to a style."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            example_id = self._run_async(self._app_state.db.add_style_example(
                style_id, example_type, content, generation_params
            ))
            logger.info(f"Example added to style: {example_id} -> {style_id}")
            return {"status": "success", "id": example_id}
        except Exception as e:
            logger.error(f"Failed to add style example: {e}")
            return {"status": "error", "message": str(e)}

    def remove_style_example(self, example_id: str) -> Dict[str, str]:
        """Remove an example from a style."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.remove_style_example(example_id))
            if success:
                logger.info(f"Style example removed: {example_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Example not found"}
        except Exception as e:
            logger.error(f"Failed to remove style example: {e}")
            return {"status": "error", "message": str(e)}

    def render_style_template(self, template: str, variables: Dict[str, str]) -> Dict[str, Any]:
        """Render a style template preview."""
        try:
            rendered = template
            for key, value in variables.items():
                rendered = rendered.replace(f"{{{key}}}", value)

            return {"status": "success", "rendered": rendered}
        except Exception as e:
            logger.error(f"Failed to render template: {e}")
            return {"status": "error", "message": str(e)}

    def upload_style_thumbnail(self, style_id: str, thumbnail_path: str) -> Dict[str, str]:
        """Upload/set style thumbnail image."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.update_style_thumbnail(style_id, thumbnail_path))
            logger.info(f"Style thumbnail updated: {style_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to upload style thumbnail: {e}")
            return {"status": "error", "message": str(e)}

    def delete_style_thumbnail(self, style_id: str) -> Dict[str, str]:
        """Delete style thumbnail image."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.update_style_thumbnail(style_id, None))
            logger.info(f"Style thumbnail deleted: {style_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to delete style thumbnail: {e}")
            return {"status": "error", "message": str(e)}

    def increment_style_usage(self, style_id: str) -> Dict[str, str]:
        """Increment usage count for a style."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.increment_style_usage(style_id))
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to increment style usage: {e}")
            return {"status": "error", "message": str(e)}
