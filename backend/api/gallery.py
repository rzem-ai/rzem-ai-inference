"""Gallery API mixin — image, folder, and tag operations for the frontend."""

from __future__ import annotations

import logging
import os
import shutil
from typing import Any

import webview

from backend.db.database import Database

logger = logging.getLogger(__name__)


class GalleryAPI:
    """pywebview js_api mixin for gallery operations (images, folders, tags)."""

    def __init__(self, db: Database) -> None:
        self._db = db

    # ── Images ───────────────────────────────────────────────────

    def get_gallery_images(
        self,
        limit: int = 50,
        offset: int = 0,
        folder_id: str | None = None,
        tag_id: int | None = None,
        search: str | None = None,
        favorites_only: bool = False,
        sort_by: str = "created_at",
        sort_order: str = "desc",
        **kwargs,
    ) -> dict[str, Any]:
        try:
            result = self._db.get_images(
                limit=limit,
                offset=offset,
                folder_id=folder_id,
                tag_id=tag_id,
                search=search,
                favorites_only=favorites_only,
                sort_by=sort_by,
                sort_order=sort_order,
            )
            return {"status": "success", **result}
        except Exception as e:
            logger.error("Failed to get gallery images: %s", e)
            return {"status": "error", "message": str(e)}

    def get_image(self, image_id: str) -> dict[str, Any]:
        try:
            image = self._db.get_image(image_id)
            if not image:
                return {"status": "error", "message": f"Image '{image_id}' not found"}
            return {"status": "success", "image": image}
        except Exception as e:
            logger.error("Failed to get image: %s", e)
            return {"status": "error", "message": str(e)}

    def toggle_favorite(self, image_id: str) -> dict[str, Any]:
        try:
            image = self._db.toggle_favorite(image_id)
            if not image:
                return {"status": "error", "message": f"Image '{image_id}' not found"}
            return {"status": "success", "image": image}
        except Exception as e:
            logger.error("Failed to toggle favorite: %s", e)
            return {"status": "error", "message": str(e)}

    def save_image_as(self, file_path: str, **kwargs) -> dict[str, Any]:
        """Open a native Save As dialog and copy the image to the chosen location."""
        try:
            if not os.path.isfile(file_path):
                return {"status": "error", "message": "Source file not found"}

            ext = os.path.splitext(file_path)[1] or ".png"
            basename = os.path.basename(file_path)
            window = webview.windows[0]
            result = window.create_file_dialog(
                webview.FileDialog.SAVE,
                save_filename=basename,
                file_types=(f"Image (*{ext})",),
            )
            if not result:
                return {"status": "success", "saved": False}

            dest = str(result) if isinstance(result, str) else str(result[0])
            if not dest.lower().endswith(ext.lower()):
                dest += ext
            shutil.copy2(file_path, dest)
            return {"status": "success", "saved": True, "path": dest}
        except Exception as e:
            logger.error("Failed to save image: %s", e)
            return {"status": "error", "message": str(e)}

    def batch_save_images(self, image_ids: list[str], **kwargs) -> dict[str, Any]:
        """Open a native folder picker and copy selected images to the chosen folder."""
        try:
            window = webview.windows[0]
            result = window.create_file_dialog(webview.FileDialog.FOLDER)
            if not result:
                return {"status": "success", "saved_count": 0}

            dest_folder = str(result) if isinstance(result, str) else str(result[0])
            saved = 0
            for image_id in image_ids:
                image = self._db.get_image(image_id)
                if not image:
                    continue
                file_path = image.get("file_path")
                if file_path and os.path.isfile(file_path):
                    shutil.copy2(file_path, dest_folder)
                    saved += 1

            return {"status": "success", "saved_count": saved, "folder": dest_folder}
        except Exception as e:
            logger.error("Failed to batch save images: %s", e)
            return {"status": "error", "message": str(e)}

    def delete_image(self, image_id: str) -> dict[str, Any]:
        try:
            image = self._db.get_image(image_id)
            if not image:
                return {"status": "error", "message": f"Image '{image_id}' not found"}

            # Remove file from disk
            file_path = image.get("file_path")
            if file_path and os.path.isfile(file_path):
                os.remove(file_path)
            thumb_path = image.get("thumbnail_path")
            if thumb_path and os.path.isfile(thumb_path):
                os.remove(thumb_path)

            self._db.delete_image(image_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to delete image: %s", e)
            return {"status": "error", "message": str(e)}

    # ── Folders ──────────────────────────────────────────────────

    def get_folders(self) -> dict[str, Any]:
        try:
            folders = self._db.get_folders()
            return {"status": "success", "folders": folders}
        except Exception as e:
            logger.error("Failed to get folders: %s", e)
            return {"status": "error", "message": str(e)}

    def create_folder(self, id: str, name: str, parent_id: str | None = None,
                      color: str | None = None, icon: str | None = None,
                      sort_order: int = 0, **kwargs) -> dict[str, Any]:
        try:
            folder = self._db.create_folder(
                id=id, name=name, parent_id=parent_id,
                color=color, icon=icon, sort_order=sort_order,
            )
            return {"status": "success", "folder": folder}
        except Exception as e:
            logger.error("Failed to create folder: %s", e)
            return {"status": "error", "message": str(e)}

    def update_folder(self, folder_id: str, **kwargs) -> dict[str, Any]:
        try:
            folder = self._db.update_folder(folder_id, **kwargs)
            if not folder:
                return {"status": "error", "message": f"Folder '{folder_id}' not found"}
            return {"status": "success", "folder": folder}
        except Exception as e:
            logger.error("Failed to update folder: %s", e)
            return {"status": "error", "message": str(e)}

    def delete_folder(self, folder_id: str) -> dict[str, Any]:
        try:
            self._db.delete_folder(folder_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to delete folder: %s", e)
            return {"status": "error", "message": str(e)}

    # ── Folder ↔ Image associations ──────────────────────────────

    def add_image_to_folder(self, image_id: str, folder_id: str) -> dict[str, Any]:
        try:
            self._db.add_image_to_folder(image_id, folder_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to add image to folder: %s", e)
            return {"status": "error", "message": str(e)}

    def remove_image_from_folder(self, image_id: str, folder_id: str) -> dict[str, Any]:
        try:
            self._db.remove_image_from_folder(image_id, folder_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to remove image from folder: %s", e)
            return {"status": "error", "message": str(e)}

    # ── Tags ─────────────────────────────────────────────────────

    def get_tags(self) -> dict[str, Any]:
        try:
            tags = self._db.get_tags()
            return {"status": "success", "tags": tags}
        except Exception as e:
            logger.error("Failed to get tags: %s", e)
            return {"status": "error", "message": str(e)}

    def create_tag(self, name: str, color: str | None = None,
                   category: str | None = None, **kwargs) -> dict[str, Any]:
        try:
            tag = self._db.create_tag(name=name, color=color, category=category)
            return {"status": "success", "tag": tag}
        except Exception as e:
            logger.error("Failed to create tag: %s", e)
            return {"status": "error", "message": str(e)}

    def update_tag(self, tag_id: int, **kwargs) -> dict[str, Any]:
        try:
            tag = self._db.update_tag(tag_id, **kwargs)
            if not tag:
                return {"status": "error", "message": f"Tag '{tag_id}' not found"}
            return {"status": "success", "tag": tag}
        except Exception as e:
            logger.error("Failed to update tag: %s", e)
            return {"status": "error", "message": str(e)}

    def delete_tag(self, tag_id: int) -> dict[str, Any]:
        try:
            self._db.delete_tag(tag_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to delete tag: %s", e)
            return {"status": "error", "message": str(e)}

    # ── Tag ↔ Image associations ─────────────────────────────────

    def add_tag_to_image(self, image_id: str, tag_id: int) -> dict[str, Any]:
        try:
            self._db.add_tag_to_image(image_id, tag_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to add tag to image: %s", e)
            return {"status": "error", "message": str(e)}

    def remove_tag_from_image(self, image_id: str, tag_id: int) -> dict[str, Any]:
        try:
            self._db.remove_tag_from_image(image_id, tag_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to remove tag from image: %s", e)
            return {"status": "error", "message": str(e)}

    def get_image_tags(self, image_id: str) -> dict[str, Any]:
        try:
            tags = self._db.get_image_tags(image_id)
            return {"status": "success", "tags": tags}
        except Exception as e:
            logger.error("Failed to get image tags: %s", e)
            return {"status": "error", "message": str(e)}

    # ── Settings ─────────────────────────────────────────────────

    def get_setting(self, key: str) -> dict[str, Any]:
        try:
            value = self._db.get_setting(key)
            return {"status": "success", "value": value}
        except Exception as e:
            logger.error("Failed to get setting: %s", e)
            return {"status": "error", "message": str(e)}

    def set_setting(self, key: str, value: str) -> dict[str, Any]:
        try:
            self._db.set_setting(key, value)
            if key == "HF_API_KEY":
                if value:
                    os.environ["HF_TOKEN"] = value
                else:
                    os.environ.pop("HF_TOKEN", None)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to set setting: %s", e)
            return {"status": "error", "message": str(e)}
