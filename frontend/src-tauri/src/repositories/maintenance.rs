use crate::db::Database;
use crate::error::{AppError, AppResult};

pub struct MaintenanceRepository<'a> {
    db: &'a Database,
}

impl<'a> MaintenanceRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn vacuum(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("VACUUM")?;
            Ok(())
        })
    }

    pub fn analyze(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("ANALYZE")?;
            Ok(())
        })
    }

    pub fn clear_cache(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("DELETE FROM tool_skill_cache; DELETE FROM tool_scan_state;")?;
            Ok(())
        })
    }

    pub fn clear_discovered(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("DELETE FROM discovered_skills;")?;
            Ok(())
        })
    }

    pub fn integrity_check(&self) -> AppResult<Vec<String>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare("PRAGMA integrity_check")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn get_table_count(&self, table: &str) -> AppResult<i64> {
        let allowed_tables = [
            "skills",
            "skill_targets",
            "skill_tags",
            "skill_tag_links",
            "settings",
            "discovered_skills",
            "tool_scan_state",
            "tool_skill_cache",
            "tool_adapter_configs",
            "skill_scope_preference",
            "recent_projects",
            "skill_usage",
        ];

        if !allowed_tables.contains(&table) {
            return Err(AppError::Unexpected(format!(
                "Invalid table name: {}",
                table
            )));
        }

        self.db.with_conn(|conn| {
            let sql = format!("SELECT COUNT(*) FROM {}", table);
            conn.query_row(&sql, [], |row| row.get(0))
                .map_err(|e| e.into())
        })
    }

    pub fn get_overview(&self) -> AppResult<DatabaseOverview> {
        let tables = [
            "skills",
            "skill_targets",
            "skill_tags",
            "skill_tag_links",
            "settings",
            "discovered_skills",
            "tool_scan_state",
            "tool_skill_cache",
            "tool_adapter_configs",
            "skill_scope_preference",
            "recent_projects",
            "skill_usage",
        ];

        let mut counts = Vec::new();
        for table in &tables {
            let count = self.get_table_count(table)?;
            counts.push((table.to_string(), count));
        }

        Ok(DatabaseOverview { tables: counts })
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseOverview {
    pub tables: Vec<(String, i64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maintenance_operations() {
        let db = Database::new_in_memory().unwrap();
        let repo = MaintenanceRepository::new(&db);

        repo.analyze().unwrap();
        repo.vacuum().unwrap();

        let results = repo.integrity_check().unwrap();
        assert_eq!(results, vec!["ok"]);
    }

    #[test]
    fn test_get_overview() {
        let db = Database::new_in_memory().unwrap();
        let repo = MaintenanceRepository::new(&db);

        let overview = repo.get_overview().unwrap();
        assert_eq!(overview.tables.len(), 12);
    }

    #[test]
    fn test_invalid_table_name() {
        let db = Database::new_in_memory().unwrap();
        let repo = MaintenanceRepository::new(&db);

        let result = repo.get_table_count("invalid_table");
        assert!(result.is_err());
    }
}
