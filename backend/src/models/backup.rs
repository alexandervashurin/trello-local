// backend/src/models/backup.rs
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use sqlx::sqlite::SqliteRow;

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct Backup {
    pub id: i64,
    pub filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub created_by: i64,
    pub created_at: i64,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateBackup {
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct RestoreBackup {
    pub backup_id: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct BackupList {
    pub id: i64,
    pub filename: String,
    pub file_size: i64,
    pub created_by: i64,
    pub created_at: i64,
    pub description: Option<String>,
    pub creator_username: String,
}

impl<'r> FromRow<'r, SqliteRow> for BackupList {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            filename: row.try_get("filename")?,
            file_size: row.try_get("file_size")?,
            created_by: row.try_get("created_by")?,
            created_at: row.try_get("created_at")?,
            description: row.try_get("description")?,
            creator_username: row.try_get("creator_username")?,
        })
    }
}
