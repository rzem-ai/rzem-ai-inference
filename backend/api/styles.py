"""Styles API mixin — style, style-lora, and style-tag operations for the frontend."""

from __future__ import annotations

import json
import base64
import logging
import mimetypes
import os
import re
import uuid
from pathlib import Path
from typing import TYPE_CHECKING, Any

import webview

from backend.db.database import Database
from backend.services.image_utils import store_style_image

if TYPE_CHECKING:
    from backend.services.chat_service import ChatService

logger = logging.getLogger(__name__)

_LORA_EXTENSIONS = ("safetensors", "ckpt", "pt")
_IMAGE_EXTENSIONS = ("png", "jpg", "jpeg", "webp", "bmp")


class StylesAPI:
    """pywebview js_api mixin for style management."""

    def __init__(self, db: Database, styles_dir: Path | None = None, chat_service: ChatService | None = None) -> None:
        self._db = db
        self._styles_dir = styles_dir or Path.home() / ".rzem-ai" / "styles"
        self._chat = chat_service

    def get_styles_dir(self, **kwargs) -> dict[str, Any]:
        """Return the styles directory path."""
        return {"status": "success", "path": str(self._styles_dir)}

    def list_filter_thumbnails(self, **kwargs) -> dict[str, Any]:
        """Scan the thumbnails directory and return filter image names that have thumbnails."""
        try:
            thumbnails_dir = self._styles_dir / "thumbnails"
            if not thumbnails_dir.is_dir():
                return {"status": "success", "thumbnails": []}

            stems: set[str] = set()
            for f in thumbnails_dir.iterdir():
                if not f.is_file():
                    continue
                stem = f.stem
                if stem.endswith("_cover"):
                    stems.add(stem[:-6])
                elif f.suffix.lower() in (".png", ".jpg", ".jpeg", ".webp"):
                    stems.add(stem)

            thumbnails = sorted(f"{s}.png" for s in stems)
            return {"status": "success", "thumbnails": thumbnails}
        except Exception as e:
            logger.error("Failed to list filter thumbnails: %s", e)
            return {"status": "error", "message": str(e)}

    def get_filter_thumbnail(self, image_name: str, **kwargs) -> dict[str, Any]:
        """Load a filter thumbnail as a base64 data URL (prefers cover WebP)."""
        try:
            thumbnails_dir = self._styles_dir / "thumbnails"
            stem = Path(image_name).stem

            # Prefer the half-res cover (smaller transfer)
            cover = thumbnails_dir / f"{stem}_cover.webp"
            if cover.is_file():
                data = cover.read_bytes()
                b64 = base64.b64encode(data).decode("ascii")
                return {"status": "success", "data_url": f"data:image/webp;base64,{b64}"}

            # Fall back to full PNG
            png = thumbnails_dir / f"{stem}.png"
            if png.is_file():
                data = png.read_bytes()
                b64 = base64.b64encode(data).decode("ascii")
                return {"status": "success", "data_url": f"data:image/png;base64,{b64}"}

            return {"status": "error", "message": f"Thumbnail not found: {image_name}"}
        except Exception as e:
            logger.error("Failed to get filter thumbnail: %s", e)
            return {"status": "error", "message": str(e)}

    # ── Styles ────────────────────────────────────────────────

    def get_styles(
        self,
        category: str | None = None,
        tag_id: int | None = None,
        search: str | None = None,
        favorites_only: bool = False,
        bundle_type: str | None = None,
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
                bundle_type=bundle_type,
                sort_by=sort_by,
                sort_order=sort_order,
            )
            return {"status": "success", "styles": styles}
        except Exception as e:
            logger.error("Failed to get styles: %s", e)
            return {"status": "error", "message": str(e)}

    def get_style_bundle_type_counts(self, **kwargs) -> dict[str, Any]:
        try:
            counts = self._db.get_style_bundle_type_counts()
            return {"status": "success", "counts": counts}
        except Exception as e:
            logger.error("Failed to get style bundle type counts: %s", e)
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
        bundle_id: str | None = None,
        **kwargs,
    ) -> dict[str, Any]:
        try:
            style = self._db.insert_style(
                id=id, name=name, prompt_template=prompt_template,
                description=description, negative_prompt=negative_prompt,
                category=category, thumbnail_path=thumbnail_path,
                bundle_id=bundle_id,
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
            local_image_path = None
            local_thumb_path = None
            if image_path:
                paths = store_style_image(image_path, self._styles_dir, style_id)
                if paths:
                    local_image_path = paths[0]  # Full resolution
                    local_thumb_path = paths[1]  # Thumbnail

            content = json.dumps({
                "prompt": prompt,
                "image_path": local_image_path,
                "thumbnail_path": local_thumb_path,
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

    def import_style_image(self, source_path: str, style_id: str = "", **kwargs) -> dict[str, Any]:
        """Copy a local image into the styles directory, returning stored paths."""
        try:
            paths = store_style_image(source_path, self._styles_dir, style_id)
            if not paths:
                return {"status": "error", "message": "Failed to process image"}
            full_path, thumb_path = paths
            return {"status": "success", "path": full_path, "thumbnail_path": thumb_path}
        except Exception as e:
            logger.error("Failed to import style image: %s", e)
            return {"status": "error", "message": str(e)}

    def browse_image_file(self, style_id: str = "", **kwargs) -> dict[str, Any]:
        """Open a native file dialog, copy the image to styles dir, return local paths."""
        try:
            window = webview.windows[0]
            file_filter = "Images (" + ";".join(f"*.{ext}" for ext in _IMAGE_EXTENSIONS) + ")"
            result = window.create_file_dialog(
                webview.FileDialog.OPEN,
                allow_multiple=False,
                file_types=(file_filter,),
            )
            if not result:
                return {"status": "success", "path": None, "thumbnail_path": None}

            source = str(result[0]) if isinstance(result, (list, tuple)) else str(result)
            paths = store_style_image(source, self._styles_dir, style_id)
            if not paths:
                return {"status": "error", "message": "Failed to process image"}

            full_path, thumb_path = paths
            return {"status": "success", "path": full_path, "thumbnail_path": thumb_path}
        except Exception as e:
            logger.error("Failed to browse image file: %s", e)
            return {"status": "error", "message": str(e)}

    # ── CivitAI Import ─────────────────────────────────────────

    def browse_metadata_files(self, **kwargs) -> dict[str, Any]:
        """Open a native file dialog for .metadata.json files and return the selected paths."""
        try:
            window = webview.windows[0]
            result = window.create_file_dialog(
                webview.FileDialog.OPEN,
                allow_multiple=True,
                file_types=("CivitAI metadata (*.metadata.json)",),
            )
            if not result:
                return {"status": "success", "paths": []}
            paths = [str(p) for p in result]
            return {"status": "success", "paths": paths}
        except Exception as e:
            logger.error("Failed to browse metadata files: %s", e)
            return {"status": "error", "message": str(e)}

    def browse_and_import_metadata(self, **kwargs) -> dict[str, Any]:
        """Open a native file dialog for .metadata.json files and import each as a style."""
        try:
            res = self.browse_metadata_files()
            if res["status"] != "success" or not res.get("paths"):
                return {"status": "success", "styles": []}

            styles = []
            errors = []
            for filepath in res["paths"]:
                r = self.import_civitai_metadata(file_path=filepath)
                if r["status"] == "success":
                    styles.append(r["style"])
                else:
                    errors.append(r["message"])

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

            civitai = meta.get("civitai") or {}
            civitai_model = civitai.get("model") or {}

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

            style_id = str(uuid.uuid4())

            # Download thumbnail to local styles dir
            thumbnail_path = None
            full_image_path = None
            preview_url = meta.get("preview_url")
            if preview_url:
                paths = store_style_image(preview_url, self._styles_dir, style_id)
                if paths:
                    full_image_path = paths[0]  # Full resolution for AI analysis
                    thumbnail_path = paths[1]   # Thumbnail WebP for style card

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
                img_meta = img.get("meta") or {}
                if not img_meta.get("prompt"):
                    continue

                # Download example image to local styles dir
                local_example_path = None
                local_example_thumb = None
                example_url = img.get("url")
                if example_url:
                    ex_paths = store_style_image(example_url, self._styles_dir, style_id)
                    if ex_paths:
                        local_example_path = ex_paths[0]  # Full resolution
                        local_example_thumb = ex_paths[1]  # Thumbnail

                example_content = json.dumps({
                    "prompt": img_meta.get("prompt"),
                    "image_path": local_example_path,
                    "thumbnail_path": local_example_thumb,
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

            # If no examples had prompts, try AI-generated template
            if example_count == 0 and full_image_path:
                ai_template = self._generate_prompt_template_with_ai(
                    image_path=full_image_path,
                    trained_words=trained_words,
                    style_name=style_name,
                )
                if ai_template:
                    style = self._db.update_style(style_id, prompt_template=ai_template)

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

    # ── AI Prompt Template Generation ──────────────────────────

    def _generate_prompt_template_with_ai(
        self,
        image_path: str,
        trained_words: list[str],
        style_name: str,
    ) -> str | None:
        """Use the active AI provider to generate a prompt template from a preview image.

        Requires a provider that supports vision (image inputs). Falls back to
        None if the provider doesn't support vision or isn't configured.
        """
        if not self._chat or not self._chat.is_configured:
            return None

        if not self._chat.active_provider.supports_vision():
            logger.debug("Active provider '%s' doesn't support vision, skipping AI template", self._chat.active_provider_name)
            return None

        path = Path(image_path)
        if not path.is_file():
            return None

        try:
            mime = mimetypes.guess_type(str(path))[0] or "image/png"
            image_data = base64.standard_b64encode(path.read_bytes()).decode("ascii")
            words_str = ", ".join(trained_words) if trained_words else "(none)"

            messages = [{
                "role": "user",
                "content": [
                    {
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": mime,
                            "data": image_data,
                        },
                    },
                    {
                        "type": "text",
                        "text": (
                            f"You are analyzing a preview image from an AI LoRA model called \"{style_name}\".\n"
                            f"Trigger/trained words: {words_str}\n\n"
                            "Create a prompt template that captures this visual style. Rules:\n"
                            "- MUST include {prompt} exactly once as a placeholder for the user's subject\n"
                            "- MUST include the trigger words\n"
                            "- Describe key visual characteristics (art style, lighting, color palette, mood)\n"
                            "- Single line, under 60 words\n"
                            "- Written as a generation prompt, not a description\n\n"
                            "Output ONLY the template text, nothing else."
                        ),
                    },
                ],
            }]

            template = self._chat.complete(messages, max_tokens=256).strip()
            if "{prompt}" not in template:
                template += " {prompt}"

            logger.info("AI-generated prompt template for '%s': %s", style_name, template[:80])
            return template

        except Exception as e:
            logger.warning("AI prompt template generation failed for '%s': %s", style_name, e)
            return None

    # ── AI Style Review ─────────────────────────────────────────

    def review_styles(self, style_ids: list | None = None, **kwargs) -> dict[str, Any]:
        """Send selected styles to the AI provider for name and tag improvement suggestions."""
        try:
            if not self._chat or not self._chat.is_configured:
                provider = self._chat.active_provider_name if self._chat else "AI"
                return {"status": "error", "message": f"{provider.capitalize()} API key not configured. Set it in Settings > AI."}

            if not style_ids:
                return {"status": "error", "message": "No styles selected for review."}

            all_styles = self._db.get_styles()
            id_set = set(style_ids)
            styles = [s for s in all_styles if s["id"] in id_set]
            if not styles:
                return {"status": "error", "message": "No matching styles found."}

            # Build compact style data with current tags (minimal keys to save tokens)
            style_entries = []
            for s in styles:
                tags = self._db.get_style_tags(s["id"])
                entry: dict[str, Any] = {
                    "id": s["id"],
                    "name": s["name"],
                    "prompt": s["prompt_template"],
                }
                if tags:
                    entry["tags"] = [t["name"] for t in tags]
                style_entries.append(entry)

            # Gather existing style tags for reuse
            all_tags = self._db.get_tags()
            existing_tag_names = [t["name"] for t in all_tags if t.get("category") == "style"]

            styles_json = json.dumps(style_entries, separators=(",", ":"))
            tags_json = json.dumps(existing_tag_names)

            messages = [{
                "role": "user",
                "content": (
                    "Review these AI image generation styles. For each, suggest a clean name "
                    "(2-5 words, no file extensions, title case) and 2-5 descriptive tags.\n\n"
                    f"Existing tags (reuse when fitting): {tags_json}\n\n"
                    f"Styles:\n{styles_json}\n\n"
                    "Return a JSON array. Each element:\n"
                    '{"id":"...","suggested_name":"...","name_changed":bool,"suggested_tags":["..."],"reason":"short"}\n\n'
                    "Keep reasons under 10 words. Output ONLY valid JSON array, no markdown fences."
                ),
            }]

            raw = self._chat.complete(messages, max_tokens=16384).strip()

            # Strip markdown fences if present
            if raw.startswith("```"):
                raw = raw.split("\n", 1)[1] if "\n" in raw else raw[3:]
                if raw.endswith("```"):
                    raw = raw[:-3]
                raw = raw.strip()

            suggestions = json.loads(raw)

            if not isinstance(suggestions, list):
                return {"status": "error", "message": "AI returned unexpected format. Please try again."}

            # Validate and filter to known style IDs
            known_ids = {s["id"] for s in styles}
            valid = []
            for s in suggestions:
                if not isinstance(s, dict):
                    continue
                if s.get("id") not in known_ids:
                    continue
                if not all(k in s for k in ("id", "suggested_name", "name_changed", "suggested_tags", "reason")):
                    continue
                valid.append({
                    "id": s["id"],
                    "suggested_name": str(s["suggested_name"]),
                    "name_changed": bool(s["name_changed"]),
                    "suggested_tags": [str(t) for t in s.get("suggested_tags", [])],
                    "reason": str(s["reason"]),
                })

            logger.info("AI style review: %d styles reviewed, %d suggestions", len(styles), len(valid))
            return {"status": "success", "suggestions": valid}

        except json.JSONDecodeError as e:
            logger.error("Failed to parse AI review response as JSON: %s", e)
            return {"status": "error", "message": "AI returned invalid JSON. Please try again."}
        except Exception as e:
            logger.error("Style review failed: %s", e)
            return {"status": "error", "message": str(e)}

    def apply_style_review(self, changes: list | None = None, **kwargs) -> dict[str, Any]:
        """Apply user-accepted style review changes (name and/or tag updates)."""
        try:
            if not changes:
                return {"status": "error", "message": "No changes to apply."}

            applied = 0
            for change in changes:
                style_id = change.get("style_id")
                if not style_id:
                    continue

                # Apply name change
                name = change.get("name")
                if name:
                    self._db.update_style(style_id, name=name)

                # Apply tag changes
                tags = change.get("tags")
                if tags and isinstance(tags, list):
                    tag_ids: list[int] = []
                    for tag_name in tags:
                        tag_name = str(tag_name).strip()
                        if not tag_name:
                            continue
                        existing = self._db.get_tag_by_name(tag_name)
                        if existing:
                            tag_ids.append(existing["id"])
                        else:
                            new_tag = self._db.create_tag(name=tag_name, category="style")
                            tag_ids.append(new_tag["id"])
                    self._db.set_style_tags(style_id, tag_ids)

                applied += 1

            logger.info("Applied %d style review changes", applied)
            return {"status": "success", "applied": applied}

        except Exception as e:
            logger.error("Failed to apply style review: %s", e)
            return {"status": "error", "message": str(e)}

    # ── Style Builder ─────────────────────────────────────────

    def generate_style_from_filters(self, filters: list[str], **kwargs) -> dict[str, Any]:
        """Use the active AI provider to generate a complete style from selected filter prompts."""
        try:
            if not self._chat or not self._chat.is_configured:
                provider = self._chat.active_provider_name if self._chat else "AI"
                return {"status": "error", "message": f"{provider.capitalize()} API key not configured. Set it in Settings > AI."}

            if not filters:
                return {"status": "error", "message": "No filters selected"}

            filters_str = ", ".join(filters)

            messages = [{
                "role": "user",
                "content": (
                    "You are an AI image generation style designer. Given a set of visual style filters, "
                    "create a cohesive generation style that combines them.\n\n"
                    f"Selected filters: {filters_str}\n\n"
                    "Return a JSON object with exactly these fields:\n"
                    '- "name": A short, creative name for this style (2-4 words)\n'
                    '- "description": A 1-2 sentence description of the visual style\n'
                    '- "prompt_template": A generation prompt template that captures the combined style. '
                    "MUST include {prompt} exactly once as a placeholder for the user's subject. "
                    "Should be descriptive, under 80 words, written as a generation prompt.\n"
                    '- "negative_prompt": Comma-separated terms to avoid (things that would clash with this style)\n'
                    '- "category": The best fitting category from: Photography, Illustration, Painting, Digital Art, '
                    "Architecture, Fashion, Character Design, Mixed Media\n\n"
                    "Output ONLY valid JSON, no markdown fences, no extra text."
                ),
            }]

            raw = self._chat.complete(messages).strip()

            # Strip markdown fences if present
            if raw.startswith("```"):
                raw = raw.split("\n", 1)[1] if "\n" in raw else raw[3:]
                if raw.endswith("```"):
                    raw = raw[:-3]
                raw = raw.strip()

            style_data = json.loads(raw)

            # Validate required fields
            required = ["name", "description", "prompt_template", "negative_prompt", "category"]
            for field in required:
                if field not in style_data:
                    return {"status": "error", "message": f"AI response missing '{field}' field"}

            # Ensure {prompt} placeholder exists
            if "{prompt}" not in style_data["prompt_template"]:
                style_data["prompt_template"] += " {prompt}"

            logger.info("AI-generated style from %d filters: %s", len(filters), style_data.get("name", "?"))
            return {"status": "success", "style_data": style_data}

        except json.JSONDecodeError as e:
            logger.error("Failed to parse AI response as JSON: %s", e)
            return {"status": "error", "message": "AI returned invalid JSON. Please try again."}
        except Exception as e:
            logger.error("Style generation from filters failed: %s", e)
            return {"status": "error", "message": str(e)}
