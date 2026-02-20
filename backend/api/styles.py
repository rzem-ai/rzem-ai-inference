"""Styles API mixin — style, style-lora, and style-tag operations for the frontend."""

from __future__ import annotations

import json
import logging
import os
import re
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
        sort_by: str = "updated_at",
        sort_order: str = "desc",
        **kwargs,
    ) -> dict[str, Any]:
        try:
            styles = self._db.get_styles(
                category=category,
                tag_id=tag_id,
                search=search,
                favorites_only=favorites_only,
                sort_by=sort_by,
                sort_order=sort_order,
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
            examples = self._db.get_examples("style", style_id)
            return {"status": "success", "style": style, "loras": loras, "tags": tags, "examples": examples}
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

    # ── Style examples ─────────────────────────────────────────

    def get_style_examples(self, style_id: str, **kwargs) -> dict[str, Any]:
        try:
            examples = self._db.get_examples("style", style_id)
            return {"status": "success", "examples": examples}
        except Exception as e:
            logger.error("Failed to get style examples: %s", e)
            return {"status": "error", "message": str(e)}

    def create_style_example(
        self,
        style_id: str,
        prompt: str,
        image_path: str | None = None,
        seed: int | None = None,
        width: int | None = None,
        height: int | None = None,
        steps: int | None = None,
        cfg_scale: float | None = None,
        **kwargs,
    ) -> dict[str, Any]:
        try:
            content = json.dumps({
                "prompt": prompt,
                "image_path": image_path,
                "seed": seed,
                "width": width,
                "height": height,
                "steps": steps,
                "cfg_scale": cfg_scale,
            })
            example = self._db.insert_example(
                id=str(uuid.uuid4()),
                entity_type="style",
                entity_id=style_id,
                example_type="prompt",
                content=content,
            )
            return {"status": "success", "example": example}
        except Exception as e:
            logger.error("Failed to create style example: %s", e)
            return {"status": "error", "message": str(e)}

    def delete_style_example(self, example_id: str, **kwargs) -> dict[str, Any]:
        try:
            self._db.delete_example(example_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to delete style example: %s", e)
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

    # ── CivitAI Import ─────────────────────────────────────────

    def browse_and_import_metadata(self, **kwargs) -> dict[str, Any]:
        """Open a native file dialog for .metadata.json files and import each as a style."""
        try:
            window = webview.windows[0]
            result = window.create_file_dialog(
                webview.FileDialog.OPEN,
                allow_multiple=True,
                file_types=("CivitAI metadata (*.metadata.json)",),
            )
            if not result:
                return {"status": "success", "styles": []}

            styles = []
            errors = []
            for filepath in result:
                res = self.import_civitai_metadata(file_path=str(filepath))
                if res["status"] == "success":
                    styles.append(res["style"])
                else:
                    errors.append(res["message"])

            if errors:
                logger.warning("Some metadata imports failed: %s", errors)

            return {"status": "success", "styles": styles, "errors": errors}
        except Exception as e:
            logger.error("Failed to browse metadata files: %s", e)
            return {"status": "error", "message": str(e)}

    def import_civitai_metadata(
        self,
        file_path: str | None = None,
        json_content: str | None = None,
        **kwargs,
    ) -> dict[str, Any]:
        """Import a CivitAI .metadata.json as a new style with associated LoRA and tags.

        Accepts either ``file_path`` (read from disk) or ``json_content``
        (raw JSON string, e.g. read by the frontend via FileReader).
        """
        try:
            if json_content:
                meta = json.loads(json_content)
            elif file_path:
                if not os.path.isfile(file_path):
                    return {"status": "error", "message": f"File not found: {file_path}"}
                with open(file_path, "r", encoding="utf-8") as f:
                    meta = json.load(f)
            else:
                return {"status": "error", "message": "Either file_path or json_content is required"}

            civitai = meta.get("civitai", {})
            civitai_model = civitai.get("model", {})

            # ── Extract fields ──
            style_name = meta.get("model_name") or meta.get("file_name", "Imported Style")
            trained_words = civitai.get("trainedWords", [])
            if trained_words:
                prompt_template = ", ".join(trained_words) + " {prompt}"
            else:
                prompt_template = "{prompt}"

            # Strip HTML from description
            raw_desc = civitai_model.get("description", "")
            description = re.sub(r"<[^>]+>", "", raw_desc).strip() if raw_desc else None

            thumbnail_path = meta.get("preview_url")
            lora_path = meta.get("file_path")
            lora_name = meta.get("file_name", "Unknown LoRA")
            base_model = meta.get("base_model")
            size_bytes = meta.get("size")
            tag_names = meta.get("tags", [])

            # ── LoRA: reuse or create ──
            lora = None
            if lora_path:
                lora = self._db.get_lora_by_path(lora_path)
                if not lora:
                    lora = self._db.insert_lora(
                        id=str(uuid.uuid4()),
                        name=os.path.splitext(lora_name)[0] if lora_name else "Unknown LoRA",
                        path=lora_path,
                        trigger_words=", ".join(trained_words) if trained_words else None,
                        base_model=base_model,
                        size_bytes=size_bytes,
                    )

            # ── Tags: reuse or create ──
            tag_ids: list[int] = []
            for tag_name in tag_names:
                tag_name = tag_name.strip()
                if not tag_name:
                    continue
                existing = self._db.get_tag_by_name(tag_name)
                if existing:
                    tag_ids.append(existing["id"])
                else:
                    new_tag = self._db.create_tag(name=tag_name, category="style")
                    tag_ids.append(new_tag["id"])

            # ── Style ──
            style_id = str(uuid.uuid4())
            style = self._db.insert_style(
                id=style_id,
                name=style_name,
                prompt_template=prompt_template,
                description=description,
                thumbnail_path=thumbnail_path,
            )

            # ── Associations ──
            if lora:
                self._db.set_style_loras(style_id, [{"lora_id": lora["id"], "strength": 1.0}])
            if tag_ids:
                self._db.set_style_tags(style_id, tag_ids)

            # ── Examples from images ──
            images = civitai.get("images", [])
            example_count = 0
            for img in images[:5]:  # Limit to first 5 examples
                img_meta = img.get("meta", {})
                if not img_meta.get("prompt"):
                    continue

                example_content = json.dumps({
                    "prompt": img_meta.get("prompt"),
                    "image_path": img.get("url"),  # CivitAI URL
                    "seed": img_meta.get("seed"),
                    "width": img.get("width"),  # At image level, not meta
                    "height": img.get("height"),  # At image level, not meta
                    "steps": img_meta.get("steps"),
                    "cfg_scale": img_meta.get("cfgScale"),
                })

                self._db.insert_example(
                    id=str(uuid.uuid4()),
                    entity_type="style",
                    entity_id=style_id,
                    example_type="prompt",
                    content=example_content,
                )
                example_count += 1

            logger.info(
                "Imported CivitAI metadata as style '%s' (id=%s) with %d examples",
                style_name, style_id, example_count
            )
            return {"status": "success", "style": style}

        except json.JSONDecodeError as e:
            logger.error("Invalid JSON in metadata file: %s", e)
            return {"status": "error", "message": f"Invalid JSON: {e}"}
        except Exception as e:
            logger.error("Failed to import CivitAI metadata: %s", e)
            return {"status": "error", "message": str(e)}
