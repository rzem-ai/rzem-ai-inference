"""Database operations for gallery management"""

import aiosqlite
from typing import Optional, List, Dict, Any
from datetime import datetime
from loguru import logger
from pathlib import Path


class InferenceDb:
    """SQLite database for managing generated images"""

    def __init__(self, db_path: str):
        self.db_path = db_path
        self.conn: Optional[aiosqlite.Connection] = None

    async def connect(self) -> None:
        """Connect to database"""
        # Create parent directory if it doesn't exist
        Path(self.db_path).parent.mkdir(parents=True, exist_ok=True)

        self.conn = await aiosqlite.connect(self.db_path)
        self.conn.row_factory = aiosqlite.Row
        await self.init_schema()

    async def disconnect(self) -> None:
        """Disconnect from database"""
        if self.conn:
            await self.conn.close()
            self.conn = None

    async def init_schema(self) -> None:
        """Initialize database schema"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        # Images table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                file_path TEXT,
                thumbnail_path TEXT,
                prompt TEXT NOT NULL,
                negative_prompt TEXT,
                width INTEGER,
                height INTEGER,
                file_size INTEGER,
                steps INTEGER,
                cfg_scale REAL,
                seed INTEGER,
                sampler TEXT,
                scheduler TEXT,
                model_name TEXT NOT NULL DEFAULT 'flux-schnell',
                model_component_id TEXT,
                t5_component_id TEXT,
                clip_component_id TEXT,
                vae_component_id TEXT,
                bundle_id TEXT,
                loras TEXT,
                favorite INTEGER DEFAULT 0,
                generation_time_ms INTEGER DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'completed',
                session_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT 0
            )
        """)

        # Folders table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS folders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT,
                color TEXT,
                icon TEXT,
                sort_order INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE,
                UNIQUE(parent_id, name)
            )
        """)

        # Image-folders junction table (many-to-many)
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS image_folders (
                image_id TEXT NOT NULL,
                folder_id TEXT NOT NULL,
                added_at INTEGER NOT NULL,
                PRIMARY KEY (image_id, folder_id),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
                FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
            )
        """)

        # Tags table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT,
                category TEXT
            )
        """)

        # Image-tags junction table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS image_tags (
                image_id TEXT NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (image_id, tag_id),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )
        """)

        # Styles table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS styles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                prompt_template TEXT NOT NULL,
                default_strength REAL DEFAULT 1.0,
                strength_min REAL DEFAULT 0.5,
                strength_max REAL DEFAULT 1.5,
                category TEXT,
                thumbnail_path TEXT,
                is_favorite INTEGER DEFAULT 0,
                usage_count INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
        """)

        # Style-LoRAs association table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS style_loras (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                style_id TEXT NOT NULL,
                lora_id TEXT NOT NULL,
                strength REAL NOT NULL DEFAULT 1.0,
                priority INTEGER DEFAULT 0,
                FOREIGN KEY (style_id) REFERENCES styles(id) ON DELETE CASCADE,
                FOREIGN KEY (lora_id) REFERENCES loras(id) ON DELETE CASCADE,
                UNIQUE (style_id, lora_id)
            )
        """)

        # Style examples table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS style_examples (
                id TEXT PRIMARY KEY,
                style_id TEXT NOT NULL,
                example_type TEXT NOT NULL,
                content TEXT NOT NULL,
                generation_params TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (style_id) REFERENCES styles(id) ON DELETE CASCADE
            )
        """)

        # LoRAs table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS loras (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                trigger_words TEXT,
                base_model TEXT,
                size_bytes INTEGER,
                strength REAL DEFAULT 1.0,
                is_active INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                metadata TEXT
            )
        """)

        # Model components table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS model_components (
                id TEXT PRIMARY KEY,
                component_type TEXT NOT NULL,
                format TEXT NOT NULL,
                file_path TEXT NOT NULL UNIQUE,
                file_size INTEGER NOT NULL,
                file_hash TEXT,
                name TEXT NOT NULL,
                repo_id TEXT,
                repo_snapshot TEXT,
                architecture TEXT,
                quantization TEXT,
                supports_loras INTEGER DEFAULT 0,
                is_sharded INTEGER DEFAULT 0,
                shard_count INTEGER,
                vram_mb INTEGER,
                discovered_at INTEGER NOT NULL,
                last_verified_at INTEGER,
                is_available INTEGER DEFAULT 1,
                metadata TEXT
            )
        """)

        # Model bundles table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS model_bundles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                bundle_type TEXT NOT NULL,
                model_family TEXT NOT NULL,
                default_steps INTEGER,
                default_guidance REAL,
                step_min INTEGER,
                step_max INTEGER,
                total_vram_mb INTEGER,
                is_complete INTEGER DEFAULT 0,
                is_active INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                validation_errors TEXT
            )
        """)

        # Bundle-components junction table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS bundle_components (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bundle_id TEXT NOT NULL,
                component_id TEXT NOT NULL,
                component_role TEXT NOT NULL,
                is_required INTEGER DEFAULT 1,
                priority INTEGER DEFAULT 0,
                FOREIGN KEY (bundle_id) REFERENCES model_bundles(id) ON DELETE CASCADE,
                FOREIGN KEY (component_id) REFERENCES model_components(id) ON DELETE CASCADE,
                UNIQUE (bundle_id, component_role, component_id)
            )
        """)

        # Model tags table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS model_tags (
                model_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (model_id, tag),
                FOREIGN KEY (model_id) REFERENCES model_components(id) ON DELETE CASCADE
            )
        """)

        # Examples table (for models/bundles)
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS examples (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                example_type TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
        """)

        # Batch template history table
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS batch_template_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                template TEXT NOT NULL,
                used_at TEXT NOT NULL,
                image_count INTEGER NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
        """)

        # Settings table for storing app configuration
        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
        """)

        # Create indexes
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_images_created_at ON images(created_at DESC)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_images_session_status ON images(session_id, status, created_at DESC)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_image_folders_folder ON image_folders(folder_id)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_image_folders_image ON image_folders(image_id)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_styles_category ON styles(category)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_styles_favorite ON styles(is_favorite)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_style_loras_style ON style_loras(style_id)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_style_loras_lora ON style_loras(lora_id)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_examples_entity ON examples(entity_type, entity_id)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_components_type ON model_components(component_type)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_components_repo ON model_components(repo_id)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_bundles_family ON model_bundles(model_family)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_bundles_active ON model_bundles(is_active)
        """)

        await self.conn.commit()
        logger.info("Database schema initialized")

    async def insert_image(self, image_data: Dict[str, Any]) -> str:
        """Insert a new image record"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        await self.conn.execute("""
            INSERT INTO images (
                id, file_path, prompt, negative_prompt, width, height,
                steps, cfg_scale, seed, sampler, scheduler, model_name,
                model_component_id, t5_component_id, clip_component_id,
                vae_component_id, bundle_id, created_at, favorite
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            image_data.get("id"),
            image_data.get("file_path"),
            image_data.get("prompt"),
            image_data.get("negative_prompt"),
            image_data.get("width"),
            image_data.get("height"),
            image_data.get("steps"),
            image_data.get("cfg_scale"),
            image_data.get("seed"),
            image_data.get("sampler"),
            image_data.get("scheduler"),
            image_data.get("model_name", "flux-schnell"),
            image_data.get("model_component_id"),
            image_data.get("t5_component_id"),
            image_data.get("clip_component_id"),
            image_data.get("vae_component_id"),
            image_data.get("bundle_id"),
            image_data.get("created_at", int(datetime.utcnow().timestamp())),
            0,
        ))

        await self.conn.commit()
        return image_data.get("id")

    @staticmethod
    def _map_image_row(row: Dict[str, Any]) -> Dict[str, Any]:
        """Convert snake_case image row to camelCase for frontend"""
        return {
            "id": row["id"],
            "filePath": row.get("file_path"),
            "thumbnailPath": row.get("thumbnail_path"),
            "prompt": row.get("prompt", ""),
            "negativePrompt": row.get("negative_prompt"),
            "width": row.get("width"),
            "height": row.get("height"),
            "fileSize": row.get("file_size"),
            "steps": row.get("steps"),
            "cfgScale": row.get("cfg_scale"),
            "seed": row.get("seed"),
            "sampler": row.get("sampler"),
            "scheduler": row.get("scheduler"),
            "modelName": row.get("model_name", "flux-schnell"),
            "modelComponentId": row.get("model_component_id"),
            "t5ComponentId": row.get("t5_component_id"),
            "clipComponentId": row.get("clip_component_id"),
            "vaeComponentId": row.get("vae_component_id"),
            "bundleId": row.get("bundle_id"),
            "loras": row.get("loras"),
            "isFavorite": bool(row.get("favorite", 0)),
            "generationTimeMs": row.get("generation_time_ms", 0),
            "status": row.get("status", "completed"),
            "sessionId": row.get("session_id"),
            "createdAt": row.get("created_at", 0),
            "updatedAt": row.get("updated_at", 0),
        }

    async def get_all_images(self, limit: Optional[int] = None) -> List[Dict[str, Any]]:
        """Get all images, optionally limited"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        if limit and isinstance(limit, int):
            cursor = await self.conn.execute(
                "SELECT * FROM images ORDER BY created_at DESC LIMIT ?", (limit,)
            )
        else:
            cursor = await self.conn.execute(
                "SELECT * FROM images ORDER BY created_at DESC"
            )
        rows = await cursor.fetchall()
        return [self._map_image_row(dict(row)) for row in rows]

    async def get_image_by_id(self, image_id: str) -> Optional[Dict[str, Any]]:
        """Get image by ID"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "SELECT * FROM images WHERE id = ?", (image_id,)
        )
        row = await cursor.fetchone()
        return self._map_image_row(dict(row)) if row else None

    async def delete_image(self, image_id: str) -> bool:
        """Delete image by ID"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "DELETE FROM images WHERE id = ?", (image_id,)
        )
        await self.conn.commit()
        return cursor.rowcount > 0

    async def update_image_folder(self, image_id: str, folder_id: Optional[str]) -> bool:
        """Update image folder"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "UPDATE images SET folder_id = ? WHERE id = ?", (folder_id, image_id)
        )
        await self.conn.commit()
        return cursor.rowcount > 0

    async def toggle_favorite(self, image_id: str) -> bool:
        """Toggle favorite status"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "UPDATE images SET favorite = NOT favorite WHERE id = ?", (image_id,)
        )
        await self.conn.commit()
        return cursor.rowcount > 0

    # ==================== Settings ====================

    async def get_setting(self, key: str) -> Optional[str]:
        """Get a setting value by key"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "SELECT value FROM settings WHERE key = ?", (key,)
        )
        row = await cursor.fetchone()
        return row[0] if row else None

    async def set_setting(self, key: str, value: str) -> None:
        """Set a setting value (upsert)"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        await self.conn.execute("""
            INSERT INTO settings (key, value) VALUES (?, ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
        """, (key, value))
        await self.conn.commit()

    async def delete_setting(self, key: str) -> bool:
        """Delete a setting by key"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "DELETE FROM settings WHERE key = ?", (key,)
        )
        await self.conn.commit()
        return cursor.rowcount > 0

    # ==================== Folders ====================

    async def create_folder(
        self,
        name: str,
        parent_id: Optional[str] = None,
        color: Optional[str] = None,
        icon: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Create a new folder"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import uuid
        import time

        folder_id = str(uuid.uuid4())
        now = int(time.time())

        # Get next sort_order for this parent
        cursor = await self.conn.execute(
            "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM folders WHERE parent_id IS ?",
            (parent_id,)
        )
        row = await cursor.fetchone()
        sort_order = row[0] if row else 0

        await self.conn.execute(
            """INSERT INTO folders (id, name, parent_id, color, icon, sort_order, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
            (folder_id, name, parent_id, color, icon, sort_order, now, now)
        )
        await self.conn.commit()

        return {
            "id": folder_id,
            "name": name,
            "parentId": parent_id,
            "color": color,
            "icon": icon,
            "sortOrder": sort_order,
            "createdAt": now,
            "updatedAt": now,
        }

    async def update_folder(
        self,
        folder_id: str,
        name: Optional[str] = None,
        color: Optional[str] = None,
        icon: Optional[str] = None,
    ) -> bool:
        """Update folder details"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import time
        now = int(time.time())

        await self.conn.execute(
            """UPDATE folders SET
                name = COALESCE(?, name),
                color = COALESCE(?, color),
                icon = COALESCE(?, icon),
                updated_at = ?
               WHERE id = ?""",
            (name, color, icon, now, folder_id)
        )
        await self.conn.commit()
        return True

    async def delete_folder(self, folder_id: str) -> bool:
        """Delete a folder (cascade)"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute("DELETE FROM folders WHERE id = ?", (folder_id,))
        await self.conn.commit()
        return cursor.rowcount > 0

    async def move_folder(self, folder_id: str, new_parent_id: Optional[str]) -> bool:
        """Move folder to a new parent"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import time
        now = int(time.time())

        # Get next sort_order in new parent
        cursor = await self.conn.execute(
            "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM folders WHERE parent_id IS ?",
            (new_parent_id,)
        )
        row = await cursor.fetchone()
        sort_order = row[0] if row else 0

        await self.conn.execute(
            "UPDATE folders SET parent_id = ?, sort_order = ?, updated_at = ? WHERE id = ?",
            (new_parent_id, sort_order, now, folder_id)
        )
        await self.conn.commit()
        return True

    async def reorder_folders(self, folder_ids: List[str]) -> bool:
        """Reorder folders within the same parent"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import time
        now = int(time.time())

        for index, folder_id in enumerate(folder_ids):
            await self.conn.execute(
                "UPDATE folders SET sort_order = ?, updated_at = ? WHERE id = ?",
                (index, now, folder_id)
            )
        await self.conn.commit()
        return True

    async def get_folder_tree(self) -> List[Dict[str, Any]]:
        """Get folder tree with computed counts"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        # Get all folders
        cursor = await self.conn.execute(
            """SELECT id, name, parent_id, color, icon, sort_order, created_at, updated_at
               FROM folders
               ORDER BY parent_id NULLS FIRST, sort_order, name"""
        )
        rows = await cursor.fetchall()
        folders = [dict(row) for row in rows]

        # Get folder image counts
        cursor = await self.conn.execute(
            "SELECT folder_id, COUNT(*) FROM image_folders GROUP BY folder_id"
        )
        count_rows = await cursor.fetchall()
        counts = {row[0]: row[1] for row in count_rows}

        # Build parent-to-children map
        children_map: Dict[Optional[str], List[Dict[str, Any]]] = {}
        for folder in folders:
            parent = folder.get("parent_id")
            if parent not in children_map:
                children_map[parent] = []
            children_map[parent].append(folder)

        # Build tree recursively
        def build_node(folder: Dict[str, Any], path: List[str]) -> Dict[str, Any]:
            direct_count = counts.get(folder["id"], 0)
            children = []
            total_count = direct_count

            folder_id = folder.get("id")
            if folder_id:
                for child_folder in children_map.get(folder_id, []):
                    child_path = path + [folder["name"]]
                    child_node = build_node(child_folder, child_path)
                    children.append(child_node)
                    total_count += child_node.get("totalImageCount", 0)

            return {
                "id": folder["id"],
                "name": folder["name"],
                "parentId": folder.get("parent_id"),
                "color": folder.get("color"),
                "icon": folder.get("icon"),
                "sortOrder": folder.get("sort_order", 0),
                "createdAt": folder["created_at"],
                "updatedAt": folder["updated_at"],
                "children": children,
                "imageCount": direct_count,
                "totalImageCount": total_count,
                "path": path,
            }

        # Build root-level nodes
        root_folders = children_map.get(None, [])
        tree = [build_node(f, []) for f in root_folders]

        return tree

    async def add_images_to_folder(self, image_ids: List[str], folder_id: str) -> bool:
        """Add images to a folder"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import time
        now = int(time.time())

        for image_id in image_ids:
            await self.conn.execute(
                "INSERT OR IGNORE INTO image_folders (image_id, folder_id, added_at) VALUES (?, ?, ?)",
                (image_id, folder_id, now)
            )
        await self.conn.commit()
        return True

    async def remove_images_from_folder(self, image_ids: List[str], folder_id: str) -> bool:
        """Remove images from a folder"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        for image_id in image_ids:
            await self.conn.execute(
                "DELETE FROM image_folders WHERE image_id = ? AND folder_id = ?",
                (image_id, folder_id)
            )
        await self.conn.commit()
        return True

    async def get_folder_images(self, folder_id: str, limit: int = 100) -> List[Dict[str, Any]]:
        """Get images in a folder"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            """SELECT i.* FROM images i
               INNER JOIN image_folders if2 ON i.id = if2.image_id
               WHERE if2.folder_id = ?
               ORDER BY i.created_at DESC
               LIMIT ?""",
            (folder_id, limit)
        )
        rows = await cursor.fetchall()
        return [self._map_image_row(dict(row)) for row in rows]

    async def get_uncategorized_images(self, limit: int = 100) -> List[Dict[str, Any]]:
        """Get images not in any folder"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            """SELECT i.* FROM images i
               LEFT JOIN image_folders if2 ON i.id = if2.image_id
               WHERE if2.folder_id IS NULL
               ORDER BY i.created_at DESC
               LIMIT ?""",
            (limit,)
        )
        rows = await cursor.fetchall()
        return [self._map_image_row(dict(row)) for row in rows]

    # ==================== Tags ====================

    async def get_all_tags(self) -> List[Dict[str, Any]]:
        """Get all tags with usage counts"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            """SELECT t.id, t.name, t.color, t.category, COUNT(it.image_id) as usage_count
               FROM tags t
               LEFT JOIN image_tags it ON t.id = it.tag_id
               GROUP BY t.id
               ORDER BY usage_count DESC, t.name"""
        )
        rows = await cursor.fetchall()
        return [
            {
                "id": row[0],
                "name": row[1],
                "color": row[2],
                "category": row[3],
                "usageCount": row[4],
            }
            for row in rows
        ]

    async def update_tag(
        self,
        tag_id: int,
        name: Optional[str] = None,
        color: Optional[str] = None,
        category: Optional[str] = None,
    ) -> bool:
        """Update tag properties"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        await self.conn.execute(
            """UPDATE tags SET
                name = COALESCE(?, name),
                color = COALESCE(?, color),
                category = COALESCE(?, category)
               WHERE id = ?""",
            (name, color, category, tag_id)
        )
        await self.conn.commit()
        return True

    async def delete_tag(self, tag_id: int) -> bool:
        """Delete a tag"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute("DELETE FROM tags WHERE id = ?", (tag_id,))
        await self.conn.commit()
        return cursor.rowcount > 0

    async def add_image_tag(self, image_id: str, tag: str) -> bool:
        """Add tag to image"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        # Get or create tag
        await self.conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", (tag,))

        cursor = await self.conn.execute("SELECT id FROM tags WHERE name = ?", (tag,))
        row = await cursor.fetchone()
        if not row:
            return False
        tag_id = row[0]

        # Link tag to image
        await self.conn.execute(
            "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?, ?)",
            (image_id, tag_id)
        )
        await self.conn.commit()
        return True

    async def remove_image_tag(self, image_id: str, tag: str) -> bool:
        """Remove tag from image"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        await self.conn.execute(
            """DELETE FROM image_tags
               WHERE image_id = ? AND tag_id = (SELECT id FROM tags WHERE name = ?)""",
            (image_id, tag)
        )
        await self.conn.commit()
        return True

    async def bulk_add_tag(self, image_ids: List[str], tag: str) -> bool:
        """Bulk add tag to multiple images"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        # Get or create tag
        await self.conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", (tag,))

        cursor = await self.conn.execute("SELECT id FROM tags WHERE name = ?", (tag,))
        row = await cursor.fetchone()
        if not row:
            return False
        tag_id = row[0]

        # Add tag to all images
        for image_id in image_ids:
            await self.conn.execute(
                "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?, ?)",
                (image_id, tag_id)
            )
        await self.conn.commit()
        return True

    async def bulk_remove_tag(self, image_ids: List[str], tag: str) -> bool:
        """Bulk remove tag from multiple images"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute("SELECT id FROM tags WHERE name = ?", (tag,))
        row = await cursor.fetchone()
        if not row:
            return False
        tag_id = row[0]

        for image_id in image_ids:
            await self.conn.execute(
                "DELETE FROM image_tags WHERE image_id = ? AND tag_id = ?",
                (image_id, tag_id)
            )
        await self.conn.commit()
        return True

    async def get_image_tags(self, image_id: str) -> List[str]:
        """Get tags for an image"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            """SELECT t.name FROM tags t
               INNER JOIN image_tags it ON t.id = it.tag_id
               WHERE it.image_id = ?""",
            (image_id,)
        )
        rows = await cursor.fetchall()
        return [row[0] for row in rows]

    # ==================== Styles ====================

    async def get_all_styles(self) -> List[Dict[str, Any]]:
        """Get all styles"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            """SELECT id, name, description, prompt_template, default_strength,
                      strength_min, strength_max, category, thumbnail_path,
                      is_favorite, usage_count, created_at, updated_at
               FROM styles ORDER BY name"""
        )
        rows = await cursor.fetchall()
        return [
            {
                "id": row[0],
                "name": row[1],
                "description": row[2],
                "promptTemplate": row[3],
                "defaultStrength": row[4],
                "strengthMin": row[5],
                "strengthMax": row[6],
                "category": row[7],
                "thumbnailPath": row[8],
                "isFavorite": bool(row[9]),
                "usageCount": row[10],
                "createdAt": row[11],
                "updatedAt": row[12],
            }
            for row in rows
        ]

    async def get_style_detail(self, style_id: str) -> Optional[Dict[str, Any]]:
        """Get complete style detail including LoRAs and examples"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        # Get style info
        cursor = await self.conn.execute(
            """SELECT id, name, description, prompt_template, default_strength,
                      strength_min, strength_max, category, thumbnail_path,
                      is_favorite, usage_count, created_at, updated_at
               FROM styles WHERE id = ?""",
            (style_id,)
        )
        row = await cursor.fetchone()
        if not row:
            return None

        style_info = {
            "id": row[0],
            "name": row[1],
            "description": row[2],
            "promptTemplate": row[3],
            "defaultStrength": row[4],
            "strengthMin": row[5],
            "strengthMax": row[6],
            "category": row[7],
            "thumbnailPath": row[8],
            "isFavorite": bool(row[9]),
            "usageCount": row[10],
            "createdAt": row[11],
            "updatedAt": row[12],
        }

        # Get associated LoRAs
        cursor = await self.conn.execute(
            """SELECT sl.lora_id, l.name, l.trigger_words, sl.strength, sl.priority
               FROM style_loras sl
               JOIN loras l ON sl.lora_id = l.id
               WHERE sl.style_id = ?
               ORDER BY sl.priority""",
            (style_id,)
        )
        lora_rows = await cursor.fetchall()
        loras = [
            {
                "loraId": row[0],
                "loraName": row[1],
                "loraTriggerWords": row[2],
                "strength": row[3],
                "priority": row[4],
            }
            for row in lora_rows
        ]

        # Get examples
        cursor = await self.conn.execute(
            """SELECT id, style_id, example_type, content, generation_params, created_at
               FROM style_examples
               WHERE style_id = ?
               ORDER BY created_at DESC""",
            (style_id,)
        )
        example_rows = await cursor.fetchall()
        examples = [
            {
                "id": row[0],
                "styleId": row[1],
                "exampleType": row[2],
                "content": row[3],
                "generationParams": row[4],
                "createdAt": row[5],
            }
            for row in example_rows
        ]

        return {
            "info": style_info,
            "loras": loras,
            "examples": examples,
        }

    async def create_style(self, style_data: Dict[str, Any]) -> str:
        """Create a new style"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import uuid
        import time

        style_id = str(uuid.uuid4())
        now = int(time.time())

        await self.conn.execute(
            """INSERT INTO styles (id, name, description, prompt_template, default_strength,
                                  strength_min, strength_max, category, is_favorite,
                                  usage_count, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?)""",
            (
                style_id,
                style_data.get("name"),
                style_data.get("description"),
                style_data.get("promptTemplate", ""),
                style_data.get("defaultStrength", 1.0),
                style_data.get("strengthMin", 0.5),
                style_data.get("strengthMax", 1.5),
                style_data.get("category"),
                1 if style_data.get("isFavorite") else 0,
                now,
                now,
            )
        )
        await self.conn.commit()
        return style_id

    async def update_style(self, style_id: str, style_data: Dict[str, Any]) -> bool:
        """Update a style"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import time
        now = int(time.time())

        await self.conn.execute(
            """UPDATE styles SET
                name = ?, description = ?, prompt_template = ?,
                default_strength = ?, strength_min = ?, strength_max = ?,
                category = ?, is_favorite = ?, updated_at = ?
               WHERE id = ?""",
            (
                style_data.get("name"),
                style_data.get("description"),
                style_data.get("promptTemplate"),
                style_data.get("defaultStrength", 1.0),
                style_data.get("strengthMin", 0.5),
                style_data.get("strengthMax", 1.5),
                style_data.get("category"),
                1 if style_data.get("isFavorite") else 0,
                now,
                style_id,
            )
        )
        await self.conn.commit()
        return True

    async def delete_style(self, style_id: str) -> bool:
        """Delete a style"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute("DELETE FROM styles WHERE id = ?", (style_id,))
        await self.conn.commit()
        return cursor.rowcount > 0

    async def add_lora_to_style(
        self, style_id: str, lora_id: str, strength: float = 1.0, priority: int = 0
    ) -> bool:
        """Add a LoRA to a style"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        await self.conn.execute(
            """INSERT INTO style_loras (style_id, lora_id, strength, priority)
               VALUES (?, ?, ?, ?)
               ON CONFLICT(style_id, lora_id) DO UPDATE SET
                   strength = excluded.strength, priority = excluded.priority""",
            (style_id, lora_id, strength, priority)
        )
        await self.conn.commit()
        return True

    async def remove_lora_from_style(self, style_id: str, lora_id: str) -> bool:
        """Remove a LoRA from a style"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "DELETE FROM style_loras WHERE style_id = ? AND lora_id = ?",
            (style_id, lora_id)
        )
        await self.conn.commit()
        return cursor.rowcount > 0

    async def add_style_example(
        self,
        style_id: str,
        example_type: str,
        content: str,
        generation_params: Optional[str] = None,
    ) -> str:
        """Add an example to a style"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import uuid
        import time

        example_id = str(uuid.uuid4())
        now = int(time.time())

        await self.conn.execute(
            """INSERT INTO style_examples (id, style_id, example_type, content, generation_params, created_at)
               VALUES (?, ?, ?, ?, ?, ?)""",
            (example_id, style_id, example_type, content, generation_params, now)
        )
        await self.conn.commit()
        return example_id

    async def remove_style_example(self, example_id: str) -> bool:
        """Remove an example from a style"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "DELETE FROM style_examples WHERE id = ?", (example_id,)
        )
        await self.conn.commit()
        return cursor.rowcount > 0

    async def increment_style_usage(self, style_id: str) -> bool:
        """Increment usage count for a style"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        await self.conn.execute(
            "UPDATE styles SET usage_count = usage_count + 1 WHERE id = ?",
            (style_id,)
        )
        await self.conn.commit()
        return True

    async def update_style_thumbnail(
        self, style_id: str, thumbnail_path: Optional[str]
    ) -> bool:
        """Update thumbnail path for a style"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import time
        now = int(time.time())

        await self.conn.execute(
            "UPDATE styles SET thumbnail_path = ?, updated_at = ? WHERE id = ?",
            (thumbnail_path, now, style_id)
        )
        await self.conn.commit()
        return True

    # ==================== Search ====================

    async def search_gallery_images(
        self,
        query: Optional[str] = None,
        tags: Optional[List[str]] = None,
        folder_id: Optional[str] = None,
        favorites_only: bool = False,
        limit: int = 100,
    ) -> List[Dict[str, Any]]:
        """Search gallery images"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        # Build query
        sql = "SELECT DISTINCT i.* FROM images i"
        params: List[Any] = []
        where_clauses: List[str] = []

        if tags:
            sql += " INNER JOIN image_tags it ON i.id = it.image_id"
            sql += " INNER JOIN tags t ON it.tag_id = t.id"
            where_clauses.append(f"t.name IN ({','.join('?' * len(tags))})")
            params.extend(tags)

        if folder_id:
            sql += " INNER JOIN image_folders if2 ON i.id = if2.image_id"
            where_clauses.append("if2.folder_id = ?")
            params.append(folder_id)

        if query:
            where_clauses.append("(i.prompt LIKE ? OR i.negative_prompt LIKE ?)")
            search_term = f"%{query}%"
            params.extend([search_term, search_term])

        if favorites_only:
            where_clauses.append("i.favorite = 1")

        if where_clauses:
            sql += " WHERE " + " AND ".join(where_clauses)

        sql += " ORDER BY i.created_at DESC LIMIT ?"
        params.append(limit)

        cursor = await self.conn.execute(sql, tuple(params))
        rows = await cursor.fetchall()
        return [self._map_image_row(dict(row)) for row in rows]

    # ==================== LoRAs ====================

    async def get_all_loras(self) -> List[Dict[str, Any]]:
        """Get all LoRAs"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            """SELECT id, name, path, trigger_words, base_model, size_bytes,
                      strength, is_active, created_at, metadata
               FROM loras ORDER BY name"""
        )
        rows = await cursor.fetchall()

        loras = []
        for row in rows:
            metadata = row[9]
            if metadata and isinstance(metadata, str):
                import json
                metadata = json.loads(metadata)

            loras.append({
                "id": row[0],
                "name": row[1],
                "path": row[2],
                "triggerWords": row[3],
                "baseModel": row[4],
                "sizeBytes": row[5],
                "strength": row[6],
                "isActive": bool(row[7]),
                "createdAt": row[8],
                "metadata": metadata or {}
            })
        return loras

    async def upsert_lora(self, lora: Dict[str, Any]) -> str:
        """Insert or update a LoRA"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import uuid
        import time
        import json

        lora_id = lora.get("id") or str(uuid.uuid4())
        now = int(time.time())

        metadata_json = json.dumps(lora.get("metadata", {}))

        await self.conn.execute(
            """INSERT INTO loras (id, name, path, trigger_words, base_model, size_bytes,
                                  strength, is_active, created_at, metadata)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   path = excluded.path,
                   trigger_words = excluded.trigger_words,
                   base_model = excluded.base_model,
                   size_bytes = excluded.size_bytes,
                   strength = excluded.strength,
                   is_active = excluded.is_active,
                   metadata = excluded.metadata""",
            (
                lora_id,
                lora.get("name", ""),
                lora.get("path", ""),
                lora.get("triggerWords"),
                lora.get("baseModel"),
                lora.get("sizeBytes", 0),
                lora.get("strength", 1.0),
                1 if lora.get("isActive") else 0,
                now,
                metadata_json
            )
        )
        await self.conn.commit()
        return lora_id

    async def delete_lora(self, lora_id: str) -> bool:
        """Delete a LoRA"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "DELETE FROM loras WHERE id = ?", (lora_id,)
        )
        await self.conn.commit()
        return cursor.rowcount > 0

    # ==================== Single-Row Lookups ====================

    async def get_model_component_by_id(self, component_id: str) -> Optional[Dict[str, Any]]:
        """Get a single model component by ID"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            """SELECT id, component_type, format, file_path, file_size, name,
                      repo_id, repo_snapshot, architecture, quantization
               FROM model_components WHERE id = ?""",
            (component_id,)
        )
        row = await cursor.fetchone()
        if not row:
            return None
        return {
            "id": row[0],
            "componentType": row[1],
            "format": row[2],
            "filePath": row[3],
            "fileSize": row[4],
            "name": row[5],
            "repoId": row[6],
            "repoSnapshot": row[7],
            "architecture": row[8],
            "quantization": row[9],
        }

    async def get_lora_by_id(self, lora_id: str) -> Optional[Dict[str, Any]]:
        """Get a single LoRA by ID"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "SELECT id, name, path, strength FROM loras WHERE id = ?",
            (lora_id,)
        )
        row = await cursor.fetchone()
        if not row:
            return None
        return {
            "id": row[0],
            "name": row[1],
            "path": row[2],
            "strength": row[3],
        }

    # ==================== Model Components ====================

    async def get_all_model_components(self) -> List[Dict[str, Any]]:
        """Get all model components"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            """SELECT id, component_type, format, file_path, file_size, file_hash, name,
                      repo_id, repo_snapshot, architecture, quantization, supports_loras,
                      is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                      is_available, metadata
               FROM model_components ORDER BY name"""
        )
        rows = await cursor.fetchall()

        components = []
        for row in rows:
            metadata = row[18]
            if metadata and isinstance(metadata, str):
                import json
                metadata = json.loads(metadata)

            # Get tags for this model
            tag_cursor = await self.conn.execute(
                "SELECT tag FROM model_tags WHERE model_id = ?", (row[0],)
            )
            tag_rows = await tag_cursor.fetchall()
            tags = [tag_row[0] for tag_row in tag_rows]

            components.append({
                "id": row[0],
                "componentType": row[1],
                "format": row[2],
                "filePath": row[3],
                "fileSize": row[4],
                "fileHash": row[5],
                "name": row[6],
                "repoId": row[7],
                "repoSnapshot": row[8],
                "architecture": row[9],
                "quantization": row[10],
                "supportsLoras": bool(row[11]),
                "isSharded": bool(row[12]),
                "shardCount": row[13],
                "vramMb": row[14],
                "discoveredAt": row[15],
                "lastVerifiedAt": row[16],
                "isAvailable": bool(row[17]),
                "metadata": metadata or {},
                "tags": tags
            })
        return components

    async def update_model_component(self, model_id: str, updates: Dict[str, Any]) -> bool:
        """Update model component"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import time
        now = int(time.time())

        # Build dynamic update query
        update_fields = []
        params = []

        field_mapping = {
            "name": "name",
            "architecture": "architecture",
            "quantization": "quantization",
            "supportsLoras": "supports_loras",
            "vramMb": "vram_mb",
            "isAvailable": "is_available"
        }

        for key, db_field in field_mapping.items():
            if key in updates:
                update_fields.append(f"{db_field} = ?")
                value = updates[key]
                # Convert boolean fields
                if key in ["supportsLoras", "isAvailable"]:
                    value = 1 if value else 0
                params.append(value)

        if not update_fields:
            return False

        update_fields.append("last_verified_at = ?")
        params.append(now)
        params.append(model_id)

        sql = f"UPDATE model_components SET {', '.join(update_fields)} WHERE id = ?"
        await self.conn.execute(sql, tuple(params))
        await self.conn.commit()
        return True

    async def add_model_tag(self, model_id: str, tag: str) -> bool:
        """Add tag to model"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        try:
            await self.conn.execute(
                "INSERT INTO model_tags (model_id, tag) VALUES (?, ?)",
                (model_id, tag)
            )
            await self.conn.commit()
            return True
        except Exception:
            return False  # Already exists or other error

    async def remove_model_tag(self, model_id: str, tag: str) -> bool:
        """Remove tag from model"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "DELETE FROM model_tags WHERE model_id = ? AND tag = ?",
            (model_id, tag)
        )
        await self.conn.commit()
        return cursor.rowcount > 0

    # ==================== Model Bundles ====================

    async def get_all_bundles(self) -> List[Dict[str, Any]]:
        """Get all model bundles"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            """SELECT id, name, description, bundle_type, model_family, default_steps,
                      default_guidance, step_min, step_max, total_vram_mb, is_complete,
                      is_active, created_at, updated_at, validation_errors
               FROM model_bundles ORDER BY name"""
        )
        rows = await cursor.fetchall()

        bundles = []
        for row in rows:
            # Get components for this bundle
            comp_cursor = await self.conn.execute(
                """SELECT bc.component_id, bc.component_role, mc.name, mc.component_type
                   FROM bundle_components bc
                   JOIN model_components mc ON bc.component_id = mc.id
                   WHERE bc.bundle_id = ?
                   ORDER BY bc.priority""",
                (row[0],)
            )
            comp_rows = await comp_cursor.fetchall()
            components = [
                {
                    "componentId": comp_row[0],
                    "role": comp_row[1],
                    "name": comp_row[2],
                    "componentType": comp_row[3]
                }
                for comp_row in comp_rows
            ]

            bundles.append({
                "id": row[0],
                "name": row[1],
                "description": row[2],
                "bundleType": row[3],
                "modelFamily": row[4],
                "defaultSteps": row[5],
                "defaultGuidance": row[6],
                "stepMin": row[7],
                "stepMax": row[8],
                "totalVramMb": row[9],
                "isComplete": bool(row[10]),
                "isActive": bool(row[11]),
                "createdAt": row[12],
                "updatedAt": row[13],
                "validationErrors": row[14],
                "components": components
            })
        return bundles

    async def create_bundle(self, bundle: Dict[str, Any]) -> str:
        """Create a bundle"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import uuid
        import time

        bundle_id = str(uuid.uuid4())
        now = int(time.time())

        await self.conn.execute(
            """INSERT INTO model_bundles (id, name, description, bundle_type, model_family,
                                          default_steps, default_guidance, step_min, step_max,
                                          total_vram_mb, is_complete, is_active, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                bundle_id,
                bundle.get("name", ""),
                bundle.get("description"),
                bundle.get("bundleType", "diffusion"),
                bundle.get("modelFamily", "flux"),
                bundle.get("defaultSteps"),
                bundle.get("defaultGuidance"),
                bundle.get("stepMin"),
                bundle.get("stepMax"),
                bundle.get("totalVramMb", 0),
                1 if bundle.get("isComplete") else 0,
                1 if bundle.get("isActive") else 0,
                now,
                now
            )
        )
        await self.conn.commit()
        return bundle_id

    async def update_bundle(self, bundle_id: str, updates: Dict[str, Any]) -> bool:
        """Update bundle"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import time
        now = int(time.time())

        # Build dynamic update query
        update_fields = []
        params = []

        field_mapping = {
            "name": "name",
            "description": "description",
            "bundleType": "bundle_type",
            "modelFamily": "model_family",
            "defaultSteps": "default_steps",
            "defaultGuidance": "default_guidance",
            "stepMin": "step_min",
            "stepMax": "step_max",
            "totalVramMb": "total_vram_mb",
            "isComplete": "is_complete",
            "isActive": "is_active"
        }

        for key, db_field in field_mapping.items():
            if key in updates:
                update_fields.append(f"{db_field} = ?")
                value = updates[key]
                # Convert boolean fields
                if key in ["isComplete", "isActive"]:
                    value = 1 if value else 0
                params.append(value)

        if not update_fields:
            return False

        update_fields.append("updated_at = ?")
        params.append(now)
        params.append(bundle_id)

        sql = f"UPDATE model_bundles SET {', '.join(update_fields)} WHERE id = ?"
        await self.conn.execute(sql, tuple(params))
        await self.conn.commit()
        return True

    async def delete_bundle(self, bundle_id: str) -> bool:
        """Delete a bundle"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "DELETE FROM model_bundles WHERE id = ?", (bundle_id,)
        )
        await self.conn.commit()
        return cursor.rowcount > 0

    async def set_active_bundle(self, bundle_id: str) -> bool:
        """Set active bundle (deactivates all others)"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        # Deactivate all bundles
        await self.conn.execute("UPDATE model_bundles SET is_active = 0")

        # Activate the specified bundle
        await self.conn.execute(
            "UPDATE model_bundles SET is_active = 1 WHERE id = ?", (bundle_id,)
        )
        await self.conn.commit()
        return True

    # ==================== Examples ====================

    async def add_example(
        self,
        entity_type: str,
        entity_id: str,
        example_type: str,
        content: str,
    ) -> str:
        """Add an example to a model or bundle"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import uuid

        example_id = str(uuid.uuid4())
        now = datetime.utcnow().isoformat() + "Z"

        await self.conn.execute(
            """INSERT INTO examples (id, entity_type, entity_id, example_type, content, created_at)
               VALUES (?, ?, ?, ?, ?, ?)""",
            (example_id, entity_type, entity_id, example_type, content, now),
        )
        await self.conn.commit()
        return example_id

    async def remove_example(self, example_id: str) -> bool:
        """Remove an example by ID"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "DELETE FROM examples WHERE id = ?", (example_id,)
        )
        await self.conn.commit()
        return cursor.rowcount > 0

    async def get_examples(
        self, entity_type: str, entity_id: str
    ) -> List[Dict[str, Any]]:
        """Get examples for an entity"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            """SELECT id, entity_type, entity_id, example_type, content, created_at
               FROM examples
               WHERE entity_type = ? AND entity_id = ?
               ORDER BY created_at""",
            (entity_type, entity_id),
        )
        rows = await cursor.fetchall()
        return [
            {
                "id": row[0],
                "entityType": row[1],
                "entityId": row[2],
                "exampleType": row[3],
                "content": row[4],
                "createdAt": row[5],
            }
            for row in rows
        ]

    # ==================== Model Component Upsert ====================

    async def upsert_model_component(self, component: Dict[str, Any]) -> str:
        """Insert or update a model component (used by scanner)"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        import uuid
        import json
        import time

        comp_id = component.get("id") or str(uuid.uuid4())
        now = int(time.time())
        metadata_json = json.dumps(component.get("metadata", {}))

        await self.conn.execute(
            """INSERT INTO model_components (
                id, component_type, format, file_path, file_size, file_hash, name,
                repo_id, repo_snapshot, architecture, quantization, supports_loras,
                is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                is_available, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(file_path) DO UPDATE SET
                name = excluded.name,
                component_type = excluded.component_type,
                format = excluded.format,
                file_size = excluded.file_size,
                last_verified_at = excluded.last_verified_at,
                is_available = excluded.is_available""",
            (
                comp_id,
                component.get("componentType", ""),
                component.get("format", ""),
                component.get("filePath", ""),
                component.get("fileSize", 0),
                component.get("fileHash"),
                component.get("name", ""),
                component.get("repoId"),
                component.get("repoSnapshot"),
                component.get("architecture"),
                component.get("quantization"),
                1 if component.get("supportsLoras") else 0,
                1 if component.get("isSharded") else 0,
                component.get("shardCount"),
                component.get("vramMb"),
                now,
                now,
                1,
                metadata_json,
            ),
        )
        await self.conn.commit()
        return comp_id

    # ==================== Compatible Models ====================

    async def get_compatible_models(
        self, base_model_id: str, target_type: str
    ) -> List[Dict[str, Any]]:
        """Get models compatible with a base model (family-based matching)"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        # Get the base model's architecture to infer family
        cursor = await self.conn.execute(
            "SELECT architecture FROM model_components WHERE id = ?",
            (base_model_id,),
        )
        row = await cursor.fetchone()
        if not row:
            return []

        family = _infer_family(row[0])

        # Get all components of the target type that match the family
        cursor = await self.conn.execute(
            """SELECT id, component_type, format, file_path, file_size, file_hash, name,
                      repo_id, repo_snapshot, architecture, quantization, supports_loras,
                      is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                      is_available, metadata
               FROM model_components
               WHERE is_available = 1
               ORDER BY name""",
        )
        rows = await cursor.fetchall()

        results = []
        for r in rows:
            comp_type = _component_type_to_model_type(r[1])
            comp_family = _infer_family(r[9])
            if comp_type == target_type and comp_family == family:
                metadata = r[18]
                if metadata and isinstance(metadata, str):
                    import json
                    metadata = json.loads(metadata)

                tag_cursor = await self.conn.execute(
                    "SELECT tag FROM model_tags WHERE model_id = ?", (r[0],)
                )
                tag_rows = await tag_cursor.fetchall()
                tags = [tr[0] for tr in tag_rows]

                results.append({
                    "id": r[0],
                    "componentType": r[1],
                    "format": r[2],
                    "filePath": r[3],
                    "fileSize": r[4],
                    "fileHash": r[5],
                    "name": r[6],
                    "repoId": r[7],
                    "repoSnapshot": r[8],
                    "architecture": r[9],
                    "quantization": r[10],
                    "supportsLoras": bool(r[11]),
                    "isSharded": bool(r[12]),
                    "shardCount": r[13],
                    "vramMb": r[14],
                    "discoveredAt": r[15],
                    "lastVerifiedAt": r[16],
                    "isAvailable": bool(r[17]),
                    "metadata": metadata or {},
                    "tags": tags,
                })
        return results


def _infer_family(architecture: Optional[str]) -> str:
    """Infer model family from architecture string"""
    if not architecture:
        return "other"
    arch = architecture.lower()
    if "flux" in arch or "clip" in arch or "t5" in arch or "vae" in arch:
        return "flux"
    if "z-image" in arch or "zimage" in arch:
        return "zimage"
    return "other"


def _component_type_to_model_type(comp_type: str) -> str:
    """Map component type to model type for compatibility matching"""
    mapping = {
        "transformer": "transformer",
        "t5_encoder": "text_encoder",
        "clip_encoder": "text_encoder",
        "vae": "vae",
        "clip_tokenizer": "tokenizer",
        "t5_tokenizer": "tokenizer",
    }
    return mapping.get(comp_type, "other")
