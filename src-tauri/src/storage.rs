use chrono::Utc;
use rusqlite::{params, Connection, Transaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use thiserror::Error;

// ─── Error Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Backup integrity check failed: expected SHA-256 {expected}, got {actual}")]
    BackupChecksumMismatch { expected: String, actual: String },

    #[error("Backup archive is missing or has an invalid format: {0}")]
    BackupFormatInvalid(String),

    #[error("Migration error: {0}")]
    MigrationFailed(String),
}

// ─── Domain Models ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesktopBox {
    pub id: Option<i64>,
    pub name: String,
    pub folder_path: String,
    pub display_mode: String, // "grid" | "list"
    pub is_capsule_collapsed: bool,
    pub sort_order: String, // "name" | "type" | "date" | "size" | "manual"
    pub position_x: f64,
    pub position_y: f64,
    pub width: f64,
    pub height: f64,
    pub display_id: Option<String>,
    pub group_id: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoxGroup {
    pub id: Option<i64>,
    pub name: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrganizeRule {
    pub id: Option<i64>,
    pub name: String,
    pub extensions: String, // comma-separated, e.g. "pdf,docx,txt"
    pub target_folder: String,
    pub is_enabled: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskItem {
    pub id: Option<i64>,
    pub title: String,
    pub description: String,
    pub period: String, // "morning" | "today" | "evening"
    pub is_completed: bool,
    pub tags: String, // comma-separated
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptureNote {
    pub id: Option<i64>,
    pub content: String,
    pub note_type: String, // "text" | "link" | "snippet"
    pub tags: String,
    pub source_url: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UndoOperation {
    pub id: Option<i64>,
    pub batch_id: String,
    pub source_path: String,
    pub destination_path: String,
    pub file_hash: String, // SHA-256
    pub file_size: i64,
    pub status: String, // "PENDING" | "COMPLETED" | "FAILED" | "UNDONE"
    pub error_message: Option<String>,
    pub created_at: Option<String>,
}

// ─── Schema & Migrations ─────────────────────────────────────────────────────

const MIGRATION_V1: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS box_groups (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS desktop_boxes (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    name                 TEXT NOT NULL,
    folder_path          TEXT NOT NULL,
    display_mode         TEXT NOT NULL DEFAULT 'grid',
    is_capsule_collapsed INTEGER NOT NULL DEFAULT 0,
    sort_order           TEXT NOT NULL DEFAULT 'name',
    position_x           REAL NOT NULL DEFAULT 100.0,
    position_y           REAL NOT NULL DEFAULT 100.0,
    width                REAL NOT NULL DEFAULT 320.0,
    height               REAL NOT NULL DEFAULT 400.0,
    display_id           TEXT,
    group_id             INTEGER REFERENCES box_groups(id) ON DELETE SET NULL,
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS organize_rules (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    extensions    TEXT NOT NULL,
    target_folder TEXT NOT NULL,
    is_enabled    INTEGER NOT NULL DEFAULT 1,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS task_items (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    title        TEXT NOT NULL,
    description  TEXT NOT NULL DEFAULT '',
    period       TEXT NOT NULL DEFAULT 'today',
    is_completed INTEGER NOT NULL DEFAULT 0,
    tags         TEXT NOT NULL DEFAULT '',
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS capture_notes (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    content    TEXT NOT NULL,
    note_type  TEXT NOT NULL DEFAULT 'text',
    tags       TEXT NOT NULL DEFAULT '',
    source_url TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS undo_operations (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_id         TEXT NOT NULL,
    source_path      TEXT NOT NULL,
    destination_path TEXT NOT NULL,
    file_hash        TEXT NOT NULL,
    file_size        INTEGER NOT NULL,
    status           TEXT NOT NULL DEFAULT 'PENDING',
    error_message    TEXT,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_undo_batch ON undo_operations(batch_id);
CREATE INDEX IF NOT EXISTS idx_undo_status ON undo_operations(status);
CREATE INDEX IF NOT EXISTS idx_boxes_group ON desktop_boxes(group_id);
CREATE INDEX IF NOT EXISTS idx_boxes_display ON desktop_boxes(display_id);
CREATE INDEX IF NOT EXISTS idx_tasks_period ON task_items(period);
CREATE INDEX IF NOT EXISTS idx_notes_type ON capture_notes(note_type);
"#;

/// Run all migrations idempotently: safe to call on an empty DB or an existing one.
pub fn run_migrations(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let _current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |_| Ok(0i64),
        )
        .unwrap_or(0);

    // Check if schema_version table exists and has data
    let has_schema_table: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    let db_version: i64 = if has_schema_table {
        conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
    } else {
        0
    };

    if db_version < 1 {
        conn.execute_batch(MIGRATION_V1)
            .map_err(|e| StorageError::MigrationFailed(format!("V1 migration failed: {e}")))?;
        conn.execute(
            "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
            params![1],
        )?;
    }

    Ok(())
}

// ─── Storage Layer (CRUD) ─────────────────────────────────────────────────────

pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Open (or create) a database at the given path and run migrations.
    pub fn open(db_path: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open(db_path)?;
        run_migrations(&conn)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for tests).
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        run_migrations(&conn)?;
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    // ── Box Groups ────────────────────────────────────────────────────────

    pub fn insert_box_group(&self, group: &BoxGroup) -> Result<i64, StorageError> {
        self.conn.execute(
            "INSERT INTO box_groups (name) VALUES (?1)",
            params![group.name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_box_group(&self, id: i64) -> Result<BoxGroup, StorageError> {
        self.conn
            .query_row(
                "SELECT id, name, created_at FROM box_groups WHERE id = ?1",
                params![id],
                |row| {
                    Ok(BoxGroup {
                        id: Some(row.get(0)?),
                        name: row.get(1)?,
                        created_at: Some(row.get(2)?),
                    })
                },
            )
            .map_err(StorageError::from)
    }

    // ── Desktop Boxes ─────────────────────────────────────────────────────

    pub fn insert_desktop_box(&self, b: &DesktopBox) -> Result<i64, StorageError> {
        self.conn.execute(
            "INSERT INTO desktop_boxes (name, folder_path, display_mode, is_capsule_collapsed, sort_order, position_x, position_y, width, height, display_id, group_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                b.name,
                b.folder_path,
                b.display_mode,
                b.is_capsule_collapsed as i32,
                b.sort_order,
                b.position_x,
                b.position_y,
                b.width,
                b.height,
                b.display_id,
                b.group_id,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_desktop_box(&self, id: i64) -> Result<DesktopBox, StorageError> {
        self.conn.query_row(
            "SELECT id, name, folder_path, display_mode, is_capsule_collapsed, sort_order, position_x, position_y, width, height, display_id, group_id, created_at, updated_at FROM desktop_boxes WHERE id = ?1",
            params![id],
            |row| {
                Ok(DesktopBox {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    folder_path: row.get(2)?,
                    display_mode: row.get(3)?,
                    is_capsule_collapsed: row.get::<_, i32>(4)? != 0,
                    sort_order: row.get(5)?,
                    position_x: row.get(6)?,
                    position_y: row.get(7)?,
                    width: row.get(8)?,
                    height: row.get(9)?,
                    display_id: row.get(10)?,
                    group_id: row.get(11)?,
                    created_at: Some(row.get(12)?),
                    updated_at: Some(row.get(13)?),
                })
            },
        ).map_err(StorageError::from)
    }

    pub fn list_desktop_boxes(&self) -> Result<Vec<DesktopBox>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, folder_path, display_mode, is_capsule_collapsed, sort_order, position_x, position_y, width, height, display_id, group_id, created_at, updated_at FROM desktop_boxes ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DesktopBox {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                folder_path: row.get(2)?,
                display_mode: row.get(3)?,
                is_capsule_collapsed: row.get::<_, i32>(4)? != 0,
                sort_order: row.get(5)?,
                position_x: row.get(6)?,
                position_y: row.get(7)?,
                width: row.get(8)?,
                height: row.get(9)?,
                display_id: row.get(10)?,
                group_id: row.get(11)?,
                created_at: Some(row.get(12)?),
                updated_at: Some(row.get(13)?),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn update_desktop_box(&self, b: &DesktopBox) -> Result<(), StorageError> {
        let Some(box_id) = b.id else {
            return Err(StorageError::MigrationFailed(
                "Cannot update desktop box without id".into(),
            ));
        };
        self.conn.execute(
            "UPDATE desktop_boxes SET 
                name = ?1,
                folder_path = ?2,
                display_mode = ?3,
                is_capsule_collapsed = ?4,
                sort_order = ?5,
                position_x = ?6,
                position_y = ?7,
                width = ?8,
                height = ?9,
                display_id = ?10,
                group_id = ?11,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?12",
            params![
                b.name,
                b.folder_path,
                b.display_mode,
                b.is_capsule_collapsed as i32,
                b.sort_order,
                b.position_x,
                b.position_y,
                b.width,
                b.height,
                b.display_id,
                b.group_id,
                box_id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_desktop_box(&self, id: i64) -> Result<(), StorageError> {
        self.conn
            .execute("DELETE FROM desktop_boxes WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_boxes_by_group(&self, group_id: i64) -> Result<Vec<DesktopBox>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, folder_path, display_mode, is_capsule_collapsed, sort_order, position_x, position_y, width, height, display_id, group_id, created_at, updated_at FROM desktop_boxes WHERE group_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![group_id], |row| {
            Ok(DesktopBox {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                folder_path: row.get(2)?,
                display_mode: row.get(3)?,
                is_capsule_collapsed: row.get::<_, i32>(4)? != 0,
                sort_order: row.get(5)?,
                position_x: row.get(6)?,
                position_y: row.get(7)?,
                width: row.get(8)?,
                height: row.get(9)?,
                display_id: row.get(10)?,
                group_id: row.get(11)?,
                created_at: Some(row.get(12)?),
                updated_at: Some(row.get(13)?),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn delete_box_group(&self, group_id: i64) -> Result<(), StorageError> {
        self.conn.execute(
            "UPDATE desktop_boxes SET group_id = NULL WHERE group_id = ?1",
            params![group_id],
        )?;
        self.conn
            .execute("DELETE FROM box_groups WHERE id = ?1", params![group_id])?;
        Ok(())
    }

    // ── Organize Rules ────────────────────────────────────────────────────

    pub fn insert_organize_rule(&self, rule: &OrganizeRule) -> Result<i64, StorageError> {
        self.conn.execute(
            "INSERT INTO organize_rules (name, extensions, target_folder, is_enabled) VALUES (?1, ?2, ?3, ?4)",
            params![rule.name, rule.extensions, rule.target_folder, rule.is_enabled as i32],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // ── Task Items ────────────────────────────────────────────────────────

    pub fn insert_task(&self, task: &TaskItem) -> Result<i64, StorageError> {
        self.conn.execute(
            "INSERT INTO task_items (title, description, period, is_completed, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![task.title, task.description, task.period, task.is_completed as i32, task.tags],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_task(&self, id: i64) -> Result<TaskItem, StorageError> {
        self.conn.query_row(
            "SELECT id, title, description, period, is_completed, tags, created_at, updated_at FROM task_items WHERE id = ?1",
            params![id],
            |row| {
                Ok(TaskItem {
                    id: Some(row.get(0)?),
                    title: row.get(1)?,
                    description: row.get(2)?,
                    period: row.get(3)?,
                    is_completed: row.get::<_, i32>(4)? != 0,
                    tags: row.get(5)?,
                    created_at: Some(row.get(6)?),
                    updated_at: Some(row.get(7)?),
                })
            },
        ).map_err(StorageError::from)
    }

    // ── Capture Notes ─────────────────────────────────────────────────────

    pub fn insert_capture_note(&self, note: &CaptureNote) -> Result<i64, StorageError> {
        self.conn.execute(
            "INSERT INTO capture_notes (content, note_type, tags, source_url) VALUES (?1, ?2, ?3, ?4)",
            params![note.content, note.note_type, note.tags, note.source_url],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_capture_note(&self, id: i64) -> Result<CaptureNote, StorageError> {
        self.conn.query_row(
            "SELECT id, content, note_type, tags, source_url, created_at FROM capture_notes WHERE id = ?1",
            params![id],
            |row| {
                Ok(CaptureNote {
                    id: Some(row.get(0)?),
                    content: row.get(1)?,
                    note_type: row.get(2)?,
                    tags: row.get(3)?,
                    source_url: row.get(4)?,
                    created_at: Some(row.get(5)?),
                })
            },
        ).map_err(StorageError::from)
    }

    // ── Undo Operations ───────────────────────────────────────────────────

    pub fn insert_undo_operation(&self, op: &UndoOperation) -> Result<i64, StorageError> {
        self.conn.execute(
            "INSERT INTO undo_operations (batch_id, source_path, destination_path, file_hash, file_size, status, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                op.batch_id,
                op.source_path,
                op.destination_path,
                op.file_hash,
                op.file_size,
                op.status,
                op.error_message,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_undo_operations_by_batch(
        &self,
        batch_id: &str,
    ) -> Result<Vec<UndoOperation>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, batch_id, source_path, destination_path, file_hash, file_size, status, error_message, created_at FROM undo_operations WHERE batch_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![batch_id], |row| {
            Ok(UndoOperation {
                id: Some(row.get(0)?),
                batch_id: row.get(1)?,
                source_path: row.get(2)?,
                destination_path: row.get(3)?,
                file_hash: row.get(4)?,
                file_size: row.get(5)?,
                status: row.get(6)?,
                error_message: row.get(7)?,
                created_at: Some(row.get(8)?),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    // ── Multi-table atomic transaction (for testing rollback) ─────────────

    /// Executes a closure inside a transaction. Rolls back automatically if the closure returns Err.
    pub fn with_transaction<F, T>(&mut self, f: F) -> Result<T, StorageError>
    where
        F: FnOnce(&Transaction<'_>) -> Result<T, StorageError>,
    {
        let tx = self.conn.transaction()?;
        match f(&tx) {
            Ok(val) => {
                tx.commit()?;
                Ok(val)
            }
            Err(e) => {
                // tx is automatically rolled back on drop
                Err(e)
            }
        }
    }
}

// ─── Backup & Restore ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupArchive {
    pub version: i32,
    pub created_at: String,
    pub sha256: String,
    pub data: BackupData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub box_groups: Vec<BoxGroup>,
    pub desktop_boxes: Vec<DesktopBox>,
    pub organize_rules: Vec<OrganizeRule>,
    pub task_items: Vec<TaskItem>,
    pub capture_notes: Vec<CaptureNote>,
}

/// Compute SHA-256 hex digest of a byte slice.
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Exports all data from the database into a JSON backup archive with SHA-256 integrity.
pub fn export_backup(storage: &Storage) -> Result<String, StorageError> {
    let box_groups = {
        let mut stmt = storage
            .connection()
            .prepare("SELECT id, name, created_at FROM box_groups ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(BoxGroup {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                created_at: Some(row.get(2)?),
            })
        })?;
        let result: Vec<BoxGroup> = rows.collect::<Result<Vec<_>, _>>()?;
        result
    };

    let desktop_boxes = storage.list_desktop_boxes()?;

    let organize_rules = {
        let mut stmt = storage.connection().prepare(
            "SELECT id, name, extensions, target_folder, is_enabled, created_at FROM organize_rules ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(OrganizeRule {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                extensions: row.get(2)?,
                target_folder: row.get(3)?,
                is_enabled: row.get::<_, i32>(4)? != 0,
                created_at: Some(row.get(5)?),
            })
        })?;
        let result: Vec<OrganizeRule> = rows.collect::<Result<Vec<_>, _>>()?;
        result
    };

    let task_items = {
        let mut stmt = storage.connection().prepare(
            "SELECT id, title, description, period, is_completed, tags, created_at, updated_at FROM task_items ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TaskItem {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                description: row.get(2)?,
                period: row.get(3)?,
                is_completed: row.get::<_, i32>(4)? != 0,
                tags: row.get(5)?,
                created_at: Some(row.get(6)?),
                updated_at: Some(row.get(7)?),
            })
        })?;
        let result: Vec<TaskItem> = rows.collect::<Result<Vec<_>, _>>()?;
        result
    };

    let capture_notes = {
        let mut stmt = storage.connection().prepare(
            "SELECT id, content, note_type, tags, source_url, created_at FROM capture_notes ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CaptureNote {
                id: Some(row.get(0)?),
                content: row.get(1)?,
                note_type: row.get(2)?,
                tags: row.get(3)?,
                source_url: row.get(4)?,
                created_at: Some(row.get(5)?),
            })
        })?;
        let result: Vec<CaptureNote> = rows.collect::<Result<Vec<_>, _>>()?;
        result
    };

    let data = BackupData {
        box_groups,
        desktop_boxes,
        organize_rules,
        task_items,
        capture_notes,
    };

    let data_json = serde_json::to_string_pretty(&data)?;
    let checksum = sha256_hex(data_json.as_bytes());

    let archive = BackupArchive {
        version: 1,
        created_at: Utc::now().to_rfc3339(),
        sha256: checksum,
        data,
    };

    Ok(serde_json::to_string_pretty(&archive)?)
}

/// Imports data from a JSON backup archive, verifying its SHA-256 integrity first.
/// **Does not destroy existing data** — only inserts records from the archive.
pub fn import_backup(storage: &mut Storage, archive_json: &str) -> Result<(), StorageError> {
    let archive: BackupArchive = serde_json::from_str(archive_json).map_err(|e| {
        StorageError::BackupFormatInvalid(format!("Failed to parse backup JSON: {e}"))
    })?;

    if archive.version != 1 {
        return Err(StorageError::BackupFormatInvalid(format!(
            "Unsupported backup version: {}",
            archive.version
        )));
    }

    // Verify SHA-256 integrity: re-serialize the data section and compare
    let data_json = serde_json::to_string_pretty(&archive.data)?;
    let actual_hash = sha256_hex(data_json.as_bytes());

    if actual_hash != archive.sha256 {
        return Err(StorageError::BackupChecksumMismatch {
            expected: archive.sha256,
            actual: actual_hash,
        });
    }

    // All good — insert into database inside a transaction
    storage.with_transaction(|tx| {
        for group in &archive.data.box_groups {
            tx.execute(
                "INSERT INTO box_groups (name) VALUES (?1)",
                params![group.name],
            )?;
        }
        for b in &archive.data.desktop_boxes {
            tx.execute(
                "INSERT INTO desktop_boxes (name, folder_path, display_mode, is_capsule_collapsed, sort_order, position_x, position_y, width, height, display_id, group_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![b.name, b.folder_path, b.display_mode, b.is_capsule_collapsed as i32, b.sort_order, b.position_x, b.position_y, b.width, b.height, b.display_id, b.group_id],
            )?;
        }
        for rule in &archive.data.organize_rules {
            tx.execute(
                "INSERT INTO organize_rules (name, extensions, target_folder, is_enabled) VALUES (?1,?2,?3,?4)",
                params![rule.name, rule.extensions, rule.target_folder, rule.is_enabled as i32],
            )?;
        }
        for task in &archive.data.task_items {
            tx.execute(
                "INSERT INTO task_items (title, description, period, is_completed, tags) VALUES (?1,?2,?3,?4,?5)",
                params![task.title, task.description, task.period, task.is_completed as i32, task.tags],
            )?;
        }
        for note in &archive.data.capture_notes {
            tx.execute(
                "INSERT INTO capture_notes (content, note_type, tags, source_url) VALUES (?1,?2,?3,?4)",
                params![note.content, note.note_type, note.tags, note.source_url],
            )?;
        }
        Ok(())
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn make_test_storage() -> Storage {
        Storage::open_in_memory().expect("Failed to open in-memory database")
    }

    // ── Required Stage 2 Test #1 ──────────────────────────────────────────
    #[test]
    fn test_migrations_idempotent_and_clean() {
        // Run migrations on a brand new empty database
        let storage = make_test_storage();

        // Verify schema_version is set
        let version: i64 = storage
            .connection()
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 1);

        // Verify all tables exist
        let tables: Vec<String> = {
            let mut stmt = storage
                .connection()
                .prepare(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
                )
                .unwrap();
            stmt.query_map([], |row| row.get(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        };
        assert!(tables.contains(&"desktop_boxes".to_string()));
        assert!(tables.contains(&"box_groups".to_string()));
        assert!(tables.contains(&"organize_rules".to_string()));
        assert!(tables.contains(&"task_items".to_string()));
        assert!(tables.contains(&"capture_notes".to_string()));
        assert!(tables.contains(&"undo_operations".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));

        // Insert some data before re-running migrations to prove idempotency
        let task_id = storage
            .insert_task(&TaskItem {
                id: None,
                title: "مهمة الاختبار — Test Task".to_string(),
                description: "وصف".to_string(),
                period: "today".to_string(),
                is_completed: false,
                tags: "test,اختبار".to_string(),
                created_at: None,
                updated_at: None,
            })
            .unwrap();
        assert!(task_id > 0);

        // Re-run migrations on the same database with data — must succeed without data loss
        run_migrations(storage.connection()).expect("Idempotent re-migration must succeed");

        // Verify existing data survived
        let task = storage.get_task(task_id).unwrap();
        assert_eq!(task.title, "مهمة الاختبار — Test Task");
        assert_eq!(task.tags, "test,اختبار");

        // Also test on a file-backed database to prove WAL mode activation
        let tmp = NamedTempFile::new().unwrap();
        let file_storage = Storage::open(tmp.path()).expect("File-backed DB must open");
        let wal_mode: String = file_storage
            .connection()
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(wal_mode.to_lowercase(), "wal");
    }

    // ── Required Stage 2 Test #2 ──────────────────────────────────────────
    #[test]
    fn test_concurrent_reads_and_atomic_rollback_on_error() {
        let mut storage = make_test_storage();

        // Insert a task before the transaction
        let pre_task_id = storage
            .insert_task(&TaskItem {
                id: None,
                title: "مهمة قبل المعاملة".to_string(),
                description: "".to_string(),
                period: "morning".to_string(),
                is_completed: false,
                tags: "".to_string(),
                created_at: None,
                updated_at: None,
            })
            .unwrap();

        // Attempt a multi-table transaction that FAILS partway through
        let result = storage.with_transaction(|tx| {
            // Insert a box group — this should succeed inside the transaction
            tx.execute(
                "INSERT INTO box_groups (name) VALUES (?1)",
                params!["مجموعة ستُلغى"],
            )?;

            // Insert a desktop box — also succeeds inside the transaction
            tx.execute(
                "INSERT INTO desktop_boxes (name, folder_path) VALUES (?1, ?2)",
                params!["صندوق مؤقت", "/tmp/rollback-test"],
            )?;

            // Now deliberately cause an error (violate NOT NULL on a required column)
            tx.execute("INSERT INTO task_items (title) VALUES (NULL)", [])
                .map_err(StorageError::from)?;

            Ok(())
        });

        // The transaction must have failed
        assert!(result.is_err(), "Transaction with NULL violation must fail");

        // Verify ROLLBACK: the box_group and desktop_box inserted inside the failed tx must NOT exist
        let group_count: i64 = storage
            .connection()
            .query_row("SELECT COUNT(*) FROM box_groups", [], |row| row.get(0))
            .unwrap();
        assert_eq!(group_count, 0, "Rolled-back box_group must not persist");

        let box_count: i64 = storage
            .connection()
            .query_row("SELECT COUNT(*) FROM desktop_boxes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(box_count, 0, "Rolled-back desktop_box must not persist");

        // Verify data inserted BEFORE the failed transaction is intact
        let pre_task = storage.get_task(pre_task_id).unwrap();
        assert_eq!(pre_task.title, "مهمة قبل المعاملة");

        // Verify the database is not corrupted — a new transaction can succeed after the rollback
        let success_result = storage.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO box_groups (name) VALUES (?1)",
                params!["مجموعة ناجحة بعد التراجع"],
            )?;
            Ok(())
        });
        assert!(
            success_result.is_ok(),
            "Database must be usable after rollback"
        );

        let final_group_count: i64 = storage
            .connection()
            .query_row("SELECT COUNT(*) FROM box_groups", [], |row| row.get(0))
            .unwrap();
        assert_eq!(final_group_count, 1);
    }

    // ── Required Stage 2 Test #3 ──────────────────────────────────────────
    #[test]
    fn test_arabic_and_emoji_unicode_roundtrip() {
        let storage = make_test_storage();

        // Arabic-only name
        let box_id = storage
            .insert_desktop_box(&DesktopBox {
                id: None,
                name: "صندوق_المشاريع_العربية".to_string(),
                folder_path: "/Users/أحمد/Desktop/مشاريع_رَوَاق_2026".to_string(),
                display_mode: "grid".to_string(),
                is_capsule_collapsed: false,
                sort_order: "name".to_string(),
                position_x: 150.5,
                position_y: 200.0,
                width: 320.0,
                height: 400.0,
                display_id: Some("شاشة_سامسونج_الرئيسية".to_string()),
                group_id: None,
                created_at: None,
                updated_at: None,
            })
            .unwrap();

        let b = storage.get_desktop_box(box_id).unwrap();
        assert_eq!(b.name, "صندوق_المشاريع_العربية");
        assert_eq!(b.folder_path, "/Users/أحمد/Desktop/مشاريع_رَوَاق_2026");
        assert_eq!(b.display_id.as_deref(), Some("شاشة_سامسونج_الرئيسية"));

        // Mixed Arabic + English + numbers + special chars
        let task_id = storage
            .insert_task(&TaskItem {
                id: None,
                title: "مراجعة ملف Rawaq_تقرير(v2).pdf — الإصدار النهائي #42".to_string(),
                description:
                    "تحقق من التوافق مع أنظمة Windows 11 و macOS Sonoma 🍎 ولينكس Ubuntu 🐧"
                        .to_string(),
                period: "today".to_string(),
                is_completed: false,
                tags: "رَوَاق,review,مراجعة,v2".to_string(),
                created_at: None,
                updated_at: None,
            })
            .unwrap();

        let task = storage.get_task(task_id).unwrap();
        assert_eq!(
            task.title,
            "مراجعة ملف Rawaq_تقرير(v2).pdf — الإصدار النهائي #42"
        );
        assert!(task.description.contains("🍎"));
        assert!(task.description.contains("🐧"));
        assert_eq!(task.tags, "رَوَاق,review,مراجعة,v2");

        // Emoji-heavy capture note
        let note_id = storage
            .insert_capture_note(&CaptureNote {
                id: None,
                content: "🚀💡🎯 فكرة المشروع: تنظيم سطح المكتب 📁🗂️ مع كبسولة علوية 🏝️"
                    .to_string(),
                note_type: "text".to_string(),
                tags: "💡,فكرة,🚀".to_string(),
                source_url: Some("https://github.com/Heisen-lab/rawaq/تقرير".to_string()),
                created_at: None,
            })
            .unwrap();

        let note = storage.get_capture_note(note_id).unwrap();
        assert!(note.content.contains("🚀💡🎯"));
        assert!(note.content.contains("📁🗂️"));
        assert!(note.content.contains("🏝️"));
        assert_eq!(note.tags, "💡,فكرة,🚀");
        assert_eq!(
            note.source_url.as_deref(),
            Some("https://github.com/Heisen-lab/rawaq/تقرير")
        );

        // Undo operation with Arabic paths
        let _op_id = storage
            .insert_undo_operation(&UndoOperation {
                id: None,
                batch_id: "batch-عربي-001".to_string(),
                source_path: "C:\\Users\\سلمى\\Desktop\\تقرير_الميزانية_2026.xlsx".to_string(),
                destination_path: "C:\\Users\\سلمى\\Documents\\مالية\\تقرير_الميزانية_2026.xlsx"
                    .to_string(),
                file_hash: "abc123def456".to_string(),
                file_size: 1048576,
                status: "COMPLETED".to_string(),
                error_message: None,
                created_at: None,
            })
            .unwrap();

        let ops = storage
            .get_undo_operations_by_batch("batch-عربي-001")
            .unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(
            ops[0].source_path,
            "C:\\Users\\سلمى\\Desktop\\تقرير_الميزانية_2026.xlsx"
        );
        assert_eq!(ops[0].batch_id, "batch-عربي-001");

        // Edge case: empty string values (allowed, not NULL)
        let empty_task_id = storage
            .insert_task(&TaskItem {
                id: None,
                title: "".to_string(),
                description: "".to_string(),
                period: "evening".to_string(),
                is_completed: true,
                tags: "".to_string(),
                created_at: None,
                updated_at: None,
            })
            .unwrap();

        let empty_task = storage.get_task(empty_task_id).unwrap();
        assert_eq!(empty_task.title, "");
        assert!(empty_task.is_completed);
    }

    // ── Required Stage 2 Test #4 ──────────────────────────────────────────
    #[test]
    fn test_rejects_corrupted_or_tampered_backup_archive() {
        let storage = make_test_storage();

        // Populate some data
        storage
            .insert_task(&TaskItem {
                id: None,
                title: "مهمة النسخ الاحتياطي".to_string(),
                description: "Backup test task".to_string(),
                period: "today".to_string(),
                is_completed: false,
                tags: "backup,نسخ".to_string(),
                created_at: None,
                updated_at: None,
            })
            .unwrap();

        storage
            .insert_capture_note(&CaptureNote {
                id: None,
                content: "ملاحظة مهمة 📝 للنسخ الاحتياطي".to_string(),
                note_type: "text".to_string(),
                tags: "نسخ".to_string(),
                source_url: None,
                created_at: None,
            })
            .unwrap();

        // Export a valid backup
        let valid_backup = export_backup(&storage).expect("Export must succeed");

        // Verify the backup is valid JSON with a checksum
        let archive: BackupArchive =
            serde_json::from_str(&valid_backup).expect("Backup must be valid JSON");
        assert_eq!(archive.version, 1);
        assert!(!archive.sha256.is_empty());

        // Import into a fresh database — must succeed
        let mut fresh_storage = make_test_storage();
        import_backup(&mut fresh_storage, &valid_backup).expect("Valid backup import must succeed");

        // Verify imported data
        let tasks: i64 = fresh_storage
            .connection()
            .query_row("SELECT COUNT(*) FROM task_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(tasks, 1);

        let notes: i64 = fresh_storage
            .connection()
            .query_row("SELECT COUNT(*) FROM capture_notes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(notes, 1);

        // ── Tampered backup: modify data content but keep old SHA-256 ──────
        let tampered = valid_backup.replace("مهمة النسخ الاحتياطي", "TAMPERED CONTENT تم التلاعب");
        let mut tampered_storage = make_test_storage();
        let result = import_backup(&mut tampered_storage, &tampered);
        assert!(result.is_err(), "Tampered backup must be rejected");
        match result.unwrap_err() {
            StorageError::BackupChecksumMismatch { expected, actual } => {
                assert_ne!(expected, actual);
            }
            other => panic!("Expected BackupChecksumMismatch, got: {other}"),
        }

        // Verify the tampered import did NOT pollute the database
        let tampered_task_count: i64 = tampered_storage
            .connection()
            .query_row("SELECT COUNT(*) FROM task_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            tampered_task_count, 0,
            "Tampered backup must not insert any data"
        );

        // ── Corrupted JSON ────────────────────────────────────────────────
        let corrupted_json = "{ this is not valid JSON at all }}}";
        let mut corrupt_storage = make_test_storage();
        let result = import_backup(&mut corrupt_storage, corrupted_json);
        assert!(result.is_err(), "Corrupted JSON must be rejected");

        // ── Wrong version ─────────────────────────────────────────────────
        let wrong_version = valid_backup.replace("\"version\": 1", "\"version\": 999");
        let mut wrong_ver_storage = make_test_storage();
        let result = import_backup(&mut wrong_ver_storage, &wrong_version);
        assert!(result.is_err(), "Wrong version must be rejected");

        // ── Empty string backup ───────────────────────────────────────────
        let mut empty_storage = make_test_storage();
        let result = import_backup(&mut empty_storage, "");
        assert!(result.is_err(), "Empty backup string must be rejected");

        // ── Verify the original database is completely unaffected ──────────
        let original_task = storage.get_task(1).unwrap();
        assert_eq!(original_task.title, "مهمة النسخ الاحتياطي");
    }
}
