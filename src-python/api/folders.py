"""Folder management API methods."""

from typing import Optional, List, Dict, Any
from loguru import logger

from api.base import ApiBase


class FolderApiMixin(ApiBase):

    def get_folder_tree(self) -> List[Dict[str, Any]]:
        """Get folder tree."""
        try:
            if not self._app_state.db:
                return []
            tree = self._run_async(self._app_state.db.get_folder_tree())
            return tree
        except Exception as e:
            logger.error(f"Failed to get folder tree: {e}")
            return []

    def create_folder(
        self,
        name: str,
        parent_id: Optional[str] = None,
        color: Optional[str] = None,
        icon: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Create a folder."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            folder_data = self._run_async(self._app_state.db.create_folder(
                name=name,
                parent_id=parent_id,
                color=color,
                icon=icon,
            ))
            logger.info(f"Folder created: {folder_data['id']}")
            return {"status": "success", "folder": folder_data}
        except Exception as e:
            logger.error(f"Failed to create folder: {e}")
            return {"status": "error", "message": str(e)}

    def update_folder(
        self,
        id: str,
        name: Optional[str] = None,
        color: Optional[str] = None,
        icon: Optional[str] = None,
    ) -> Dict[str, str]:
        """Update a folder."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.update_folder(
                folder_id=id,
                name=name,
                color=color,
                icon=icon,
            ))
            logger.info(f"Folder updated: {id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to update folder: {e}")
            return {"status": "error", "message": str(e)}

    def delete_folder(self, id: str) -> Dict[str, str]:
        """Delete a folder."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_folder(id))
            if success:
                logger.info(f"Folder deleted: {id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Folder not found"}
        except Exception as e:
            logger.error(f"Failed to delete folder: {e}")
            return {"status": "error", "message": str(e)}

    def move_folder(self, id: str, new_parent_id: Optional[str] = None) -> Dict[str, str]:
        """Move folder to a new parent."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.move_folder(id, new_parent_id))
            logger.info(f"Folder moved: {id} -> {new_parent_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to move folder: {e}")
            return {"status": "error", "message": str(e)}

    def reorder_folders(self, folder_ids: List[str]) -> Dict[str, str]:
        """Reorder folders within the same parent."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.reorder_folders(folder_ids))
            logger.info(f"Folders reordered: {len(folder_ids)} folders")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to reorder folders: {e}")
            return {"status": "error", "message": str(e)}

    def add_images_to_folder(self, image_ids: List[str], folder_id: str) -> Dict[str, str]:
        """Add images to a folder."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.add_images_to_folder(image_ids, folder_id))
            logger.info(f"Added {len(image_ids)} images to folder {folder_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to add images to folder: {e}")
            return {"status": "error", "message": str(e)}

    def remove_images_from_folder(self, image_ids: List[str], folder_id: str) -> Dict[str, str]:
        """Remove images from a folder."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            self._run_async(self._app_state.db.remove_images_from_folder(image_ids, folder_id))
            logger.info(f"Removed {len(image_ids)} images from folder {folder_id}")
            return {"status": "success"}
        except Exception as e:
            logger.error(f"Failed to remove images from folder: {e}")
            return {"status": "error", "message": str(e)}

    def get_folder_images(self, folder_id: str, limit: Optional[int] = 100) -> List[Dict[str, Any]]:
        """Get images in a specific folder."""
        try:
            if not self._app_state.db:
                return []
            images = self._run_async(self._app_state.db.get_folder_images(folder_id, limit or 100))
            return images
        except Exception as e:
            logger.error(f"Failed to get folder images: {e}")
            return []

    def get_uncategorized_images(self, limit: Optional[int] = 100) -> List[Dict[str, Any]]:
        """Get images not in any folder."""
        try:
            if not self._app_state.db:
                return []
            images = self._run_async(self._app_state.db.get_uncategorized_images(limit or 100))
            return images
        except Exception as e:
            logger.error(f"Failed to get uncategorized images: {e}")
            return []
