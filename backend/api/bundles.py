"""Bundles API mixin — CRUD operations for model bundles (backed by SQLite)."""

from __future__ import annotations

import logging
from typing import Any

from backend.db.database import Database

logger = logging.getLogger(__name__)


class BundlesAPI:
    """pywebview js_api mixin for model bundle operations."""

    def __init__(self, db: Database) -> None:
        self._bundles_db = db

    def get_bundle_types(self, **kwargs) -> dict[str, Any]:
        try:
            types = self._bundles_db.get_bundle_types()
            return {"status": "success", "bundle_types": types}
        except Exception as e:
            logger.error("Failed to get bundle types: %s", e)
            return {"status": "error", "message": str(e)}

    def get_bundles(self, include_hidden: bool = False, **kwargs) -> dict[str, Any]:
        try:
            bundles = self._bundles_db.get_bundles(include_hidden=include_hidden)
            # Hide cloud bundles if FAL_KEY is not configured
            fal_key = self._bundles_db.get_setting("FAL_KEY")
            if not fal_key:
                bundles = [b for b in bundles if b.get("source", "local") != "cloud"]
            # Filter by generation mode preference
            gen_mode = self._bundles_db.get_setting("GENERATION_MODE")
            if gen_mode == "local":
                bundles = [b for b in bundles if b.get("source", "local") != "cloud"]
            elif gen_mode == "cloud":
                bundles = [b for b in bundles if b.get("source", "local") != "local"]
            return {"status": "success", "bundles": bundles}
        except Exception as e:
            logger.error("Failed to get bundles: %s", e)
            return {"status": "error", "message": str(e)}

    def get_bundle(self, bundle_id: str, **kwargs) -> dict[str, Any]:
        try:
            bundle = self._bundles_db.get_bundle(bundle_id)
            if not bundle:
                return {"status": "error", "message": f"Bundle '{bundle_id}' not found"}
            return {"status": "success", "bundle": bundle}
        except Exception as e:
            logger.error("Failed to get bundle: %s", e)
            return {"status": "error", "message": str(e)}

    def get_bundles_for_type(self, transformer_type: str, **kwargs) -> dict[str, Any]:
        try:
            all_bundles = self._bundles_db.get_bundles()
            filtered = [b for b in all_bundles if b["transformer_type"] == transformer_type]
            return {"status": "success", "bundles": filtered}
        except Exception as e:
            logger.error("Failed to get bundles for type: %s", e)
            return {"status": "error", "message": str(e)}

    def create_bundle(self, **kwargs) -> dict[str, Any]:
        try:
            bundle = self._bundles_db.insert_bundle(**kwargs)
            return {"status": "success", "bundle": bundle}
        except Exception as e:
            logger.error("Failed to create bundle: %s", e)
            return {"status": "error", "message": str(e)}

    def update_bundle(self, bundle_id: str, **kwargs) -> dict[str, Any]:
        try:
            bundle = self._bundles_db.update_bundle(bundle_id, **kwargs)
            return {"status": "success", "bundle": bundle}
        except Exception as e:
            logger.error("Failed to update bundle: %s", e)
            return {"status": "error", "message": str(e)}

    def delete_bundle(self, bundle_id: str, **kwargs) -> dict[str, Any]:
        try:
            self._bundles_db.delete_bundle(bundle_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to delete bundle: %s", e)
            return {"status": "error", "message": str(e)}

    def toggle_bundle_favorite(self, bundle_id: str, **kwargs) -> dict[str, Any]:
        try:
            bundle = self._bundles_db.toggle_bundle_favorite(bundle_id)
            if not bundle:
                return {"status": "error", "message": f"Bundle '{bundle_id}' not found"}
            return {"status": "success", "bundle": bundle}
        except Exception as e:
            logger.error("Failed to toggle bundle favorite: %s", e)
            return {"status": "error", "message": str(e)}

    def toggle_bundle_hidden(self, bundle_id: str, **kwargs) -> dict[str, Any]:
        try:
            bundle = self._bundles_db.toggle_bundle_hidden(bundle_id)
            if not bundle:
                return {"status": "error", "message": f"Bundle '{bundle_id}' not found"}
            return {"status": "success", "bundle": bundle}
        except Exception as e:
            logger.error("Failed to toggle bundle hidden: %s", e)
            return {"status": "error", "message": str(e)}

    def reset_default_bundles(self, **kwargs) -> dict[str, Any]:
        try:
            from backend.bundles import get_default_bundles_as_dicts
            # Delete all existing bundles and re-seed
            for b in self._bundles_db.get_bundles():
                self._bundles_db.delete_bundle(b["id"])
            self._bundles_db.seed_bundles(get_default_bundles_as_dicts())
            bundles = self._bundles_db.get_bundles()
            return {"status": "success", "bundles": bundles}
        except Exception as e:
            logger.error("Failed to reset bundles: %s", e)
            return {"status": "error", "message": str(e)}
