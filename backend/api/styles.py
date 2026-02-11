"""Styles API mixin — style, style-lora, and style-tag operations for the frontend."""

from __future__ import annotations

import logging
import os
import uuid
from typing import Any

import webview

from backend.db.database import Database

logger = logging.getLogger(__name__)

_LORA_EXTENSIONS = ("safetensors", "ckpt", "pt")
_IMAGE_EXTENSIONS = ("png", "jpg", "jpeg", "webp", "bmp")


class StylesAPI:
    """pywebview js_api mixin for style management."""

    def __init__(self, db: Database) -> None:
        self._db = db

    # ── Styles ────────────────────────────────────────────────

    def get_styles(
        self,
        category: str | None = None,
        tag_id: int | None = None,
        search: str | None = None,
        favorites_only: bool = False,
        **kwargs,
    ) -> dict[str, Any]:
        try:
            styles = self._db.get_styles(
                category=category,
                tag_id=tag_id,
                search=search,
                favorites_only=favorites_only,
            )
            return {"status": "success", "styles": styles}
        except Exception as e:
            logger.error("Failed to get styles: %s", e)
            return {"status": "error", "message": str(e)}

    def get_style(self, style_id: str, **kwargs) -> dict[str, Any]:
        try:
            style = self._db.get_style(style_id)
            if not style:
                return {"status": "error", "message": f"Style '{style_id}' not found"}
            loras = self._db.get_style_loras(style_id)
            tags = self._db.get_style_tags(style_id)
            return {"status": "success", "style": style, "loras": loras, "tags": tags}
        except Exception as e:
            logger.error("Failed to get style: %s", e)
            return {"status": "error", "message": str(e)}

    def create_style(
        self,
        id: str,
        name: str,
        prompt_template: str,
        description: str | None = None,
        negative_prompt: str | None = None,
        category: str | None = None,
        thumbnail_path: str | None = None,
        **kwargs,
    ) -> dict[str, Any]:
        try:
            style = self._db.insert_style(
                id=id, name=name, prompt_template=prompt_template,
                description=description, negative_prompt=negative_prompt,
                category=category, thumbnail_path=thumbnail_path,
            )
            return {"status": "success", "style": style}
        except Exception as e:
            logger.error("Failed to create style: %s", e)
            return {"status": "error", "message": str(e)}

    def update_style(self, style_id: str, **kwargs) -> dict[str, Any]:
        try:
            style = self._db.update_style(style_id, **kwargs)
            if not style:
                return {"status": "error", "message": f"Style '{style_id}' not found"}
            return {"status": "success", "style": style}
        except Exception as e:
            logger.error("Failed to update style: %s", e)
            return {"status": "error", "message": str(e)}

    def delete_style(self, style_id: str, **kwargs) -> dict[str, Any]:
        try:
            self._db.delete_style(style_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to delete style: %s", e)
            return {"status": "error", "message": str(e)}

    def toggle_style_favorite(self, style_id: str, **kwargs) -> dict[str, Any]:
        try:
            style = self._db.toggle_style_favorite(style_id)
            if not style:
                return {"status": "error", "message": f"Style '{style_id}' not found"}
            return {"status": "success", "style": style}
        except Exception as e:
            logger.error("Failed to toggle style favorite: %s", e)
            return {"status": "error", "message": str(e)}

    def get_style_categories(self, **kwargs) -> dict[str, Any]:
        try:
            categories = self._db.get_style_categories()
            return {"status": "success", "categories": categories}
        except Exception as e:
            logger.error("Failed to get style categories: %s", e)
            return {"status": "error", "message": str(e)}

    # ── Style ↔ LoRA ──────────────────────────────────────────

    def get_style_loras(self, style_id: str, **kwargs) -> dict[str, Any]:
        try:
            loras = self._db.get_style_loras(style_id)
            return {"status": "success", "loras": loras}
        except Exception as e:
            logger.error("Failed to get style loras: %s", e)
            return {"status": "error", "message": str(e)}

    def set_style_loras(self, style_id: str, loras: list | None = None, **kwargs) -> dict[str, Any]:
        try:
            result = self._db.set_style_loras(style_id, loras or [])
            return {"status": "success", "loras": result}
        except Exception as e:
            logger.error("Failed to set style loras: %s", e)
            return {"status": "error", "message": str(e)}

    # ── Style ↔ Tag ───────────────────────────────────────────

    def get_style_tags(self, style_id: str, **kwargs) -> dict[str, Any]:
        try:
            tags = self._db.get_style_tags(style_id)
            return {"status": "success", "tags": tags}
        except Exception as e:
            logger.error("Failed to get style tags: %s", e)
            return {"status": "error", "message": str(e)}

    def set_style_tags(self, style_id: str, tag_ids: list | None = None, **kwargs) -> dict[str, Any]:
        try:
            tags = self._db.set_style_tags(style_id, tag_ids or [])
            return {"status": "success", "tags": tags}
        except Exception as e:
            logger.error("Failed to set style tags: %s", e)
            return {"status": "error", "message": str(e)}

    # ── LoRAs ─────────────────────────────────────────────────

    def get_loras(self, **kwargs) -> dict[str, Any]:
        try:
            loras = self._db.get_loras()
            return {"status": "success", "loras": loras}
        except Exception as e:
            logger.error("Failed to get loras: %s", e)
            return {"status": "error", "message": str(e)}

    def create_lora(
        self,
        id: str,
        name: str,
        path: str,
        trigger_words: str | None = None,
        base_model: str | None = None,
        size_bytes: int | None = None,
        strength: float = 1.0,
        **kwargs,
    ) -> dict[str, Any]:
        try:
            lora = self._db.insert_lora(
                id=id, name=name, path=path,
                trigger_words=trigger_words, base_model=base_model,
                size_bytes=size_bytes, strength=strength,
            )
            return {"status": "success", "lora": lora}
        except Exception as e:
            logger.error("Failed to create lora: %s", e)
            return {"status": "error", "message": str(e)}

    def browse_lora_files(self, **kwargs) -> dict[str, Any]:
        """Open a native file dialog to select LoRA files and register them."""
        try:
            window = webview.windows[0]
            file_filter = "LoRA files (" + ";".join(f"*.{ext}" for ext in _LORA_EXTENSIONS) + ")"
            result = window.create_file_dialog(
                webview.FileDialog.OPEN,
                allow_multiple=True,
                file_types=(file_filter,),
            )
            if not result:
                return {"status": "success", "loras": []}

            created = []
            for filepath in result:
                filepath = str(filepath)
                if not os.path.isfile(filepath):
                    continue
                name = os.path.splitext(os.path.basename(filepath))[0]
                size_bytes = os.path.getsize(filepath)
                lora = self._db.insert_lora(
                    id=str(uuid.uuid4()),
                    name=name,
                    path=filepath,
                    size_bytes=size_bytes,
                )
                if lora:
                    created.append(lora)

            return {"status": "success", "loras": created}
        except Exception as e:
            logger.error("Failed to browse lora files: %s", e)
            return {"status": "error", "message": str(e)}

    def browse_image_file(self, **kwargs) -> dict[str, Any]:
        """Open a native file dialog to select a single image file."""
        try:
            window = webview.windows[0]
            file_filter = "Images (" + ";".join(f"*.{ext}" for ext in _IMAGE_EXTENSIONS) + ")"
            result = window.create_file_dialog(
                webview.FileDialog.OPEN,
                allow_multiple=False,
                file_types=(file_filter,),
            )
            if not result:
                return {"status": "success", "path": None}
            path = str(result[0]) if isinstance(result, (list, tuple)) else str(result)
            return {"status": "success", "path": path}
        except Exception as e:
            logger.error("Failed to browse image file: %s", e)
            return {"status": "error", "message": str(e)}
