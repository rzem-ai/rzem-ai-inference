import Database from 'better-sqlite3';

/**
 * Creates a minimal in-memory SQLite database for unit tests.
 * Each test gets an isolated instance — no shared state between tests.
 */
export function makeDb(): Database.Database {
  const db = new Database(':memory:');
  db.exec(`
    CREATE TABLE settings (
      key   TEXT PRIMARY KEY,
      value TEXT NOT NULL
    );

    CREATE TABLE styles (
      id               TEXT PRIMARY KEY,
      name             TEXT NOT NULL,
      description      TEXT,
      prompt_template  TEXT NOT NULL,
      negative_prompt  TEXT,
      default_strength REAL NOT NULL DEFAULT 1.0,
      strength_min     REAL NOT NULL DEFAULT 0.5,
      strength_max     REAL NOT NULL DEFAULT 1.5,
      category         TEXT,
      thumbnail_path   TEXT,
      bundle_id        TEXT,
      is_favorite      INTEGER NOT NULL DEFAULT 0,
      usage_count      INTEGER NOT NULL DEFAULT 0,
      created_at       INTEGER NOT NULL,
      updated_at       INTEGER NOT NULL
    );

    CREATE TABLE tags (
      id       INTEGER PRIMARY KEY AUTOINCREMENT,
      name     TEXT NOT NULL UNIQUE,
      color    TEXT,
      category TEXT
    );

    CREATE TABLE style_tags (
      style_id TEXT    NOT NULL,
      tag_id   INTEGER NOT NULL,
      PRIMARY KEY (style_id, tag_id)
    );

    CREATE TABLE style_loras (
      id       INTEGER PRIMARY KEY AUTOINCREMENT,
      style_id TEXT NOT NULL,
      lora_id  TEXT NOT NULL,
      strength REAL NOT NULL DEFAULT 1.0,
      priority INTEGER NOT NULL DEFAULT 0,
      UNIQUE (style_id, lora_id)
    );

    CREATE TABLE loras (
      id            TEXT PRIMARY KEY,
      name          TEXT NOT NULL,
      path          TEXT NOT NULL,
      trigger_words TEXT,
      base_model    TEXT,
      bundle_id     TEXT,
      size_bytes    INTEGER,
      strength      REAL NOT NULL DEFAULT 1.0,
      is_active     INTEGER NOT NULL DEFAULT 0,
      created_at    INTEGER NOT NULL,
      metadata      TEXT
    );

    CREATE TABLE examples (
      id           TEXT PRIMARY KEY,
      entity_type  TEXT NOT NULL,
      entity_id    TEXT NOT NULL,
      example_type TEXT NOT NULL,
      content      TEXT NOT NULL,
      created_at   INTEGER NOT NULL
    );

    CREATE TABLE conversations (
      id         TEXT PRIMARY KEY,
      title      TEXT NOT NULL,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );

    CREATE TABLE conversation_messages (
      id              TEXT PRIMARY KEY,
      conversation_id TEXT NOT NULL,
      role            TEXT NOT NULL,
      content         TEXT NOT NULL,
      tool_calls      TEXT,
      tool_results    TEXT,
      created_at      INTEGER NOT NULL
    );

    CREATE TABLE bundles (
      id               TEXT PRIMARY KEY,
      transformer_type TEXT NOT NULL
    );
  `);
  return db;
}
