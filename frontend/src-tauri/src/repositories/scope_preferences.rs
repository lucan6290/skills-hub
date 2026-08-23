use crate::db::{now_ms, Database};
use crate::error::AppResult;
use crate::models::ScopePreference;

pub struct ScopePreferencesRepository<'a> {
    db: &'a Database,
}

impl<'a> ScopePreferencesRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn get(&self, skill_id: &str) -> AppResult<Option<ScopePreference>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT skill_id, scope, project_paths, updated_at
                 FROM skill_scope_preference WHERE skill_id = ?1",
            )?;
            let result = stmt.query_row([skill_id], row_to_scope_preference);
            match result {
                Ok(pref) => Ok(Some(pref)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    pub fn set(&self, skill_id: &str, scope: &str, project_paths: &str) -> AppResult<()> {
        let now = now_ms();
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO skill_scope_preference (skill_id, scope, project_paths, updated_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(skill_id) DO UPDATE SET
                   scope = excluded.scope,
                   project_paths = excluded.project_paths,
                   updated_at = excluded.updated_at",
                rusqlite::params![skill_id, scope, project_paths, now],
            )?;
            Ok(())
        })
    }

    pub fn list_all(&self) -> AppResult<Vec<ScopePreference>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT skill_id, scope, project_paths, updated_at
                 FROM skill_scope_preference",
            )?;
            let rows = stmt.query_map([], row_to_scope_preference)?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }
}

fn row_to_scope_preference(row: &rusqlite::Row) -> rusqlite::Result<ScopePreference> {
    Ok(ScopePreference {
        skill_id: row.get(0)?,
        scope: row.get(1)?,
        project_paths: row.get(2)?,
        updated_at: row.get(3)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_preferences_crud() {
        let db = Database::new_in_memory().unwrap();
        let repo = ScopePreferencesRepository::new(&db);

        assert!(repo.get("skill-1").unwrap().is_none());

        repo.set("skill-1", "project", "[\"/path1\"]").unwrap();
        let pref = repo.get("skill-1").unwrap().unwrap();
        assert_eq!(pref.scope, "project");
        assert_eq!(pref.project_paths, "[\"/path1\"]");

        repo.set("skill-1", "global", "[]").unwrap();
        let pref = repo.get("skill-1").unwrap().unwrap();
        assert_eq!(pref.scope, "global");

        let all = repo.list_all().unwrap();
        assert_eq!(all.len(), 1);
    }
}
