use crate::config;
use crate::db::{now_ms, Database};
use crate::error::AppResult;
use crate::models::ToolAdapterConfig;

pub struct ToolAdapterConfigsRepository<'a> {
    db: &'a Database,
}

impl<'a> ToolAdapterConfigsRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list_enabled(&self) -> AppResult<Vec<ToolAdapterConfig>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT tool_key, display_name, skills_dir, detect_dir,
                        project_skills_dir,
                        supports_symlink, supports_junction, force_copy,
                        supports_project_scope, is_custom, enabled, sort_order, updated_at
                 FROM tool_adapter_configs
                 WHERE enabled = 1
                 ORDER BY sort_order ASC, is_custom ASC",
            )?;
            let rows = stmt.query_map([], row_to_tool_adapter_config)?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn upsert(&self, config: &mut ToolAdapterConfig) -> AppResult<()> {
        if config.sort_order == 0.0 {
            let existing_order = self.db.with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT sort_order FROM tool_adapter_configs WHERE tool_key = ?1")?;
                let result = stmt.query_row([&config.tool_key], |row| row.get(0));
                match result {
                    Ok(order) => Ok(Some(order)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e),
                }
            })?;

            if let Some(order) = existing_order {
                config.sort_order = order;
            } else {
                let max_order: f64 = self.db.with_conn(|conn| {
                    conn.query_row(
                        "SELECT COALESCE(MAX(sort_order), 0) FROM tool_adapter_configs",
                        [],
                        |row| row.get(0),
                    )
                })?;
                config.sort_order = max_order + 1.0;
            }
        }

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO tool_adapter_configs (
                   tool_key, display_name, skills_dir, detect_dir,
                   project_skills_dir,
                   supports_symlink, supports_junction, force_copy,
                   supports_project_scope, is_custom, enabled, sort_order, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                 ON CONFLICT(tool_key) DO UPDATE SET
                   display_name = excluded.display_name,
                   skills_dir = excluded.skills_dir,
                   detect_dir = excluded.detect_dir,
                   project_skills_dir = excluded.project_skills_dir,
                   supports_symlink = excluded.supports_symlink,
                   supports_junction = excluded.supports_junction,
                   force_copy = excluded.force_copy,
                   supports_project_scope = excluded.supports_project_scope,
                   is_custom = excluded.is_custom,
                   enabled = excluded.enabled,
                   sort_order = excluded.sort_order,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    config.tool_key,
                    config.display_name,
                    config.skills_dir,
                    config.detect_dir,
                    config.project_skills_dir,
                    config.supports_symlink as i32,
                    config.supports_junction as i32,
                    config.force_copy as i32,
                    config.supports_project_scope.map(|b| b as i32),
                    config.is_custom as i32,
                    config.enabled as i32,
                    config.sort_order,
                    config.updated_at,
                ],
            )?;
            Ok(())
        })
    }

    pub fn delete(&self, tool_key: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "DELETE FROM tool_adapter_configs WHERE tool_key = ?1",
                [tool_key],
            )?;
            conn.execute(
                "DELETE FROM tool_skill_cache WHERE tool_key = ?1",
                [tool_key],
            )?;
            conn.execute(
                "DELETE FROM tool_scan_state WHERE tool_key = ?1",
                [tool_key],
            )?;
            Ok(())
        })
    }

    pub fn reset_to_default(&self, tool_key: &str) -> AppResult<bool> {
        let adapters = config::default_tool_adapters();
        let default_cfg = match adapters.get(tool_key) {
            Some(cfg) => cfg,
            None => return Ok(false),
        };

        let now = now_ms();

        self.db.with_conn(|conn| {
            let existing_order: f64 = {
                let mut stmt = conn
                    .prepare("SELECT sort_order FROM tool_adapter_configs WHERE tool_key = ?1")?;
                stmt.query_row([tool_key], |row| row.get(0)).unwrap_or(0.0)
            };

            conn.execute(
                "INSERT INTO tool_adapter_configs
                 (tool_key, display_name, skills_dir, detect_dir, project_skills_dir,
                  supports_symlink, supports_junction, force_copy, supports_project_scope,
                  is_custom, enabled, sort_order, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 1, ?10, ?11)
                 ON CONFLICT(tool_key) DO UPDATE SET
                   display_name = excluded.display_name,
                   skills_dir = excluded.skills_dir,
                   detect_dir = excluded.detect_dir,
                   project_skills_dir = excluded.project_skills_dir,
                   supports_symlink = excluded.supports_symlink,
                   supports_junction = excluded.supports_junction,
                   force_copy = excluded.force_copy,
                   supports_project_scope = excluded.supports_project_scope,
                   is_custom = 0,
                   enabled = 1,
                   sort_order = excluded.sort_order,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    tool_key,
                    default_cfg.display_name,
                    default_cfg.skills_dir,
                    default_cfg.detect_dir,
                    default_cfg.project_skills_dir,
                    default_cfg.supports_symlink as i32,
                    default_cfg.supports_junction as i32,
                    default_cfg.force_copy as i32,
                    default_cfg.supports_project_scope.map(|b| b as i32),
                    existing_order,
                    now,
                ],
            )?;

            conn.execute(
                "DELETE FROM tool_skill_cache WHERE tool_key = ?1",
                [tool_key],
            )?;
            conn.execute(
                "DELETE FROM tool_scan_state WHERE tool_key = ?1",
                [tool_key],
            )?;

            Ok(true)
        })
    }

    pub fn reorder(&self, items: &[(String, f64)]) -> AppResult<()> {
        self.db.with_conn(|conn| {
            for (tool_key, sort_order) in items {
                conn.execute(
                    "UPDATE tool_adapter_configs SET sort_order = ?1 WHERE tool_key = ?2",
                    rusqlite::params![sort_order, tool_key],
                )?;
            }
            Ok(())
        })
    }
}

fn row_to_tool_adapter_config(row: &rusqlite::Row) -> rusqlite::Result<ToolAdapterConfig> {
    let supports_project_scope: Option<i32> = row.get(8)?;
    Ok(ToolAdapterConfig {
        tool_key: row.get(0)?,
        display_name: row.get(1)?,
        skills_dir: row.get(2)?,
        detect_dir: row.get(3)?,
        project_skills_dir: row.get(4)?,
        supports_symlink: row.get::<_, i32>(5)? != 0,
        supports_junction: row.get::<_, i32>(6)? != 0,
        force_copy: row.get::<_, i32>(7)? != 0,
        supports_project_scope: supports_project_scope.map(|v| v != 0),
        is_custom: row.get::<_, i32>(9)? != 0,
        enabled: row.get::<_, i32>(10)? != 0,
        sort_order: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_adapter_configs_list() {
        let db = Database::new_in_memory().unwrap();
        let repo = ToolAdapterConfigsRepository::new(&db);

        let configs = repo.list_enabled().unwrap();
        assert!(configs.len() >= 40);
    }

    #[test]
    fn test_tool_adapter_configs_reset() {
        let db = Database::new_in_memory().unwrap();
        let repo = ToolAdapterConfigsRepository::new(&db);

        let result = repo.reset_to_default("cursor").unwrap();
        assert!(result);

        let result = repo.reset_to_default("nonexistent").unwrap();
        assert!(!result);
    }
}
