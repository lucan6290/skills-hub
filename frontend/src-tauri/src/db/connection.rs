use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, Result as SqlResult};

use super::schema;
use crate::error::{AppError, AppResult};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(db_path: P) -> AppResult<Self> {
        let conn = Connection::open(db_path.as_ref())
            .map_err(|e| AppError::Unexpected(format!("Failed to open database: {}", e)))?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| AppError::Unexpected(format!("Failed to set PRAGMA: {}", e)))?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        db.ensure_schema()?;

        Ok(db)
    }

    pub fn new_in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory().map_err(|e| {
            AppError::Unexpected(format!("Failed to open in-memory database: {}", e))
        })?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| AppError::Unexpected(format!("Failed to set PRAGMA: {}", e)))?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        db.ensure_schema()?;

        Ok(db)
    }

    pub fn with_conn<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> SqlResult<T>,
    {
        let guard = self
            .conn
            .lock()
            .map_err(|e| AppError::Unexpected(format!("Failed to acquire database lock: {}", e)))?;
        f(&guard).map_err(|e| {
            log::warn!("[DB_ERROR] with_conn failed: {}", e);
            AppError::Unexpected(format!("Database error: {}", e))
        })
    }

    pub fn with_conn_mut<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> SqlResult<T>,
    {
        let guard = self
            .conn
            .lock()
            .map_err(|e| AppError::Unexpected(format!("Failed to acquire database lock: {}", e)))?;
        f(&guard).map_err(|e| {
            log::warn!("[DB_ERROR] with_conn_mut failed: {}", e);
            AppError::Unexpected(format!("Database error: {}", e))
        })
    }

    fn ensure_schema(&self) -> AppResult<()> {
        self.with_conn(|conn| schema::ensure_schema(conn))?;
        self.initialize_tool_adapter_configs()?;
        Ok(())
    }

    fn initialize_tool_adapter_configs(&self) -> AppResult<()> {
        use crate::config::default_tool_adapters;

        let adapters = default_tool_adapters();
        let now = now_ms();

        self.with_conn(|conn| {
            let existing_keys: Vec<String> = {
                let mut stmt = conn.prepare("SELECT tool_key FROM tool_adapter_configs")?;
                let rows = stmt.query_map([], |row| row.get(0))?;
                rows.collect::<SqlResult<Vec<_>>>()?
            };

            let mut order = 1.0f64;
            for (key, cfg) in &adapters {
                if !existing_keys.contains(key) {
                    conn.execute(
                        "INSERT OR IGNORE INTO tool_adapter_configs
                         (tool_key, display_name, skills_dir, detect_dir, project_skills_dir,
                          supports_symlink, supports_junction, force_copy, supports_project_scope,
                          is_custom, enabled, sort_order, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 1, ?10, ?11)",
                        rusqlite::params![
                            key,
                            cfg.display_name,
                            cfg.skills_dir,
                            cfg.detect_dir,
                            cfg.project_skills_dir,
                            cfg.supports_symlink as i32,
                            cfg.supports_junction as i32,
                            cfg.force_copy as i32,
                            cfg.supports_project_scope.map(|b| b as i32),
                            order,
                            now,
                        ],
                    )?;
                } else {
                    conn.execute(
                        "UPDATE tool_adapter_configs
                         SET skills_dir = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ?1 ELSE skills_dir END,
                             detect_dir = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ?2 ELSE detect_dir END,
                             project_skills_dir = CASE WHEN skills_dir = '' AND detect_dir = '' AND project_skills_dir IS NULL THEN ?3 ELSE project_skills_dir END,
                             supports_symlink = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ?4 ELSE supports_symlink END,
                             supports_junction = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ?5 ELSE supports_junction END,
                             force_copy = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ?6 ELSE force_copy END,
                             supports_project_scope = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ?7 ELSE supports_project_scope END,
                             sort_order = CASE WHEN sort_order = 0 THEN ?8 ELSE sort_order END,
                             updated_at = CASE WHEN (skills_dir = '' AND detect_dir = '') OR sort_order = 0 THEN ?9 ELSE updated_at END
                         WHERE tool_key = ?10",
                        rusqlite::params![
                            cfg.skills_dir,
                            cfg.detect_dir,
                            cfg.project_skills_dir,
                            cfg.supports_symlink as i32,
                            cfg.supports_junction as i32,
                            cfg.force_copy as i32,
                            cfg.supports_project_scope.map(|b| b as i32),
                            order,
                            now,
                            key,
                        ],
                    )?;
                }
                order += 1.0;
            }

            let custom_rows: Vec<String> = {
                let mut stmt = conn.prepare(
                    "SELECT tool_key FROM tool_adapter_configs WHERE is_custom = 1 AND sort_order = 0"
                )?;
                let rows = stmt.query_map([], |row| row.get(0))?;
                rows.collect::<SqlResult<Vec<_>>>()?
            };

            for key in custom_rows {
                conn.execute(
                    "UPDATE tool_adapter_configs SET sort_order = ?1 WHERE tool_key = ?2",
                    rusqlite::params![order, key],
                )?;
                order += 1.0;
            }

            Ok(())
        })
    }
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_in_memory() {
        let db = Database::new_in_memory().expect("Failed to create in-memory database");
        db.with_conn(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )?;
            assert!(count >= 12);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_tool_adapter_configs_initialized() {
        let db = Database::new_in_memory().expect("Failed to create in-memory database");
        db.with_conn(|conn| {
            let count: i64 =
                conn.query_row("SELECT COUNT(*) FROM tool_adapter_configs", [], |row| {
                    row.get(0)
                })?;
            assert!(count >= 40);
            Ok(())
        })
        .unwrap();
    }
}
