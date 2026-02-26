"""Synchronous SQLite database layer for image metadata, folders, tags, and settings.

Uses plain ``sqlite3`` rather than aiosqlite — pywebview calls API methods from
non-main threads, and every call site would need ``asyncio.run()`` to bridge.
Synchronous access is simpler and avoids event-loop lifetime issues.
"""

from __future__ import annotations

import json
import logging
import sqlite3
import threading
import time
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)

_SCHEMA_PATH = Path(__file__).parent / "schema.sql"

# Current schema version — bump this when adding a new migration.
SCHEMA_VERSION = 2


def _dict_row(cursor: sqlite3.Cursor, row: tuple) -> dict[str, Any]:
    """Row factory that returns dicts keyed by column name."""
    return {col[0]: row[idx] for idx, col in enumerate(cursor.description)}


# ── Migrations ────────────────────────────────────────────────
#
# Each migration is a function(conn) that runs the SQL needed to
# move from version N-1 → N.  Migrations run inside a transaction.
# The list index + 1 equals the target version:
#   _MIGRATIONS[0] → upgrades 0 → 1
#   _MIGRATIONS[1] → upgrades 1 → 2  (etc.)
#
# schema.sql is the source of truth for *fresh* databases.
# Migrations handle *existing* databases that need updating.

def _migration_1(conn: sqlite3.Connection) -> None:
    """v0 → v1: Add bundle_types table."""
    conn.executescript("""
        CREATE TABLE IF NOT EXISTS bundle_types (
            id          TEXT PRIMARY KEY,
            label       TEXT NOT NULL,
            icon        TEXT,
            guide       TEXT,
            sort_order  INTEGER NOT NULL DEFAULT 0
        );
    """)


def _migration_2(conn: sqlite3.Connection) -> None:
    """v1 → v2: Add t5_encoder_config column to bundles."""
    try:
        conn.execute("ALTER TABLE bundles ADD COLUMN t5_encoder_config TEXT")
    except sqlite3.OperationalError:
        pass  # Column already exists (table was recreated from updated schema.sql)
    # Set NF4 quantization on FLUX.1 performance and balanced bundles
    nf4_json = json.dumps({"load_in_4bit": True, "bnb_4bit_quant_type": "nf4"})
    conn.execute(
        "UPDATE bundles SET t5_encoder_config = ? WHERE id IN (?, ?)",
        (nf4_json, "flux1_dev_performance", "flux1_dev_balanced"),
    )


_MIGRATIONS: list[callable] = [
    _migration_1,
    _migration_2,
]


class Database:
    """Thread-safe SQLite database at ``db_path``.

    All public methods are synchronous, use parameterized SQL, and are safe to
    call from any thread (the underlying ``sqlite3`` connection is protected by
    a lock and created with ``check_same_thread=False``).
    """

    def __init__(self, db_path: Path) -> None:
        self._db_path = db_path
        self._conn: sqlite3.Connection | None = None
        self._lock = threading.Lock()

    # ── Lifecycle ────────────────────────────────────────────────

    def connect(self) -> None:
        """Open the database, enable WAL + foreign keys, create tables, and run migrations."""
        self._db_path.parent.mkdir(parents=True, exist_ok=True)
        self._conn = sqlite3.connect(
            str(self._db_path), check_same_thread=False
        )
        self._conn.row_factory = _dict_row

        # PRAGMAs must be set before any transaction
        self._conn.execute("PRAGMA journal_mode = WAL")
        self._conn.execute("PRAGMA foreign_keys = ON")

        # Create tables (IF NOT EXISTS ensures this is safe on existing DBs)
        schema_sql = _SCHEMA_PATH.read_text()
        self._conn.executescript(schema_sql)

        # Run pending migrations
        self._migrate()
        logger.info("Database connected: %s (schema v%d)", self._db_path, SCHEMA_VERSION)

    def _migrate(self) -> None:
        """Run any pending migrations to bring the schema up to date."""
        current = self.conn.execute("PRAGMA user_version").fetchone()
        current_version = list(current.values())[0] if current else 0

        if current_version >= SCHEMA_VERSION:
            return

        for i in range(current_version, SCHEMA_VERSION):
            migration = _MIGRATIONS[i]
            logger.info("Running migration %d → %d (%s)", i, i + 1, migration.__doc__ or migration.__name__)
            migration(self.conn)
            self.conn.commit()

        self.conn.execute(f"PRAGMA user_version = {SCHEMA_VERSION}")
        self.conn.commit()
        logger.info("Schema upgraded: v%d → v%d", current_version, SCHEMA_VERSION)

    def close(self) -> None:
        if self._conn:
            self._conn.close()
            self._conn = None
            logger.info("Database closed")

    @property
    def conn(self) -> sqlite3.Connection:
        if not self._conn:
            raise RuntimeError("Database not connected")
        return self._conn

    # ── Images ───────────────────────────────────────────────────

    def insert_image(
        self,
        *,
        id: str,
        file_path: str,
        prompt: str,
        width: int,
        height: int,
        steps: int,
        cfg_scale: float,
        seed: int,
        thumbnail_path: str | None = None,
        raw_prompt: str | None = None,
        negative_prompt: str | None = None,
        file_size: int | None = None,
        bundle_id: str | None = None,
        model_config: str | None = None,
        loras: str | None = None,
        favorite: int = 0,
        generation_time_ms: int | None = None,
        status: str = "completed",
    ) -> dict[str, Any]:
        now = int(time.time())
        with self._lock:
            self.conn.execute(
                """INSERT INTO images
                   (id, file_path, thumbnail_path, prompt, raw_prompt, negative_prompt,
                    width, height, file_size, steps, cfg_scale, seed,
                    bundle_id, model_config, loras, favorite,
                    generation_time_ms, status, created_at, updated_at)
                   VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)""",
                (
                    id, file_path, thumbnail_path, prompt, raw_prompt, negative_prompt,
                    width, height, file_size, steps, cfg_scale, seed,
                    bundle_id, model_config, loras, favorite,
                    generation_time_ms, status, now, now,
                ),
            )
            self.conn.commit()
        return self.get_image(id)

    def get_image(self, image_id: str) -> dict[str, Any] | None:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM images WHERE id = ?", (image_id,)
            )
            row = cursor.fetchone()
        return row if row else None

    _IMAGE_SORT_COLUMNS = {"created_at", "file_size", "steps", "seed", "raw_prompt"}

    def get_images(
        self,
        *,
        limit: int = 50,
        offset: int = 0,
        folder_id: str | None = None,
        tag_id: int | None = None,
        search: str | None = None,
        favorites_only: bool = False,
        status: str | None = None,
        sort_by: str = "created_at",
        sort_order: str = "desc",
    ) -> dict[str, Any]:
        """Return paginated images with total count.

        Returns ``{"images": [...], "total": int}``.
        """
        where_clauses: list[str] = []
        params: list[Any] = []
        joins: list[str] = []

        if folder_id:
            joins.append("JOIN image_folders if_ ON if_.image_id = images.id")
            where_clauses.append("if_.folder_id = ?")
            params.append(folder_id)

        if tag_id is not None:
            joins.append("JOIN image_tags it ON it.image_id = images.id")
            where_clauses.append("it.tag_id = ?")
            params.append(tag_id)

        if search:
            where_clauses.append("images.prompt LIKE ?")
            params.append(f"%{search}%")

        if favorites_only:
            where_clauses.append("images.favorite = 1")

        if status:
            where_clauses.append("images.status = ?")
            params.append(status)

        join_sql = " ".join(joins)
        where_sql = f"WHERE {' AND '.join(where_clauses)}" if where_clauses else ""

        col = sort_by if sort_by in self._IMAGE_SORT_COLUMNS else "created_at"
        direction = "ASC" if sort_order.lower() == "asc" else "DESC"
        order_expr = f"LOWER(images.{col})" if col == "raw_prompt" else f"images.{col}"

        with self._lock:
            # Total count
            count_sql = f"SELECT COUNT(DISTINCT images.id) FROM images {join_sql} {where_sql}"
            cursor = self.conn.execute(count_sql, params)
            row = cursor.fetchone()
            # _dict_row returns a dict, so get the first value
            total = list(row.values())[0] if row else 0

            # Paginated results
            query_sql = (
                f"SELECT DISTINCT images.* FROM images {join_sql} {where_sql} "
                f"ORDER BY {order_expr} {direction} LIMIT ? OFFSET ?"
            )
            cursor = self.conn.execute(query_sql, [*params, limit, offset])
            rows = cursor.fetchall()

        return {"images": rows, "total": total}

    def update_image(self, image_id: str, **updates: Any) -> dict[str, Any] | None:
        allowed = {
            "thumbnail_path", "prompt", "negative_prompt", "favorite",
            "file_size", "status",
        }
        filtered = {k: v for k, v in updates.items() if k in allowed}
        if not filtered:
            return self.get_image(image_id)

        filtered["updated_at"] = int(time.time())
        set_clause = ", ".join(f"{k} = ?" for k in filtered)
        values = list(filtered.values()) + [image_id]

        with self._lock:
            self.conn.execute(
                f"UPDATE images SET {set_clause} WHERE id = ?", values
            )
            self.conn.commit()
        return self.get_image(image_id)

    def delete_image(self, image_id: str) -> bool:
        with self._lock:
            cursor = self.conn.execute(
                "DELETE FROM images WHERE id = ?", (image_id,)
            )
            self.conn.commit()
        return cursor.rowcount > 0

    def toggle_favorite(self, image_id: str) -> dict[str, Any] | None:
        with self._lock:
            self.conn.execute(
                "UPDATE images SET favorite = 1 - favorite, updated_at = ? WHERE id = ?",
                (int(time.time()), image_id),
            )
            self.conn.commit()
        return self.get_image(image_id)

    # ── Folders ──────────────────────────────────────────────────

    def create_folder(
        self,
        *,
        id: str,
        name: str,
        parent_id: str | None = None,
        color: str | None = None,
        icon: str | None = None,
        sort_order: int = 0,
    ) -> dict[str, Any] | None:
        now = int(time.time())
        with self._lock:
            self.conn.execute(
                """INSERT INTO folders (id, name, parent_id, color, icon, sort_order, created_at, updated_at)
                   VALUES (?,?,?,?,?,?,?,?)""",
                (id, name, parent_id, color, icon, sort_order, now, now),
            )
            self.conn.commit()
        return self._get_folder(id)

    def get_folders(self) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM folders ORDER BY sort_order, name"
            )
            return cursor.fetchall()

    def update_folder(self, folder_id: str, **updates: Any) -> dict[str, Any] | None:
        allowed = {"name", "parent_id", "color", "icon", "sort_order"}
        filtered = {k: v for k, v in updates.items() if k in allowed}
        if not filtered:
            return self._get_folder(folder_id)

        filtered["updated_at"] = int(time.time())
        set_clause = ", ".join(f"{k} = ?" for k in filtered)
        values = list(filtered.values()) + [folder_id]

        with self._lock:
            self.conn.execute(
                f"UPDATE folders SET {set_clause} WHERE id = ?", values
            )
            self.conn.commit()
        return self._get_folder(folder_id)

    def delete_folder(self, folder_id: str) -> bool:
        with self._lock:
            cursor = self.conn.execute(
                "DELETE FROM folders WHERE id = ?", (folder_id,)
            )
            self.conn.commit()
        return cursor.rowcount > 0

    def _get_folder(self, folder_id: str) -> dict[str, Any] | None:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM folders WHERE id = ?", (folder_id,)
            )
            row = cursor.fetchone()
        return row if row else None

    # ── Folder associations ──────────────────────────────────────

    def add_image_to_folder(self, image_id: str, folder_id: str) -> None:
        with self._lock:
            self.conn.execute(
                "INSERT OR IGNORE INTO image_folders (image_id, folder_id, added_at) VALUES (?,?,?)",
                (image_id, folder_id, int(time.time())),
            )
            self.conn.commit()

    def remove_image_from_folder(self, image_id: str, folder_id: str) -> None:
        with self._lock:
            self.conn.execute(
                "DELETE FROM image_folders WHERE image_id = ? AND folder_id = ?",
                (image_id, folder_id),
            )
            self.conn.commit()

    def get_folder_images(self, folder_id: str) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute(
                """SELECT images.* FROM images
                   JOIN image_folders if_ ON if_.image_id = images.id
                   WHERE if_.folder_id = ?
                   ORDER BY images.created_at DESC""",
                (folder_id,),
            )
            return cursor.fetchall()

    # ── Tags ─────────────────────────────────────────────────────

    def create_tag(
        self,
        *,
        name: str,
        color: str | None = None,
        category: str | None = None,
    ) -> dict[str, Any]:
        with self._lock:
            cursor = self.conn.execute(
                "INSERT INTO tags (name, color, category) VALUES (?,?,?)",
                (name, color, category),
            )
            self.conn.commit()
        return {"id": cursor.lastrowid, "name": name, "color": color, "category": category}

    def get_tags(self) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute("SELECT * FROM tags ORDER BY name")
            return cursor.fetchall()

    def update_tag(self, tag_id: int, **updates: Any) -> dict[str, Any] | None:
        allowed = {"name", "color", "category"}
        filtered = {k: v for k, v in updates.items() if k in allowed}
        if not filtered:
            return self._get_tag(tag_id)

        set_clause = ", ".join(f"{k} = ?" for k in filtered)
        values = list(filtered.values()) + [tag_id]
        with self._lock:
            self.conn.execute(
                f"UPDATE tags SET {set_clause} WHERE id = ?", values
            )
            self.conn.commit()
        return self._get_tag(tag_id)

    def delete_tag(self, tag_id: int) -> bool:
        with self._lock:
            cursor = self.conn.execute("DELETE FROM tags WHERE id = ?", (tag_id,))
            self.conn.commit()
        return cursor.rowcount > 0

    def _get_tag(self, tag_id: int) -> dict[str, Any] | None:
        with self._lock:
            cursor = self.conn.execute("SELECT * FROM tags WHERE id = ?", (tag_id,))
            row = cursor.fetchone()
        return row if row else None

    def get_tag_by_name(self, name: str) -> dict[str, Any] | None:
        with self._lock:
            cursor = self.conn.execute("SELECT * FROM tags WHERE name = ?", (name,))
            row = cursor.fetchone()
        return row if row else None

    # ── Tag associations ─────────────────────────────────────────

    def add_tag_to_image(self, image_id: str, tag_id: int) -> None:
        with self._lock:
            self.conn.execute(
                "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?,?)",
                (image_id, tag_id),
            )
            self.conn.commit()

    def remove_tag_from_image(self, image_id: str, tag_id: int) -> None:
        with self._lock:
            self.conn.execute(
                "DELETE FROM image_tags WHERE image_id = ? AND tag_id = ?",
                (image_id, tag_id),
            )
            self.conn.commit()

    def get_image_tags(self, image_id: str) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute(
                """SELECT tags.* FROM tags
                   JOIN image_tags it ON it.tag_id = tags.id
                   WHERE it.image_id = ?
                   ORDER BY tags.name""",
                (image_id,),
            )
            return cursor.fetchall()

    # ── Styles ─────────────────────────────────────────────────

    def insert_style(
        self,
        *,
        id: str,
        name: str,
        prompt_template: str,
        description: str | None = None,
        negative_prompt: str | None = None,
        category: str | None = None,
        thumbnail_path: str | None = None,
        bundle_id: str | None = None,
    ) -> dict[str, Any] | None:
        now = int(time.time())
        with self._lock:
            self.conn.execute(
                """INSERT INTO styles
                   (id, name, description, prompt_template, negative_prompt,
                    category, thumbnail_path, bundle_id, created_at, updated_at)
                   VALUES (?,?,?,?,?,?,?,?,?,?)""",
                (id, name, description, prompt_template, negative_prompt,
                 category, thumbnail_path, bundle_id, now, now),
            )
            self.conn.commit()
        return self.get_style(id)

    def get_style(self, style_id: str) -> dict[str, Any] | None:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM styles WHERE id = ?", (style_id,)
            )
            row = cursor.fetchone()
        return row if row else None

    _STYLE_SORT_COLUMNS = {"updated_at", "created_at", "name", "usage_count"}

    def get_styles(
        self,
        *,
        category: str | None = None,
        tag_id: int | None = None,
        search: str | None = None,
        favorites_only: bool = False,
        sort_by: str = "updated_at",
        sort_order: str = "desc",
    ) -> list[dict[str, Any]]:
        where_clauses: list[str] = []
        params: list[Any] = []
        joins: list[str] = []

        if category:
            where_clauses.append("styles.category = ?")
            params.append(category)

        if tag_id is not None:
            joins.append("JOIN style_tags st ON st.style_id = styles.id")
            where_clauses.append("st.tag_id = ?")
            params.append(tag_id)

        if search:
            where_clauses.append("(styles.name LIKE ? OR styles.prompt_template LIKE ?)")
            params.extend([f"%{search}%", f"%{search}%"])

        if favorites_only:
            where_clauses.append("styles.is_favorite = 1")

        join_sql = " ".join(joins)
        where_sql = f"WHERE {' AND '.join(where_clauses)}" if where_clauses else ""

        col = sort_by if sort_by in self._STYLE_SORT_COLUMNS else "updated_at"
        direction = "ASC" if sort_order.lower() == "asc" else "DESC"

        with self._lock:
            query_sql = (
                f"SELECT DISTINCT styles.* FROM styles {join_sql} {where_sql} "
                f"ORDER BY styles.{col} {direction}"
            )
            cursor = self.conn.execute(query_sql, params)
            return cursor.fetchall()

    def update_style(self, style_id: str, **updates: Any) -> dict[str, Any] | None:
        allowed = {
            "name", "description", "prompt_template", "negative_prompt",
            "category", "thumbnail_path", "bundle_id",
        }
        filtered = {k: v for k, v in updates.items() if k in allowed}
        if not filtered:
            return self.get_style(style_id)

        filtered["updated_at"] = int(time.time())
        set_clause = ", ".join(f"{k} = ?" for k in filtered)
        values = list(filtered.values()) + [style_id]

        with self._lock:
            self.conn.execute(
                f"UPDATE styles SET {set_clause} WHERE id = ?", values
            )
            self.conn.commit()
        return self.get_style(style_id)

    def delete_style(self, style_id: str) -> bool:
        with self._lock:
            cursor = self.conn.execute(
                "DELETE FROM styles WHERE id = ?", (style_id,)
            )
            self.conn.commit()
        return cursor.rowcount > 0

    def toggle_style_favorite(self, style_id: str) -> dict[str, Any] | None:
        with self._lock:
            self.conn.execute(
                "UPDATE styles SET is_favorite = 1 - is_favorite, updated_at = ? WHERE id = ?",
                (int(time.time()), style_id),
            )
            self.conn.commit()
        return self.get_style(style_id)

    def get_style_categories(self) -> list[str]:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT DISTINCT category FROM styles WHERE category IS NOT NULL ORDER BY category"
            )
            return [row["category"] for row in cursor.fetchall()]

    # ── Style ↔ LoRA ──────────────────────────────────────────

    def get_style_loras(self, style_id: str) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute(
                """SELECT sl.id, sl.style_id, sl.lora_id, sl.strength, sl.priority,
                          l.name AS lora_name, l.path AS lora_path
                   FROM style_loras sl
                   JOIN loras l ON l.id = sl.lora_id
                   WHERE sl.style_id = ?
                   ORDER BY sl.priority""",
                (style_id,),
            )
            return cursor.fetchall()

    def set_style_loras(
        self, style_id: str, loras: list[dict[str, Any]]
    ) -> list[dict[str, Any]]:
        """Replace all LoRA associations for a style.

        Each item in ``loras`` must have ``lora_id``, ``strength``, and
        optionally ``priority``.
        """
        with self._lock:
            self.conn.execute(
                "DELETE FROM style_loras WHERE style_id = ?", (style_id,)
            )
            for i, lora in enumerate(loras):
                self.conn.execute(
                    """INSERT INTO style_loras (style_id, lora_id, strength, priority)
                       VALUES (?,?,?,?)""",
                    (style_id, lora["lora_id"], lora.get("strength", 1.0),
                     lora.get("priority", i)),
                )
            self.conn.commit()
        return self.get_style_loras(style_id)

    # ── Style ↔ Tag ───────────────────────────────────────────

    def get_style_tags(self, style_id: str) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute(
                """SELECT tags.* FROM tags
                   JOIN style_tags st ON st.tag_id = tags.id
                   WHERE st.style_id = ?
                   ORDER BY tags.name""",
                (style_id,),
            )
            return cursor.fetchall()

    def set_style_tags(self, style_id: str, tag_ids: list[int]) -> list[dict[str, Any]]:
        """Replace all tag associations for a style."""
        with self._lock:
            self.conn.execute(
                "DELETE FROM style_tags WHERE style_id = ?", (style_id,)
            )
            for tag_id in tag_ids:
                self.conn.execute(
                    "INSERT OR IGNORE INTO style_tags (style_id, tag_id) VALUES (?,?)",
                    (style_id, tag_id),
                )
            self.conn.commit()
        return self.get_style_tags(style_id)

    def add_tag_to_style(self, style_id: str, tag_id: int) -> None:
        with self._lock:
            self.conn.execute(
                "INSERT OR IGNORE INTO style_tags (style_id, tag_id) VALUES (?,?)",
                (style_id, tag_id),
            )
            self.conn.commit()

    def remove_tag_from_style(self, style_id: str, tag_id: int) -> None:
        with self._lock:
            self.conn.execute(
                "DELETE FROM style_tags WHERE style_id = ? AND tag_id = ?",
                (style_id, tag_id),
            )
            self.conn.commit()

    # ── LoRAs ─────────────────────────────────────────────────

    def get_lora_by_path(self, path: str) -> dict[str, Any] | None:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM loras WHERE path = ?", (path,)
            )
            row = cursor.fetchone()
        return row if row else None

    def get_loras(self) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM loras ORDER BY name"
            )
            return cursor.fetchall()

    def get_lora(self, lora_id: str) -> dict[str, Any] | None:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM loras WHERE id = ?", (lora_id,)
            )
            row = cursor.fetchone()
        return row if row else None

    def insert_lora(
        self,
        *,
        id: str,
        name: str,
        path: str,
        trigger_words: str | None = None,
        base_model: str | None = None,
        size_bytes: int | None = None,
        strength: float = 1.0,
    ) -> dict[str, Any] | None:
        now = int(time.time())
        with self._lock:
            self.conn.execute(
                """INSERT INTO loras (id, name, path, trigger_words, base_model,
                   size_bytes, strength, created_at)
                   VALUES (?,?,?,?,?,?,?,?)""",
                (id, name, path, trigger_words, base_model, size_bytes,
                 strength, now),
            )
            self.conn.commit()
        return self.get_lora(id)

    # ── Examples ──────────────────────────────────────────────────

    def get_examples(self, entity_type: str, entity_id: str) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute(
                """SELECT * FROM examples
                   WHERE entity_type = ? AND entity_id = ?
                   ORDER BY created_at""",
                (entity_type, entity_id),
            )
            return cursor.fetchall()

    def insert_example(
        self,
        *,
        id: str,
        entity_type: str,
        entity_id: str,
        example_type: str,
        content: str,
    ) -> dict[str, Any] | None:
        now = int(time.time())
        with self._lock:
            self.conn.execute(
                """INSERT INTO examples (id, entity_type, entity_id, example_type, content, created_at)
                   VALUES (?,?,?,?,?,?)""",
                (id, entity_type, entity_id, example_type, content, now),
            )
            self.conn.commit()
            cursor = self.conn.execute(
                "SELECT * FROM examples WHERE id = ?", (id,)
            )
            return cursor.fetchone()

    def delete_example(self, example_id: str) -> bool:
        with self._lock:
            cursor = self.conn.execute(
                "DELETE FROM examples WHERE id = ?", (example_id,)
            )
            self.conn.commit()
        return cursor.rowcount > 0

    # ── Settings ─────────────────────────────────────────────────

    def get_setting(self, key: str) -> str | None:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT value FROM settings WHERE key = ?", (key,)
            )
            row = cursor.fetchone()
        return row["value"] if row else None

    def set_setting(self, key: str, value: str) -> None:
        with self._lock:
            self.conn.execute(
                "INSERT INTO settings (key, value) VALUES (?,?) ON CONFLICT(key) DO UPDATE SET value = ?",
                (key, value, value),
            )
            self.conn.commit()

    # ── Bundle Types ─────────────────────────────────────────────

    def get_bundle_types(self) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM bundle_types ORDER BY sort_order, label"
            )
            return cursor.fetchall()

    def get_bundle_type(self, type_id: str) -> dict[str, Any] | None:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM bundle_types WHERE id = ?", (type_id,)
            )
            return cursor.fetchone()

    def seed_bundle_types(self, types: list[dict[str, Any]]) -> None:
        """Bulk insert default bundle types."""
        for t in types:
            with self._lock:
                self.conn.execute(
                    """INSERT OR IGNORE INTO bundle_types
                       (id, label, icon, guide, sort_order)
                       VALUES (?,?,?,?,?)""",
                    (
                        t["id"], t["label"], t.get("icon"),
                        t.get("guide"), t.get("sort_order", 0),
                    ),
                )
            self.conn.commit()
        logger.info("Seeded %d bundle types into database", len(types))

    def count_bundle_types(self) -> int:
        with self._lock:
            cursor = self.conn.execute("SELECT COUNT(*) FROM bundle_types")
            row = cursor.fetchone()
        return list(row.values())[0] if row else 0

    # ── Bundles ────────────────────────────────────────────────────

    def _deserialize_bundle(self, row: dict[str, Any]) -> dict[str, Any]:
        """Parse JSON fields on a bundle row."""
        if row and row.get("fal_aspectratio"):
            try:
                row["fal_aspectratio"] = json.loads(row["fal_aspectratio"])
            except (json.JSONDecodeError, TypeError):
                row["fal_aspectratio"] = None
        if row and row.get("t5_encoder_config"):
            try:
                row["t5_encoder_config"] = json.loads(row["t5_encoder_config"])
            except (json.JSONDecodeError, TypeError):
                row["t5_encoder_config"] = None
        return row

    def get_bundles(self, *, include_hidden: bool = True) -> list[dict[str, Any]]:
        where = "" if include_hidden else "WHERE hidden = 0"
        with self._lock:
            cursor = self.conn.execute(
                f"SELECT * FROM bundles {where} ORDER BY transformer_type, label"
            )
            rows = cursor.fetchall()
        return [self._deserialize_bundle(r) for r in rows]

    def get_bundle(self, bundle_id: str) -> dict[str, Any] | None:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM bundles WHERE id = ?", (bundle_id,)
            )
            row = cursor.fetchone()
        return self._deserialize_bundle(row) if row else None

    def insert_bundle(
        self,
        *,
        id: str,
        label: str,
        description: str = "",
        transformer_type: str,
        tier: str = "balanced",
        transformer_model: str,
        vae_model: str,
        clip_tokenizer: str | None = None,
        clip_encoder: str | None = None,
        t5_tokenizer: str | None = None,
        t5_encoder: str | None = None,
        t5_encoder_config: dict | str | None = None,
        qwen3_tokenizer: str | None = None,
        qwen3_encoder: str | None = None,
        steps: int = 20,
        cfg_scale: float = 1.0,
        sampler: str = "euler",
        scheduler: str = "normal",
        vram_estimate_gb: float = 0.0,
        is_default: int = 1,
        source: str = "local",
        fal_endpoint: str | None = None,
        fal_aspectratio: list[str] | None = None,
        favorite: int = 0,
        hidden: int = 0,
    ) -> dict[str, Any] | None:
        ar_json = json.dumps(fal_aspectratio) if fal_aspectratio else None
        t5_config_json = json.dumps(t5_encoder_config) if isinstance(t5_encoder_config, dict) else t5_encoder_config
        with self._lock:
            self.conn.execute(
                """INSERT INTO bundles
                   (id, label, description, transformer_type, tier,
                    transformer_model, vae_model,
                    clip_tokenizer, clip_encoder, t5_tokenizer, t5_encoder,
                    t5_encoder_config, qwen3_tokenizer, qwen3_encoder,
                    steps, cfg_scale, sampler, scheduler,
                    vram_estimate_gb, is_default, source,
                    fal_endpoint, fal_aspectratio, favorite, hidden)
                   VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)""",
                (
                    id, label, description, transformer_type, tier,
                    transformer_model, vae_model,
                    clip_tokenizer, clip_encoder, t5_tokenizer, t5_encoder,
                    t5_config_json, qwen3_tokenizer, qwen3_encoder,
                    steps, cfg_scale, sampler, scheduler,
                    vram_estimate_gb, is_default, source,
                    fal_endpoint, ar_json, favorite, hidden,
                ),
            )
            self.conn.commit()
        return self.get_bundle(id)

    def update_bundle(self, bundle_id: str, **updates: Any) -> dict[str, Any] | None:
        allowed = {
            "label", "description", "transformer_type", "tier",
            "transformer_model", "vae_model",
            "clip_tokenizer", "clip_encoder", "t5_tokenizer", "t5_encoder",
            "t5_encoder_config", "qwen3_tokenizer", "qwen3_encoder",
            "steps", "cfg_scale", "sampler", "scheduler",
            "vram_estimate_gb", "is_default", "source",
            "fal_endpoint", "fal_aspectratio", "favorite", "hidden",
        }
        filtered = {k: v for k, v in updates.items() if k in allowed}
        if not filtered:
            return self.get_bundle(bundle_id)

        if "fal_aspectratio" in filtered and isinstance(filtered["fal_aspectratio"], list):
            filtered["fal_aspectratio"] = json.dumps(filtered["fal_aspectratio"])
        if "t5_encoder_config" in filtered and isinstance(filtered["t5_encoder_config"], dict):
            filtered["t5_encoder_config"] = json.dumps(filtered["t5_encoder_config"])

        set_clause = ", ".join(f"{k} = ?" for k in filtered)
        values = list(filtered.values()) + [bundle_id]

        with self._lock:
            self.conn.execute(
                f"UPDATE bundles SET {set_clause} WHERE id = ?", values
            )
            self.conn.commit()
        return self.get_bundle(bundle_id)

    def delete_bundle(self, bundle_id: str) -> bool:
        with self._lock:
            cursor = self.conn.execute(
                "DELETE FROM bundles WHERE id = ?", (bundle_id,)
            )
            self.conn.commit()
        return cursor.rowcount > 0

    def toggle_bundle_favorite(self, bundle_id: str) -> dict[str, Any] | None:
        with self._lock:
            self.conn.execute(
                "UPDATE bundles SET favorite = 1 - favorite WHERE id = ?",
                (bundle_id,),
            )
            self.conn.commit()
        return self.get_bundle(bundle_id)

    def toggle_bundle_hidden(self, bundle_id: str) -> dict[str, Any] | None:
        with self._lock:
            self.conn.execute(
                "UPDATE bundles SET hidden = 1 - hidden WHERE id = ?",
                (bundle_id,),
            )
            self.conn.commit()
        return self.get_bundle(bundle_id)

    def count_bundles(self) -> int:
        with self._lock:
            cursor = self.conn.execute("SELECT COUNT(*) FROM bundles")
            row = cursor.fetchone()
        return list(row.values())[0] if row else 0

    def seed_bundles(self, bundles: list[dict[str, Any]]) -> None:
        """Bulk insert default bundles."""
        for b in bundles:
            ar = b.get("fal_aspectratio")
            ar_json = json.dumps(ar) if ar else None
            t5_cfg = b.get("t5_encoder_config")
            t5_cfg_json = json.dumps(t5_cfg) if isinstance(t5_cfg, dict) else t5_cfg
            with self._lock:
                self.conn.execute(
                    """INSERT OR IGNORE INTO bundles
                       (id, label, description, transformer_type, tier,
                        transformer_model, vae_model,
                        clip_tokenizer, clip_encoder, t5_tokenizer, t5_encoder,
                        t5_encoder_config, qwen3_tokenizer, qwen3_encoder,
                        steps, cfg_scale, sampler, scheduler,
                        vram_estimate_gb, is_default, source,
                        fal_endpoint, fal_aspectratio, favorite, hidden)
                       VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)""",
                    (
                        b["id"], b["label"], b.get("description", ""),
                        b["transformer_type"], b.get("tier", "balanced"),
                        b["transformer_model"], b["vae_model"],
                        b.get("clip_tokenizer"), b.get("clip_encoder"),
                        b.get("t5_tokenizer"), b.get("t5_encoder"),
                        t5_cfg_json,
                        b.get("qwen3_tokenizer"), b.get("qwen3_encoder"),
                        b.get("steps", 20), b.get("cfg_scale", 1.0),
                        b.get("sampler", "euler"), b.get("scheduler", "normal"),
                        b.get("vram_estimate_gb", 0.0),
                        1 if b.get("is_default", True) else 0,
                        b.get("source", "local"),
                        b.get("fal_endpoint"), ar_json, 0, 0,
                    ),
                )
            self.conn.commit()
        logger.info("Seeded %d bundles into database", len(bundles))

    # ── Conversations ─────────────────────────────────────────────

    def create_conversation(self, id: str, title: str) -> dict[str, Any]:
        now = int(time.time())
        with self._lock:
            self.conn.execute(
                "INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?,?,?,?)",
                (id, title, now, now),
            )
            self.conn.commit()
            cursor = self.conn.execute(
                "SELECT * FROM conversations WHERE id = ?", (id,)
            )
            return cursor.fetchone()

    def get_conversations(self) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM conversations ORDER BY updated_at DESC"
            )
            return cursor.fetchall()

    def update_conversation(self, conv_id: str, **updates: Any) -> dict[str, Any] | None:
        allowed = {"title"}
        filtered = {k: v for k, v in updates.items() if k in allowed}
        filtered["updated_at"] = int(time.time())
        set_clause = ", ".join(f"{k} = ?" for k in filtered)
        values = list(filtered.values()) + [conv_id]
        with self._lock:
            self.conn.execute(
                f"UPDATE conversations SET {set_clause} WHERE id = ?", values
            )
            self.conn.commit()
            cursor = self.conn.execute(
                "SELECT * FROM conversations WHERE id = ?", (conv_id,)
            )
            return cursor.fetchone()

    def delete_conversation(self, conv_id: str) -> bool:
        with self._lock:
            cursor = self.conn.execute(
                "DELETE FROM conversations WHERE id = ?", (conv_id,)
            )
            self.conn.commit()
        return cursor.rowcount > 0

    def insert_conversation_message(
        self,
        *,
        id: str,
        conversation_id: str,
        role: str,
        content: str,
        display_text: str | None = None,
        image_paths: str | None = None,
        tool_calls: str | None = None,
    ) -> dict[str, Any]:
        now = int(time.time())
        with self._lock:
            self.conn.execute(
                """INSERT INTO conversation_messages
                   (id, conversation_id, role, content, display_text, image_paths, tool_calls, created_at)
                   VALUES (?,?,?,?,?,?,?,?)""",
                (id, conversation_id, role, content, display_text, image_paths, tool_calls, now),
            )
            self.conn.commit()
            cursor = self.conn.execute(
                "SELECT * FROM conversation_messages WHERE id = ?", (id,)
            )
            return cursor.fetchone()

    def get_conversation_messages(self, conversation_id: str) -> list[dict[str, Any]]:
        with self._lock:
            cursor = self.conn.execute(
                "SELECT * FROM conversation_messages WHERE conversation_id = ? ORDER BY created_at ASC",
                (conversation_id,),
            )
            return cursor.fetchall()
