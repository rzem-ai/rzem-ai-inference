"""Gallery image API methods."""

from typing import Optional, List, Dict, Any
from loguru import logger

from api.base import ApiBase


class GalleryApiMixin(ApiBase):

    def get_all_images(self, limit: Optional[int] = None) -> List[Dict[str, Any]]:
        """Get all images from gallery."""
        try:
            if not self._app_state.db:
                return []
            images = self._run_async(self._app_state.db.get_all_images(limit))
            return images
        except Exception as e:
            logger.debug(f"No images in gallery (database may be empty): {e}")
            return []

    def get_image_by_id(self, image_id: str) -> Optional[Dict[str, Any]]:
        """Get image by ID."""
        try:
            if not self._app_state.db:
                return None
            image = self._run_async(self._app_state.db.get_image_by_id(image_id))
            return image
        except Exception as e:
            logger.error(f"Failed to get image {image_id}: {e}")
            return None

    def delete_image(self, image_id: str) -> Dict[str, Any]:
        """Delete an image."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_image(image_id))
            if success:
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Image not found"}
        except Exception as e:
            logger.error(f"Failed to delete image {image_id}: {e}")
            return {"status": "error", "message": str(e)}

    def delete_gallery_image(self, image_id: str) -> Dict[str, Any]:
        """Delete a gallery image (alias for delete_image)."""
        return self.delete_image(image_id)

    def search_gallery_images(
        self,
        query: Optional[str] = None,
        tags: Optional[List[str]] = None,
        folder_id: Optional[str] = None,
        favorites_only: bool = False,
        limit: Optional[int] = 100,
    ) -> List[Dict[str, Any]]:
        """Search gallery images."""
        try:
            if not self._app_state.db:
                return []
            images = self._run_async(self._app_state.db.search_gallery_images(
                query=query,
                tags=tags,
                folder_id=folder_id,
                favorites_only=favorites_only,
                limit=limit or 100,
            ))
            return images
        except Exception as e:
            logger.error(f"Failed to search gallery images: {e}")
            return []

    def toggle_favorite(self, image_id: str) -> Dict[str, Any]:
        """Toggle favorite status of an image."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.toggle_favorite(image_id))
            if success:
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Image not found"}
        except Exception as e:
            logger.error(f"Failed to toggle favorite {image_id}: {e}")
            return {"status": "error", "message": str(e)}

    def get_gallery_images(self, limit: Optional[int] = None) -> List[Dict[str, Any]]:
        """Get gallery images (alias for get_all_images)."""
        return self.get_all_images(limit)
