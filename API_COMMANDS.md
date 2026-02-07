# Frontend API Commands - Complete List

This document lists all `invoke()` calls found in the Vue/TypeScript frontend code.

## System & Health
- `health_check` - Check backend health status
- `get_runtime_config` - Get runtime configuration
- `get_system_stats` - Get CPU/GPU/RAM statistics

## Queue & Generation
- `client_add_to_queue` - Add generation job to queue
- `client_get_queue_jobs` - Get all queue jobs
- `client_get_queue_job` - Get specific queue job
- `client_cancel_queue_job` - Cancel a queue job
- `clear_completed_jobs` - Clear completed/failed jobs
- `increment_style_usage` - Increment style usage counter

## Gallery & Images
- `get_gallery_images` - Get gallery images with optional limit
- `search_gallery_images` - Search images by criteria
- `get_folder_images` - Get images in specific folder
- `get_uncategorized_images` - Get images without folder
- `delete_gallery_image` - Delete image from gallery
- `toggle_favorite` - Toggle favorite status
- `add_image_tag` - Add tag to image
- `remove_image_tag` - Remove tag from image
- `add_images_to_folder` - Add multiple images to folder
- `remove_images_from_folder` - Remove images from folder
- `analyze_image_for_prompt` - Analyze image to generate prompt

## Folders
- `get_folder_tree` - Get folder hierarchy
- `create_folder` - Create new folder
- `update_folder` - Update folder details
- `delete_folder` - Delete folder
- `move_folder` - Move folder to new parent
- `reorder_folders` - Reorder folder positions

## Tags
- `get_all_tags` - Get all available tags
- `update_tag` - Update tag details
- `delete_tag` - Delete tag
- `bulk_add_tag` - Add tag to multiple images
- `bulk_remove_tag` - Remove tag from multiple images

## Styles
- `get_all_styles` - Get all style presets
- `get_style_detail` - Get detailed style information
- `create_style` - Create new style
- `update_style` - Update style details
- `delete_style` - Delete style
- `add_lora_to_style` - Add LoRA to style
- `remove_lora_from_style` - Remove LoRA from style
- `add_style_example` - Add example to style
- `remove_style_example` - Remove style example
- `render_style_template` - Render style template preview
- `upload_style_thumbnail` - Upload style thumbnail image
- `delete_style_thumbnail` - Delete style thumbnail

## Models
- `get_all_models` - Get all model components
- `update_model` - Update model details
- `add_model_tag` - Add tag to model
- `remove_model_tag` - Remove tag from model
- `add_example` - Add example (model/bundle)
- `remove_example` - Remove example
- `scan_directory_for_models` - Scan directory for models
- `scan_and_discover_models` - Auto-discover models in default paths
- `convert_comfyui_model` - Convert ComfyUI model format
- `get_compatible_models` - Get compatible models for bundle

## LoRAs
- `get_loras` - Get all LoRA adapters
- `import_lora` - Import LoRA from file
- `remove_lora` - Remove LoRA
- `get_lora_file_info` - Get LoRA file metadata

## Bundles
- `get_all_bundles` - Get all model bundles
- `create_bundle` - Create new bundle
- `update_bundle` - Update bundle details
- `delete_bundle` - Delete bundle
- `set_active_bundle` - Set active bundle for generation

## Cache Management
- `get_cache_stats` - Get model cache statistics
- `get_cache_config` - Get cache configuration
- `set_cache_config` - Update cache configuration
- `set_cache_preset` - Set cache preset (balanced/performance/memory)
- `clear_model_cache` - Clear all cached models

## Auto-Tagging
- `get_auto_tag_settings` - Get auto-tag settings
- `update_auto_tag_settings` - Update auto-tag settings
- `check_vision_model_status` - Check vision model download status
- `download_vision_model` - Download vision model
- `clear_vision_model_locks` - Clear vision model lock files
- `auto_tag_images` - Auto-tag images using vision model

## Chatbot
- `chat_refine_prompt` - Refine prompt using AI chatbot

## Batch Generation
- `batch_parse_data` - Parse batch data (CSV/JSON)
- `batch_render_template` - Render batch template preview
- `batch_save_template` - Save batch template
- `batch_get_recent_templates` - Get recent template history
- `batch_generate_combinations` - Generate parameter combinations

## API Keys (Settings)
- Dynamic commands based on `apiKey.setCommand`:
  - `save_hf_token` - Save HuggingFace token
  - `save_claude_api_key` - Save Claude API key
  - `save_fal_key` - Save Fal.ai key

---

## Summary Statistics

**Total Commands:** ~75+ unique API commands

**Categories:**
- Queue & Generation: 6
- Gallery & Images: 10
- Folders: 6
- Tags: 5
- Styles: 11
- Models: 9
- LoRAs: 4
- Bundles: 6
- Cache: 5
- Auto-Tagging: 6
- Chatbot: 1
- Batch: 5
- System: 3
- API Keys: 3

## Implementation Status

✅ **Implemented in Python backend:**
- Basic health/system commands
- Queue management (basic)
- Gallery operations (basic)
- Settings/API keys
- Cache stats/config

⚠️ **Stub/Partial implementations:**
- Styles system
- Folders system
- Tags system
- Models management
- Bundles system
- Auto-tagging
- Chatbot
- Batch generation

❌ **Not yet implemented:**
- LoRA management
- Advanced model discovery
- ComfyUI conversion
- Template rendering
