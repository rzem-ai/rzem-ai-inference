"""Bundles API mixin — CRUD operations for model bundles."""

from __future__ import annotations

import logging
from typing import Any

from backend.bundles import BundleStore
from backend.db.database import Database

logger = logging.getLogger(__name__)


class BundlesAPI:
    """pywebview js_api mixin for model bundle operations."""

    def __init__(self, bundle_store: BundleStore, db: Database) -> None:
        self._bundle_store = bundle_store
        self._bundles_db = db

    def get_bundles(self) -> dict[str, Any]:
        try:
            bundles = self._bundle_store.get_all()
            # Hide cloud bundles if FAL_KEY is not configured
            fal_key = self._bundles_db.get_setting("FAL_KEY")
            if not fal_key:
                bundles = [b for b in bundles if b.get("source", "local") != "cloud"]
            return {"status": "success", "bundles": bundles}
        except Exception as e:
            logger.error("Failed to get bundles: %s", e)
            return {"status": "error", "message": str(e)}

    def get_bundle(self, bundle_id: str) -> dict[str, Any]:
        try:
            bundle = self._bundle_store.get_by_id(bundle_id)
            if not bundle:
                return {"status": "error", "message": f"Bundle '{bundle_id}' not found"}
            return {"status": "success", "bundle": bundle}
        except Exception as e:
            logger.error("Failed to get bundle: %s", e)
            return {"status": "error", "message": str(e)}

    def get_bundles_for_type(self, transformer_type: str) -> dict[str, Any]:
        try:
            return {"status": "success", "bundles": self._bundle_store.get_by_type(transformer_type)}
        except Exception as e:
            logger.error("Failed to get bundles for type: %s", e)
            return {"status": "error", "message": str(e)}

    def create_bundle(self, **kwargs) -> dict[str, Any]:
        try:
            bundle = self._bundle_store.add(kwargs)
            return {"status": "success", "bundle": bundle}
        except Exception as e:
            logger.error("Failed to create bundle: %s", e)
            return {"status": "error", "message": str(e)}

    def update_bundle(self, bundle_id: str, **kwargs) -> dict[str, Any]:
        try:
            bundle = self._bundle_store.update(bundle_id, kwargs)
            return {"status": "success", "bundle": bundle}
        except Exception as e:
            logger.error("Failed to update bundle: %s", e)
            return {"status": "error", "message": str(e)}

    def delete_bundle(self, bundle_id: str) -> dict[str, Any]:
        try:
            self._bundle_store.delete(bundle_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to delete bundle: %s", e)
            return {"status": "error", "message": str(e)}

    def reset_default_bundles(self) -> dict[str, Any]:
        try:
            bundles = self._bundle_store.reset_defaults()
            return {"status": "success", "bundles": bundles}
        except Exception as e:
            logger.error("Failed to reset bundles: %s", e)
            return {"status": "error", "message": str(e)}
