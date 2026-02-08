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

        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                prompt TEXT NOT NULL,
                negative_prompt TEXT,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                steps INTEGER NOT NULL,
                cfg_scale REAL NOT NULL,
                seed INTEGER NOT NULL,
                sampler TEXT,
                scheduler TEXT,
                model_type TEXT,
                model_component_id TEXT,
                t5_component_id TEXT,
                clip_component_id TEXT,
                vae_component_id TEXT,
                bundle_id TEXT,
                created_at INTEGER NOT NULL,
                folder_id TEXT,
                favorite INTEGER DEFAULT 0,
                FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
            )
        """)

        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS folders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
            )
        """)

        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT
            )
        """)

        await self.conn.execute("""
            CREATE TABLE IF NOT EXISTS image_tags (
                image_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (image_id, tag_id),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )
        """)

        # Create indexes
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_images_created_at ON images(created_at DESC)
        """)
        await self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_images_folder_id ON images(folder_id)
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
                steps, cfg_scale, seed, sampler, scheduler, model_type,
                model_component_id, t5_component_id, clip_component_id,
                vae_component_id, bundle_id, created_at, folder_id, favorite
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            image_data.get("model_type"),
            image_data.get("model_component_id"),
            image_data.get("t5_component_id"),
            image_data.get("clip_component_id"),
            image_data.get("vae_component_id"),
            image_data.get("bundle_id"),
            int(datetime.utcnow().timestamp()),
            image_data.get("folder_id"),
            0,
        ))

        await self.conn.commit()
        return image_data.get("id")

    async def get_all_images(self, limit: Optional[int] = None) -> List[Dict[str, Any]]:
        """Get all images, optionally limited"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        query = "SELECT * FROM images ORDER BY created_at DESC"
        if limit:
            query += f" LIMIT {limit}"

        cursor = await self.conn.execute(query)
        rows = await cursor.fetchall()
        return [dict(row) for row in rows]

    async def get_image_by_id(self, image_id: str) -> Optional[Dict[str, Any]]:
        """Get image by ID"""
        if not self.conn:
            raise RuntimeError("Database not connected")

        cursor = await self.conn.execute(
            "SELECT * FROM images WHERE id = ?", (image_id,)
        )
        row = await cursor.fetchone()
        return dict(row) if row else None

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
