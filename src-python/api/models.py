"""Model components, bundles, and LoRA management API methods."""

import os
from pathlib import Path
from typing import List, Dict, Any, Optional
from loguru import logger

from api.base import ApiBase
from models.scanner import scan_hf_cache, scan_directory, filter_sharded_components, DiscoveredComponent
from models.converter import convert_comfyui_to_native, is_comfyui_format
import events


class ModelApiMixin(ApiBase):

    # ==================== Model Components ====================

    def get_all_models(self) -> List[Dict[str, Any]]:
        """Get all model components."""
        try:
            if not self._app_state.db:
                return []

            models = self._run_async(self._app_state.db.get_all_model_components())
            logger.debug(f"Retrieved {len(models)} model components")
            return models
        except Exception as e:
            logger.error(f"Failed to get models: {e}")
            return []

    def update_model(
        self,
        model_id: str,
        display_name: Optional[str] = None,
        description: Optional[str] = None,
        **kwargs,
    ) -> Dict[str, str]:
        """Update model details."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            updates = {}
            if display_name is not None:
                updates["name"] = display_name
            if description is not None:
                updates["description"] = description

            if not updates:
                return {"status": "error", "message": "No updates provided"}

            success = self._run_async(
                self._app_state.db.update_model_component(model_id, updates)
            )
            if success:
                logger.info(f"Model updated: {model_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Model not found or no updates"}
        except Exception as e:
            logger.error(f"Failed to update model: {e}")
            return {"status": "error", "message": str(e)}

    def add_model_tag(self, model_id: str, tag: str) -> Dict[str, str]:
        """Add tag to model."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.add_model_tag(model_id, tag))
            if success:
                logger.info(f"Tag added to model: {model_id} -> {tag}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Failed to add tag"}
        except Exception as e:
            logger.error(f"Failed to add model tag: {e}")
            return {"status": "error", "message": str(e)}

    def remove_model_tag(self, model_id: str, tag: str) -> Dict[str, str]:
        """Remove tag from model."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(
                self._app_state.db.remove_model_tag(model_id, tag)
            )
            if success:
                logger.info(f"Tag removed from model: {model_id} -> {tag}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Tag not found"}
        except Exception as e:
            logger.error(f"Failed to remove model tag: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Examples ====================

    def add_example(
        self, entity_type: str, entity_id: str, example_type: str, content: str
    ) -> Dict[str, Any]:
        """Add example to model/bundle.

        Args:
            entity_type: "model" or "bundle"
            entity_id: The model or bundle ID
            example_type: "prompt" or "image"
            content: The example content (prompt text or image ID)
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            if entity_type not in ("model", "bundle"):
                return {"status": "error", "message": "entity_type must be 'model' or 'bundle'"}
            if example_type not in ("prompt", "image"):
                return {"status": "error", "message": "example_type must be 'prompt' or 'image'"}

            example_id = self._run_async(
                self._app_state.db.add_example(entity_type, entity_id, example_type, content)
            )
            logger.info(f"Example added: {example_id} ({entity_type}/{entity_id})")
            return {"status": "success", "id": example_id}
        except Exception as e:
            logger.error(f"Failed to add example: {e}")
            return {"status": "error", "message": str(e)}

    def remove_example(self, example_id: str) -> Dict[str, str]:
        """Remove example by ID."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.remove_example(example_id))
            if success:
                logger.info(f"Example removed: {example_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Example not found"}
        except Exception as e:
            logger.error(f"Failed to remove example: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Model Discovery ====================

    def scan_directory_for_models(self, directory: str) -> Dict[str, Any]:
        """Scan an arbitrary directory for model files.

        Recursively discovers .safetensors and .gguf files, identifies their
        component type by inspecting file headers, and saves them to the database.
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            def on_progress(current: int, total: int, name: str):
                self._run_async(events.push_event("scan-progress", {
                    "current": current,
                    "total": total,
                    "name": name,
                    "source": "directory",
                }))

            components = scan_directory(directory, progress_callback=on_progress)
            saved = self._save_components(components)

            logger.info(f"Directory scan complete: {saved} components saved from {directory}")
            return {"status": "success", "found": len(components), "saved": saved}
        except FileNotFoundError as e:
            return {"status": "error", "message": str(e)}
        except Exception as e:
            logger.error(f"Failed to scan directory: {e}")
            return {"status": "error", "message": str(e)}

    def scan_and_discover_models(self) -> Dict[str, Any]:
        """Auto-discover models in the HuggingFace cache.

        Scans ~/.cache/huggingface/hub/ (or HF_HUB_CACHE / HF_HOME) for
        transformers, text encoders, VAE decoders, and tokenizers.
        """
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            def on_progress(current: int, total: int, name: str):
                self._run_async(events.push_event("scan-progress", {
                    "current": current,
                    "total": total,
                    "name": name,
                    "source": "hf_cache",
                }))

            components = scan_hf_cache(progress_callback=on_progress)
            saved = self._save_components(components)

            self._run_async(events.push_event("scan-complete", {
                "found": len(components),
                "saved": saved,
            }))

            logger.info(f"HF cache scan complete: {saved} components saved")
            return {"status": "success", "found": len(components), "saved": saved}
        except Exception as e:
            logger.error(f"Failed to scan HF cache: {e}")
            return {"status": "error", "message": str(e)}

    def _save_components(self, components: List[DiscoveredComponent]) -> int:
        """Save discovered components to the database. Returns count saved.

        Filters out sharded components when a non-sharded alternative of the
        same type exists for the same repo, avoiding duplicate entries.
        """
        components = filter_sharded_components(components)
        saved = 0
        for comp in components:
            try:
                self._run_async(
                    self._app_state.db.upsert_model_component(comp.to_dict())
                )
                saved += 1
            except Exception as e:
                logger.warning(f"Failed to save component {comp.path}: {e}")
        return saved

    # ==================== ComfyUI Conversion ====================

    def convert_comfyui_model(
        self, source_path: str, output_path: Optional[str] = None
    ) -> Dict[str, Any]:
        """Convert a ComfyUI format model to native FLUX format.

        Strips the `model.diffusion_model.` prefix from safetensors tensor names.
        Returns the output file path on success.
        """
        try:
            result_path = convert_comfyui_to_native(source_path, output_path)
            logger.info(f"ComfyUI conversion complete: {result_path}")
            return {"status": "success", "outputPath": result_path}
        except FileNotFoundError as e:
            return {"status": "error", "message": str(e)}
        except ValueError as e:
            return {"status": "error", "message": str(e)}
        except Exception as e:
            logger.error(f"Failed to convert ComfyUI model: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== Compatible Models ====================

    def get_compatible_models(
        self, base_model_id: str, target_type: str = "transformer"
    ) -> List[Dict[str, Any]]:
        """Get models compatible with a base model.

        Compatibility is family-based: all FLUX components are compatible
        with all other FLUX components. Returns components of `target_type`
        that match the base model's family.
        """
        try:
            if not self._app_state.db:
                return []

            models = self._run_async(
                self._app_state.db.get_compatible_models(base_model_id, target_type)
            )
            logger.debug(f"Found {len(models)} compatible models for {base_model_id}")
            return models
        except Exception as e:
            logger.error(f"Failed to get compatible models: {e}")
            return []

    # ==================== Bundles ====================

    def get_all_bundles(self) -> List[Dict[str, Any]]:
        """Get all bundles (model collections)."""
        try:
            if not self._app_state.db:
                return []

            bundles = self._run_async(self._app_state.db.get_all_bundles())
            logger.debug(f"Retrieved {len(bundles)} bundles")
            return bundles
        except Exception as e:
            logger.error(f"Failed to get bundles: {e}")
            return []

    def create_bundle(self, bundle: Dict[str, Any]) -> Dict[str, Any]:
        """Create a bundle."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            bundle_id = self._run_async(self._app_state.db.create_bundle(bundle))
            logger.info(f"Bundle created: {bundle_id}")
            return {"status": "success", "id": bundle_id}
        except Exception as e:
            logger.error(f"Failed to create bundle: {e}")
            return {"status": "error", "message": str(e)}

    def update_bundle(
        self,
        bundle_id: str,
        display_name: Optional[str] = None,
        description: Optional[str] = None,
        **kwargs,
    ) -> Dict[str, str]:
        """Update bundle details."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            updates = {}
            if display_name is not None:
                updates["name"] = display_name
            if description is not None:
                updates["description"] = description

            if not updates:
                return {"status": "error", "message": "No updates provided"}

            success = self._run_async(
                self._app_state.db.update_bundle(bundle_id, updates)
            )
            if success:
                logger.info(f"Bundle updated: {bundle_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Bundle not found or no updates"}
        except Exception as e:
            logger.error(f"Failed to update bundle: {e}")
            return {"status": "error", "message": str(e)}

    def delete_bundle(self, bundle_id: str) -> Dict[str, str]:
        """Delete a bundle."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_bundle(bundle_id))
            if success:
                logger.info(f"Bundle deleted: {bundle_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Bundle not found"}
        except Exception as e:
            logger.error(f"Failed to delete bundle: {e}")
            return {"status": "error", "message": str(e)}

    def set_active_bundle(self, bundle_id: str) -> Dict[str, str]:
        """Set active bundle for generation."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.set_active_bundle(bundle_id))
            if success:
                logger.info(f"Active bundle set: {bundle_id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "Bundle not found"}
        except Exception as e:
            logger.error(f"Failed to set active bundle: {e}")
            return {"status": "error", "message": str(e)}

    # ==================== LoRAs ====================

    def get_loras(self) -> List[Dict[str, Any]]:
        """Get all LoRAs."""
        try:
            if not self._app_state.db:
                return []

            loras = self._run_async(self._app_state.db.get_all_loras())
            logger.debug(f"Retrieved {len(loras)} LoRAs")
            return loras
        except Exception as e:
            logger.error(f"Failed to get LoRAs: {e}")
            return []

    def import_lora(
        self,
        source_path: str,
        name: Optional[str] = None,
        trigger_words: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Import LoRA from file."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            if not os.path.exists(source_path):
                return {"status": "error", "message": "File not found"}

            file_size = os.path.getsize(source_path)
            file_name = name or Path(source_path).stem

            lora_data = {
                "name": file_name,
                "path": source_path,
                "sizeBytes": file_size,
                "strength": 1.0,
                "isActive": False,
                "triggerWords": trigger_words,
            }

            lora_id = self._run_async(self._app_state.db.upsert_lora(lora_data))
            logger.info(f"LoRA imported: {lora_id} from {source_path}")
            return {"status": "success", "id": lora_id}
        except Exception as e:
            logger.error(f"Failed to import LoRA: {e}")
            return {"status": "error", "message": str(e)}

    def remove_lora(self, id: str) -> Dict[str, str]:
        """Remove a LoRA."""
        try:
            if not self._app_state.db:
                return {"status": "error", "message": "Database not initialized"}

            success = self._run_async(self._app_state.db.delete_lora(id))
            if success:
                logger.info(f"LoRA removed: {id}")
                return {"status": "success"}
            else:
                return {"status": "error", "message": "LoRA not found"}
        except Exception as e:
            logger.error(f"Failed to remove LoRA: {e}")
            return {"status": "error", "message": str(e)}

    def get_lora_file_info(self, path: str) -> Dict[str, Any]:
        """Get LoRA file metadata."""
        try:
            if not os.path.exists(path):
                return {"status": "error", "message": "File not found"}

            file_size = os.path.getsize(path)
            file_name = Path(path).stem
            file_ext = Path(path).suffix

            return {
                "name": file_name,
                "size": file_size,
                "format": file_ext.lstrip('.') if file_ext else "unknown",
                "path": path,
            }
        except Exception as e:
            logger.error(f"Failed to get LoRA file info: {e}")
            return {"status": "error", "message": str(e)}
